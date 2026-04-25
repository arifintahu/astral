use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
    routing::get,
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::{broadcast, Mutex};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::config::SharedConfig;
use crate::db::Db;
use crate::metrics::SystemMetrics;

pub type AlertHistory = Arc<Mutex<VecDeque<AlertEvent>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub kind: String,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
    pub timestamp: i64,
}

pub fn create_alert_channel() -> broadcast::Sender<AlertEvent> {
    let (tx, _) = broadcast::channel::<AlertEvent>(32);
    tx
}

pub fn create_alert_history() -> AlertHistory {
    Arc::new(Mutex::new(VecDeque::with_capacity(50)))
}

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub tx: broadcast::Sender<SystemMetrics>,
    pub alert_tx: broadcast::Sender<AlertEvent>,
    pub config: SharedConfig,
    pub alert_history: AlertHistory,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/api/stream", get(sse_handler))
        .route("/api/history", get(history_handler))
        .route("/api/history/export", get(export_handler))
        .route("/api/alerts", get(alerts_handler))
        .route("/api/alerts/history", get(alerts_history_handler))
        .route("/api/settings", get(get_settings_handler).post(post_settings_handler))
        .with_state(state)
}

async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx);

    Sse::new(stream.map(|msg| match msg {
        Ok(metrics) => {
            let data = serde_json::to_string(&metrics).unwrap_or_default();
            Ok(Event::default().data(data))
        }
        Err(_) => Ok(Event::default().comment("missed message")),
    }))
    .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(1)))
}

async fn alerts_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.alert_tx.subscribe();
    let stream = BroadcastStream::new(rx);

    Sse::new(stream.map(|msg| match msg {
        Ok(alert) => {
            let data = serde_json::to_string(&alert).unwrap_or_default();
            Ok(Event::default().event("alert").data(data))
        }
        Err(_) => Ok(Event::default().comment("missed")),
    }))
    .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(5)))
}

async fn alerts_history_handler(State(state): State<AppState>) -> impl IntoResponse {
    let history = state.alert_history.lock().await;
    let alerts: Vec<AlertEvent> = history.iter().cloned().collect();
    Json(alerts)
}

async fn get_settings_handler(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.read().await;
    Json(config.clone())
}

#[derive(Deserialize)]
struct SettingsUpdate {
    enable_process_list: Option<bool>,
    alert_cpu: Option<f32>,
    alert_ram: Option<f32>,
    retention_days: Option<u64>,
    slack_webhook: Option<String>,
}

async fn post_settings_handler(
    State(state): State<AppState>,
    Json(body): Json<SettingsUpdate>,
) -> impl IntoResponse {
    let mut config = state.config.write().await;
    if let Some(v) = body.enable_process_list {
        config.enable_process_list = v;
    }
    if let Some(v) = body.alert_cpu {
        if v > 0.0 && v <= 100.0 {
            config.alert_cpu = v;
        }
    }
    if let Some(v) = body.alert_ram {
        if v > 0.0 && v <= 100.0 {
            config.alert_ram = v;
        }
    }
    if let Some(v) = body.retention_days {
        if v >= 1 && v <= 365 {
            config.retention_days = v;
        }
    }
    if let Some(v) = body.slack_webhook {
        config.slack_webhook = if v.trim().is_empty() { None } else { Some(v) };
    }
    StatusCode::OK
}

#[derive(Deserialize)]
struct HistoryQuery {
    window: Option<String>,
}

fn resolve_window(window: &str) -> (&'static str, i64) {
    match window {
        "6h" => ("metrics_5m", 6 * 3600),
        "24h" => ("metrics_15m", 24 * 3600),
        "7d" => ("metrics_1h", 7 * 24 * 3600),
        "all" => ("metrics_1h", 90 * 24 * 3600),
        _ => ("metrics_5m", 6 * 3600),
    }
}

async fn history_handler(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> impl IntoResponse {
    let window = query.window.as_deref().unwrap_or("6h");
    let (table, duration) = resolve_window(window);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let from_ts = now - duration;

    match state.db.get_history(table, from_ts).await {
        Ok(data) => Json(data).into_response(),
        Err(e) => {
            tracing::error!("History query failed: {:#}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn export_handler(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> impl IntoResponse {
    let window = query.window.as_deref().unwrap_or("6h");
    let (table, duration) = resolve_window(window);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let from_ts = now - duration;

    match state.db.get_history(table, from_ts).await {
        Ok(data) => {
            let mut csv = String::from(
                "timestamp,cpu_usage,used_memory_bytes,network_tx_bps,network_rx_bps,disk_read_bps,disk_write_bps\n",
            );
            for point in &data {
                csv.push_str(&format!(
                    "{},{:.2},{},{},{},{},{}\n",
                    point.timestamp,
                    point.cpu_usage,
                    point.used_memory,
                    point.network_tx,
                    point.network_rx,
                    point.disk_read_rate,
                    point.disk_write_rate,
                ));
            }
            let disposition =
                format!("attachment; filename=\"astral-history-{}.csv\"", window);
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, "text/csv".parse().unwrap());
            headers.insert(
                header::CONTENT_DISPOSITION,
                disposition.parse().unwrap(),
            );
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(csv))
                .map(|mut r| {
                    *r.headers_mut() = headers;
                    r
                })
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => {
            tracing::error!("Export query failed: {:#}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
