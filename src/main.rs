use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{header, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use clap::Parser;
use hmac::{Hmac, Mac};
use rust_embed::RustEmbed;
use sha2::Sha256;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, Mutex};

mod api;
mod db;
mod metrics;
mod worker;

use api::AppState;
use db::Db;
use metrics::SystemMetrics;
use worker::Worker;

type HmacSha256 = Hmac<Sha256>;
type RevocationSet = Arc<Mutex<HashSet<String>>>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "8080")]
    port: u16,

    #[arg(long, default_value = "7")]
    retention: u64,

    /// Credentials as USER:PASS. Also read from the ASTRAL_AUTH environment variable.
    #[arg(long, env = "ASTRAL_AUTH")]
    auth: Option<String>,

    #[arg(long)]
    webhook: Option<String>,

    #[arg(long, default_value = "90")]
    alert_cpu: f32,

    #[arg(long, default_value = "90")]
    alert_ram: f32,

    /// Include the process list in the SSE stream (opt-in, defaults to off).
    #[arg(long)]
    enable_process_list: bool,
}

#[derive(RustEmbed)]
#[folder = "web/dist"]
struct Assets;

#[derive(Clone)]
struct AuthConfig {
    username: String,
    password_hash: String,
    secret: String,
    revocation: RevocationSet,
}

const MAX_LOGIN_ATTEMPTS: u32 = 5;
const RATE_LIMIT_WINDOW_SECS: u64 = 60;
const SESSION_MAX_AGE_SECS: u64 = 86400;

#[derive(Clone)]
struct LoginLimiter {
    attempts: Arc<Mutex<HashMap<std::net::IpAddr, (u32, Instant)>>>,
}

impl LoginLimiter {
    fn new() -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn check_rate_limit(&self, ip: std::net::IpAddr) -> bool {
        let mut map = self.attempts.lock().await;
        let now = Instant::now();
        let entry = map.entry(ip).or_insert((0, now));
        if now.duration_since(entry.1) > Duration::from_secs(RATE_LIMIT_WINDOW_SECS) {
            *entry = (1, now);
            true
        } else if entry.0 >= MAX_LOGIN_ATTEMPTS {
            false
        } else {
            entry.0 += 1;
            true
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    // C-1: Credentials readable from ASTRAL_AUTH env var (via clap `env`).
    // M-2: Use split_once so passwords containing ':' are accepted.
    let (username, password) = if let Some(auth_str) = args.auth {
        let (u, p) = auth_str
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("Invalid auth format. Expected USER:PASS"))?;
        (u.to_string(), p.to_string())
    } else {
        let username = "admin".to_string();
        let password = uuid::Uuid::new_v4().to_string();
        // M-3: Write to stderr so credentials are not captured by stdout log pipelines.
        eprintln!("No auth provided — generated credentials (save these now):");
        eprintln!("  Username : {}", username);
        eprintln!("  Password : {}", password);
        eprintln!("  Persist  : set ASTRAL_AUTH={}:<password> or use --auth", username);
        (username, password)
    };

    let salt = SaltString::generate(&mut OsRng);
    // L-1: Use ? instead of .expect() so errors propagate via anyhow.
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();

    let secret = uuid::Uuid::new_v4().to_string();
    let revocation: RevocationSet = Arc::new(Mutex::new(HashSet::new()));
    let auth_config = AuthConfig {
        username,
        password_hash,
        secret,
        revocation,
    };
    let limiter = LoginLimiter::new();

    // H-5: Periodically evict stale rate-limit entries to prevent unbounded HashMap growth.
    let limiter_cleanup = limiter.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(300));
        loop {
            tick.tick().await;
            let mut map = limiter_cleanup.attempts.lock().await;
            let now = Instant::now();
            map.retain(|_, (_, last)| {
                now.duration_since(*last) < Duration::from_secs(RATE_LIMIT_WINDOW_SECS * 10)
            });
        }
    });

    // L-2: WAL mode is enabled inside Db::init() via PRAGMA journal_mode=WAL.
    let db = Db::new("sqlite:astral.db?mode=rwc").await?;

    let (tx, _rx) = broadcast::channel::<SystemMetrics>(100);

    // M-5: Process list is opt-in via --enable-process-list.
    let tx_clone = tx.clone();
    let enable_procs = args.enable_process_list;
    tokio::spawn(async move {
        metrics::run_metrics_collector(tx_clone, enable_procs).await;
    });

    let worker_rx = tx.subscribe();
    let worker_db = db.clone();
    let alert_tx = api::create_alert_channel();
    let alert_tx_worker = alert_tx.clone();
    let worker = Worker::new(
        worker_db,
        worker_rx,
        args.alert_cpu,
        args.alert_ram,
        args.webhook,
        alert_tx_worker,
    );
    tokio::spawn(async move {
        worker.run().await;
    });

    let app_state = AppState { db, tx, alert_tx };
    let auth_config_clone = auth_config.clone();

    let api_router = api::app(app_state);

    // Auth middleware — checks HttpOnly session cookie and revocation set.
    let auth_layer = middleware::from_fn(move |req: Request<Body>, next: Next| {
        let config = auth_config_clone.clone();
        async move { auth_middleware(req, next, config).await }
    });

    let login_config = auth_config.clone();
    let login_limiter = limiter.clone();
    let public_router = Router::new()
        .route(
            "/api/login",
            post(move |ConnectInfo(addr): ConnectInfo<SocketAddr>, body: Json<LoginRequest>| {
                let config = login_config.clone();
                let limiter = login_limiter.clone();
                async move { handle_login(body, config, limiter, addr).await }
            }),
        )
        .route("/", get(index_handler))
        .route("/{*file}", get(static_handler));

    let logout_config = auth_config.clone();
    let protected_router = Router::new()
        .merge(api_router)
        .route("/api/auth/check", get(|| async { StatusCode::OK }))
        .route(
            "/api/logout",
            post(move |headers: axum::http::HeaderMap| {
                let config = logout_config.clone();
                async move { handle_logout(headers, config).await }
            }),
        )
        .layer(auth_layer);

    // H-3: Content-Security-Policy prevents XSS execution.
    // L-3: Strict-Transport-Security enforces HTTPS when served behind a TLS proxy.
    let security_headers = middleware::from_fn(|req: Request<Body>, next: Next| async move {
        let mut resp = next.run(req).await;
        let h = resp.headers_mut();
        h.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
        h.insert("X-Frame-Options", "DENY".parse().unwrap());
        h.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
        h.insert(
            "Content-Security-Policy",
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'"
                .parse()
                .unwrap(),
        );
        h.insert(
            "Strict-Transport-Security",
            "max-age=31536000; includeSubDomains".parse().unwrap(),
        );
        resp
    });

    let app = Router::new()
        .merge(protected_router)
        .merge(public_router)
        .layer(security_headers);

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}

