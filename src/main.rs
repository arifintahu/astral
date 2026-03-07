use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
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
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;

mod api;
mod db;
mod metrics;
mod worker;

use api::AppState;
use db::Db;
use metrics::SystemMetrics;
use worker::Worker;

type HmacSha256 = Hmac<Sha256>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "8080")]
    port: u16,

    #[arg(long, default_value = "7")]
    retention: u64,

    #[arg(long)]
    auth: Option<String>,

    #[arg(long)]
    webhook: Option<String>,

    #[arg(long, default_value = "90")]
    alert_cpu: f32,

    #[arg(long, default_value = "90")]
    alert_ram: f32,
}

#[derive(RustEmbed)]
#[folder = "web/dist"]
struct Assets;

#[derive(Clone)]
struct AuthConfig {
    username: String,
    password_hash: String,
    secret: String,
}

const MAX_LOGIN_ATTEMPTS: u32 = 5;
const RATE_LIMIT_WINDOW_SECS: u64 = 60;
const SESSION_MAX_AGE_SECS: u64 = 86400;

#[derive(Clone)]
struct LoginLimiter {
    attempts: std::sync::Arc<tokio::sync::Mutex<HashMap<std::net::IpAddr, (u32, Instant)>>>,
}

impl LoginLimiter {
    fn new() -> Self {
        Self {
            attempts: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    async fn check_rate_limit(&self, ip: std::net::IpAddr) -> bool {
        let mut map = self.attempts.lock().await;
        let now = Instant::now();
        let entry = map.entry(ip).or_insert((0, now));
        if now.duration_since(entry.1) > std::time::Duration::from_secs(RATE_LIMIT_WINDOW_SECS) {
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

    // Setup Auth
    let (username, password) = if let Some(auth_str) = args.auth {
        let parts: Vec<&str> = auth_str.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid auth format. Expected USER:PASS");
        }
        (parts[0].to_string(), parts[1].to_string())
    } else {
        let username = "admin".to_string();
        let password = uuid::Uuid::new_v4().to_string();
        println!("No auth provided. Generated credentials: {}:{}", username, password);
        (username, password)
    };

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    let secret = uuid::Uuid::new_v4().to_string();
    let auth_config = AuthConfig {
        username,
        password_hash,
        secret,
    };
    let limiter = LoginLimiter::new();

    // Initialize DB
    let db = Db::new("sqlite:astral.db?mode=rwc").await?;

    // Metrics Channel
    let (tx, _rx) = broadcast::channel::<SystemMetrics>(100);

    // Start Metrics Collector
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        metrics::run_metrics_collector(tx_clone).await;
    });

    // Start Worker
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

    // Setup Router
    let app_state = AppState { db, tx, alert_tx };
    let auth_config_clone = auth_config.clone();

    // API Router is already built in api::app(state)
    let api_router = api::app(app_state);

    // Auth Middleware
    let auth_layer = middleware::from_fn(move |req: Request<Body>, next: Next| {
        let config = auth_config_clone.clone();
        async move {
            auth_middleware(req, next, config).await
        }
    });

    // Public routes (Login + Static Files)
    let login_config = auth_config.clone();
    let login_limiter = limiter.clone();
    let public_router = Router::new()
        .route("/api/login", post(move |
            ConnectInfo(addr): ConnectInfo<SocketAddr>,
            body: Json<LoginRequest>,
        | {
            let config = login_config.clone();
            let limiter = login_limiter.clone();
            async move { handle_login(body, config, limiter, addr).await }
        }))
        .route("/", get(index_handler))
        .route("/{*file}", get(static_handler));

    // Protected API routes
    let protected_router = Router::new()
        .merge(api_router) // /api/stream, /api/history, /api/alerts
        .route("/api/auth/check", get(|| async { StatusCode::OK }))
        .layer(auth_layer);

    // Combine routers
    // Note: We merge protected_router FIRST to ensure its routes are registered with priority,
    // but the critical part is that auth_layer is already applied to it.
    // However, axum route matching is based on path specificity.
    // The fallback route `/{*file}` in public_router is very broad.
    // If we merge public_router (with fallback) first, it might shadow protected routes if they overlap or if fallback logic is aggressive.
    // Ideally, specific routes should take precedence.
    let security_headers = middleware::from_fn(|req: Request<Body>, next: Next| async move {
        let mut resp = next.run(req).await;
        let headers = resp.headers_mut();
        headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
        headers.insert("X-Frame-Options", "DENY".parse().unwrap());
        headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
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

fn create_session_token(secret: &str, username: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    let payload = format!("astral:{}:session:{}", username, now);
    mac.update(payload.as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());
    format!("{}.{}", now, sig)
}

fn verify_session_token(secret: &str, username: &str, token: &str) -> bool {
    let parts: Vec<&str> = token.splitn(2, '.').collect();
    if parts.len() != 2 {
        return false;
    }
    let timestamp: u64 = match parts[0].parse() {
        Ok(t) => t,
        Err(_) => return false,
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if now.saturating_sub(timestamp) > SESSION_MAX_AGE_SECS {
        return false;
    }
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    let payload = format!("astral:{}:session:{}", username, timestamp);
    mac.update(payload.as_bytes());
    let sig_bytes = match hex::decode(parts[1]) {
        Ok(b) => b,
        Err(_) => return false,
    };
    mac.verify_slice(&sig_bytes).is_ok()
}

#[derive(serde::Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(serde::Serialize)]
struct LoginResponse {
    token: String,
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
        let token = create_session_token(&config.secret, &body.username);
        let resp = LoginResponse {
            token,
            username: body.username,
        };
        Json(resp).into_response()
    } else {
        tracing::warn!("Failed login attempt for user: {}", body.username);
        StatusCode::UNAUTHORIZED.into_response()
    }
}

async fn auth_middleware(
    req: Request<Body>,
    next: Next,
    config: AuthConfig,
) -> Response {
    // Check for session token in Authorization: Bearer header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                if verify_session_token(&config.secret, &config.username, token) {
                    return next.run(req).await;
                }
            }
        }
    }

    StatusCode::UNAUTHORIZED.into_response()
}

async fn static_handler(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');

    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
    } else {
        // SPA fallback: serve index.html for non-file routes
        if !path.contains('.') {
            if let Some(content) = Assets::get("index.html") {
                let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                return ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], content.data).into_response();
            }
        }
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn index_handler() -> impl IntoResponse {
    static_handler(axum::extract::Path("index.html".to_string())).await
}
