use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use hmac::{Hmac, Mac};
use rust_embed::RustEmbed;
use sha2::Sha256;
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

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
    password: String,
    secret: String,
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

    let secret = uuid::Uuid::new_v4().to_string();
    let auth_config = AuthConfig {
        username,
        password,
        secret,
    };

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

    let api_router = api::app(app_state);

    // Auth Middleware — skips /api/login so the login endpoint is public
    let auth_layer = middleware::from_fn(move |req: Request<Body>, next: Next| {
        let config = auth_config_clone.clone();
        async move {
            auth_middleware(req, next, config).await
        }
    });

    // Login route (no auth required — placed outside the auth layer)
    let login_config = auth_config.clone();
    let login_router = Router::new()
        .route("/api/login", post(move |body: Json<LoginRequest>| {
            let config = login_config.clone();
            async move { handle_login(body, config) }
        }))
        .route("/api/auth/check", get(|| async { StatusCode::OK }));

    let app = Router::new()
        .merge(api_router)
        .route("/", get(index_handler))
        .route("/{*file}", get(static_handler))
        .layer(auth_layer)
        .merge(login_router)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_session_token(secret: &str, username: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    let payload = format!("astral:{}:{}", username, "session");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn verify_session_token(secret: &str, username: &str, token: &str) -> bool {
    let expected = create_session_token(secret, username);
    expected == token
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

fn handle_login(
    Json(body): Json<LoginRequest>,
    config: AuthConfig,
) -> Response {
    if body.username == config.username && body.password == config.password {
        let token = create_session_token(&config.secret, &body.username);
        let resp = LoginResponse {
            token,
            username: body.username,
        };
        Json(resp).into_response()
    } else {
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
