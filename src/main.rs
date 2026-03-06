use axum::{
    body::Body,
    http::{Request, StatusCode, HeaderValue},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use clap::Parser;
use rust_embed::RustEmbed;
use std::{net::SocketAddr, sync::Arc};
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

    // Initialize DB
    let db = Db::new("sqlite:astral.db?mode=rwc").await?;

    // Metrics Channel
    let (tx, rx) = broadcast::channel::<SystemMetrics>(100);

    // Start Metrics Collector
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        metrics::run_metrics_collector(tx_clone).await;
    });

    // Start Worker
    let worker_rx = tx.subscribe();
    let worker_db = db.clone();
    let worker = Worker::new(
        worker_db,
        worker_rx,
        args.alert_cpu,
        args.alert_ram,
        args.webhook,
    );
    tokio::spawn(async move {
        worker.run().await;
    });

    // Setup Router
    let app_state = AppState { db, tx };
    
    let api_router = api::app(app_state);
    
    // Auth Middleware
    let auth_layer = middleware::from_fn(move |req: Request<Body>, next: Next| {
        let username = username.clone();
        let password = password.clone();
        async move {
            auth_middleware(req, next, username, password).await
        }
    });

    let app = Router::new()
        .merge(api_router)
        .route("/", get(index_handler))
        .route("/{*file}", get(static_handler))
        .layer(auth_layer)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn auth_middleware(
    req: Request<Body>,
    next: Next,
    expected_user: String,
    expected_pass: String,
) -> Response {
    use base64::Engine;
    
    let auth_header = req.headers().get("Authorization");
    
    if let Some(header) = auth_header {
        if let Ok(auth_str) = header.to_str() {
            if auth_str.starts_with("Basic ") {
                let token = &auth_str[6..];
                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(token) {
                    if let Ok(cred) = String::from_utf8(decoded) {
                        let parts: Vec<&str> = cred.splitn(2, ':').collect();
                        if parts.len() == 2 && parts[0] == expected_user && parts[1] == expected_pass {
                            return next.run(req).await;
                        }
                    }
                }
            }
        }
    }

    let mut response = StatusCode::UNAUTHORIZED.into_response();
    response.headers_mut().insert(
        "WWW-Authenticate", 
        HeaderValue::from_static("Basic realm=\"Astral\"")
    );
    response
}

async fn static_handler(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    
    if let Some(content) = Assets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
    } else {
        // Fallback to index.html for SPA routing if we had client-side routing, 
        // but currently we just return 404 or maybe index.html?
        // PRD says "Single-Page Elegance", likely handled by Svelte.
        // If file not found, return 404.
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn index_handler() -> impl IntoResponse {
    static_handler(axum::extract::Path("index.html".to_string())).await
}