/// H-1: Token format is `{timestamp}.{nonce}.{hmac}`.
/// The nonce prevents identical tokens when two logins occur within the same second.
fn create_session_token(secret: &str, username: &str) -> anyhow::Result<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    // Use UUID v4 as a 32-char hex nonce (128 bits of randomness).
    let nonce = uuid::Uuid::new_v4().to_string().replace('-', "");
    let mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| anyhow::anyhow!("HMAC key error: {}", e))?;
    let mut mac = mac;
    mac.update(format!("astral:{}:session:{}:{}", username, now, nonce).as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());
    Ok(format!("{}.{}.{}", now, nonce, sig))
}

fn verify_session_token(secret: &str, username: &str, token: &str) -> bool {
    // Expected format: {timestamp}.{nonce}.{sig}
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    if parts.len() != 3 {
        return false;
    }
    let timestamp: u64 = match parts[0].parse() {
        Ok(t) => t,
        Err(_) => return false,
    };
    let nonce = parts[1];
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now.saturating_sub(timestamp) > SESSION_MAX_AGE_SECS {
        return false;
    }
    let mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    let mut mac = mac;
    mac.update(format!("astral:{}:session:{}:{}", username, timestamp, nonce).as_bytes());
    let sig_bytes = match hex::decode(parts[2]) {
        Ok(b) => b,
        Err(_) => return false,
    };
    mac.verify_slice(&sig_bytes).is_ok()
}

/// Extract the signature component from a session token for revocation purposes.
fn token_sig(token: &str) -> Option<String> {
    token.splitn(3, '.').nth(2).map(|s| s.to_string())
}

/// Parse the session token from the Cookie request header.
fn get_cookie_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let cookie_str = headers.get("cookie")?.to_str().ok()?;
    for part in cookie_str.split(';') {
        if let Some(val) = part.trim().strip_prefix("astral_session=") {
            return Some(val.to_string());
        }
    }
    None
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(serde::Serialize)]
struct LoginResponse {
    username: String,
}

