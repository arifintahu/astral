use axum::{
    extract::{Query, State},
    response::{sse::{Event, Sse}, IntoResponse},
    routing::get,
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use crate::db::Db;
use crate::metrics::SystemMetrics;

#[derive(Debug, Clone, Serialize)]
pub struct AlertEvent {
    pub kind: String,    // "cpu" or "ram"
    pub message: String,
    pub value: f64,
    pub threshold: f64,
    pub timestamp: i64,
}

pub fn create_alert_channel() -> broadcast::Sender<AlertEvent> {
    let (tx, _) = broadcast::channel::<AlertEvent>(32);
    tx
}

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub tx: broadcast::Sender<SystemMetrics>,
    pub alert_tx: broadcast::Sender<AlertEvent>,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/api/stream", get(sse_handler))
        .route("/api/history", get(history_handler))
        .route("/api/alerts", get(alerts_handler))
        .with_state(state)
}

async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx);

    Sse::new(stream.map(|msg| {
        match msg {
            Ok(metrics) => {
                let data = serde_json::to_string(&metrics).unwrap_or_default();
                Ok(Event::default().data(data))
            }
            Err(_) => Ok(Event::default().comment("missed message")),
        }
    }))
    .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(1)))
}

async fn alerts_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.alert_tx.subscribe();
    let stream = BroadcastStream::new(rx);

    Sse::new(stream.map(|msg| {
        match msg {
            Ok(alert) => {
                let data = serde_json::to_string(&alert).unwrap_or_default();
                Ok(Event::default().event("alert").data(data))
            }
            Err(_) => Ok(Event::default().comment("missed")),
        }
    }))
    .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(5)))
}

#[derive(Deserialize)]
struct HistoryQuery {
    window: String, // 6h, 24h, 7d, all
}

async fn history_handler(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> impl IntoResponse {
    let (table, duration) = match query.window.as_str() {
        "6h" => ("metrics_5m", 6 * 3600),
        "24h" => ("metrics_15m", 24 * 3600),
        "7d" => ("metrics_1h", 7 * 24 * 3600),
        "all" => ("metrics_1h", 90 * 24 * 3600),
        _ => ("metrics_5m", 6 * 3600),
    };

    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let from_ts = now - duration;

    match state.db.get_history(table, from_ts).await {
        Ok(data) => Json(data).into_response(),
        Err(e) => {
            tracing::error!("History query failed: {:#}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