async fn handle_login(
    Json(body): Json<LoginRequest>,
    config: AuthConfig,
    limiter: LoginLimiter,
    addr: SocketAddr,
) -> Response {
    if !limiter.check_rate_limit(addr.ip()).await {
        return (StatusCode::TOO_MANY_REQUESTS, "Too many login attempts").into_response();
    }

    let password_ok = match PasswordHash::new(&config.password_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    };

    if body.username == config.username && password_ok {
        let token = match create_session_token(&config.secret, &body.username) {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Token creation failed: {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        // M-1: Deliver token via HttpOnly cookie — inaccessible to JavaScript.
        let cookie = format!(
            "astral_session={}; HttpOnly; Path=/; SameSite=Strict; Max-Age={}",
            token, SESSION_MAX_AGE_SECS
        );
        let cookie_val = match HeaderValue::from_str(&cookie) {
            Ok(v) => v,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        (
            StatusCode::OK,
            [(header::SET_COOKIE, cookie_val)],
            Json(LoginResponse { username: body.username }),
        )
            .into_response()
    } else {
        tracing::warn!("Failed login attempt for user: {}", body.username);
        StatusCode::UNAUTHORIZED.into_response()
    }
}

async fn handle_logout(headers: axum::http::HeaderMap, config: AuthConfig) -> Response {
    // H-2: Add token signature to revocation set so it cannot be reused after logout.
    if let Some(token) = get_cookie_token(&headers) {
        if let Some(sig) = token_sig(&token) {
            config.revocation.lock().await.insert(sig);
        }
    }
    let clear_cookie = HeaderValue::from_static(
        "astral_session=; HttpOnly; Path=/; SameSite=Strict; Max-Age=0",
    );
    (StatusCode::OK, [(header::SET_COOKIE, clear_cookie)]).into_response()
}

async fn auth_middleware(req: Request<Body>, next: Next, config: AuthConfig) -> Response {
    // M-1: Read token from HttpOnly cookie instead of Authorization header.
    if let Some(token) = get_cookie_token(req.headers()) {
        if verify_session_token(&config.secret, &config.username, &token) {
            // H-2: Reject tokens that have been explicitly revoked (e.g. after logout).
            let sig = token_sig(&token).unwrap_or_default();
            if !config.revocation.lock().await.contains(&sig) {
                return next.run(req).await;
            }
        }
    }
    StatusCode::UNAUTHORIZED.into_response()
}

async fn static_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
    } else {
        if !path.contains('.') {
            if let Some(content) = Assets::get("index.html") {
                let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                return (
                    [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                    content.data,
                )
                    .into_response();
            }
        }
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn index_handler() -> impl IntoResponse {
    static_handler(axum::extract::Path("index.html".to_string())).await
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Session Token Tests ---

    #[test]
    fn test_token_format_is_timestamp_dot_nonce_dot_hmac() {
        let token = create_session_token("secret", "admin").unwrap();
        let parts: Vec<&str> = token.splitn(3, '.').collect();
        assert_eq!(parts.len(), 3, "Token should be timestamp.nonce.hmac");
        assert!(parts[0].parse::<u64>().is_ok(), "First part should be a u64 timestamp");
        assert_eq!(parts[1].len(), 32, "Nonce should be 16 bytes = 32 hex chars");
        assert_eq!(parts[2].len(), 64, "HMAC-SHA256 hex should be 64 chars");
    }

    #[test]
    fn test_verify_accepts_valid_token() {
        let secret = "test_secret";
        let token = create_session_token(secret, "admin").unwrap();
        assert!(verify_session_token(secret, "admin", &token));
    }

    #[test]
    fn test_verify_rejects_wrong_secret() {
        let token = create_session_token("secret_a", "admin").unwrap();
        assert!(!verify_session_token("secret_b", "admin", &token));
    }

    #[test]
    fn test_verify_rejects_wrong_username() {
        let secret = "test_secret";
        let token = create_session_token(secret, "admin").unwrap();
        assert!(!verify_session_token(secret, "attacker", &token));
    }

    #[test]
    fn test_verify_rejects_tampered_signature() {
        let secret = "test_secret";
        let token = create_session_token(secret, "admin").unwrap();
        let parts: Vec<&str> = token.splitn(3, '.').collect();
        let tampered = format!("{}.{}.{}", parts[0], parts[1], "a".repeat(64));
        assert!(!verify_session_token(secret, "admin", &tampered));
    }

    #[test]
    fn test_verify_rejects_expired_token() {
        let secret = "test_secret";
        let old_ts: u64 = 1000;
        let nonce = "abcdef1234567890abcdef1234567890"; // 32 hex chars
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(format!("astral:admin:session:{}:{}", old_ts, nonce).as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());
        let expired_token = format!("{}.{}.{}", old_ts, nonce, sig);
        assert!(!verify_session_token(secret, "admin", &expired_token));
    }

    #[test]
    fn test_verify_rejects_malformed_tokens() {
        let secret = "s";
        assert!(!verify_session_token(secret, "u", ""));
        assert!(!verify_session_token(secret, "u", "no_dot"));
        assert!(!verify_session_token(secret, "u", "."));
        assert!(!verify_session_token(secret, "u", ".."));
        assert!(!verify_session_token(secret, "u", "notanumber.nonce.abc"));
        assert!(!verify_session_token(secret, "u", "123.nonce.not_hex_$$"));
        // Old 2-part format (no nonce) should be rejected
        assert!(!verify_session_token(secret, "u", "123.abc"));
    }

    #[test]
    fn test_different_users_get_different_tokens() {
        let secret = "shared_secret";
        let t1 = create_session_token(secret, "alice").unwrap();
        let t2 = create_session_token(secret, "bob").unwrap();
        let sig1 = t1.splitn(3, '.').nth(2).unwrap();
        let sig2 = t2.splitn(3, '.').nth(2).unwrap();
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_concurrent_logins_produce_different_tokens() {
        // H-1: Two logins for the same user at the same second must differ due to nonce.
        let secret = "test_secret";
        let t1 = create_session_token(secret, "admin").unwrap();
        let t2 = create_session_token(secret, "admin").unwrap();
        let nonce1 = t1.splitn(3, '.').nth(1).unwrap();
        let nonce2 = t2.splitn(3, '.').nth(1).unwrap();
        assert_ne!(nonce1, nonce2, "Nonces must be unique across concurrent logins");
    }

    // --- Rate Limiter Tests ---

    #[tokio::test]
    async fn test_limiter_allows_under_limit() {
        let limiter = LoginLimiter::new();
        let ip: std::net::IpAddr = "10.0.0.1".parse().unwrap();
        for _ in 0..MAX_LOGIN_ATTEMPTS {
            assert!(limiter.check_rate_limit(ip).await);
        }
    }

    #[tokio::test]
    async fn test_limiter_blocks_over_limit() {
        let limiter = LoginLimiter::new();
        let ip: std::net::IpAddr = "10.0.0.1".parse().unwrap();
        for _ in 0..MAX_LOGIN_ATTEMPTS {
            limiter.check_rate_limit(ip).await;
        }
        assert!(!limiter.check_rate_limit(ip).await, "Should block after max attempts");
    }

    #[tokio::test]
    async fn test_limiter_isolates_by_ip() {
        let limiter = LoginLimiter::new();
        let ip1: std::net::IpAddr = "10.0.0.1".parse().unwrap();
        let ip2: std::net::IpAddr = "10.0.0.2".parse().unwrap();
        for _ in 0..MAX_LOGIN_ATTEMPTS {
            limiter.check_rate_limit(ip1).await;
        }
        assert!(!limiter.check_rate_limit(ip1).await);
        assert!(limiter.check_rate_limit(ip2).await);
    }

    // --- Password Hashing Tests ---

    #[test]
    fn test_argon2_hash_and_verify() {
        let password = "correct_password";
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        let parsed = PasswordHash::new(&hash).unwrap();
        assert!(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok());
    }

    #[test]
    fn test_argon2_rejects_wrong_password() {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(b"correct", &salt)
            .unwrap()
            .to_string();
        let parsed = PasswordHash::new(&hash).unwrap();
        assert!(Argon2::default().verify_password(b"wrong", &parsed).is_err());
    }

    #[test]
    fn test_argon2_different_salts_produce_different_hashes() {
        let password = b"same_password";
        let h1 = Argon2::default()
            .hash_password(password, &SaltString::generate(&mut OsRng))
            .unwrap()
            .to_string();
        let h2 = Argon2::default()
            .hash_password(password, &SaltString::generate(&mut OsRng))
            .unwrap()
            .to_string();
        assert_ne!(h1, h2);
    }
}
