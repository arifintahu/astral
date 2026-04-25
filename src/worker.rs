use crate::api::AlertEvent;
use crate::db::{Db, MetricPoint};
use crate::metrics::SystemMetrics;
use anyhow::Result;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tokio::time::{interval};

const ALERT_COOLDOWN_SECS: u64 = 15 * 60;

pub struct Worker {
    db: Db,
    metrics_rx: broadcast::Receiver<SystemMetrics>,
    buffer: Vec<SystemMetrics>,
    alert_cpu_threshold: f32,
    alert_ram_threshold: f32,
    webhook_url: Option<String>,
    alert_tx: broadcast::Sender<AlertEvent>,
    last_cpu_alert_at: Option<Instant>,
    last_ram_alert_at: Option<Instant>,
}

impl Worker {
    pub fn new(
        db: Db,
        metrics_rx: broadcast::Receiver<SystemMetrics>,
        alert_cpu_threshold: f32,
        alert_ram_threshold: f32,
        webhook_url: Option<String>,
        alert_tx: broadcast::Sender<AlertEvent>,
    ) -> Self {
        Self {
            db,
            metrics_rx,
            buffer: Vec::new(),
            alert_cpu_threshold,
            alert_ram_threshold,
            webhook_url,
            alert_tx,
            last_cpu_alert_at: None,
            last_ram_alert_at: None,
        }
    }

    pub async fn run(mut self) {
        let mut ticker_1m = interval(Duration::from_secs(60));
        let mut ticker_5m = interval(Duration::from_secs(300));
        let mut ticker_15m = interval(Duration::from_secs(900));
        let mut ticker_1h = interval(Duration::from_secs(3600));

        loop {
            tokio::select! {
                Ok(metric) = self.metrics_rx.recv() => {
                    self.buffer.push(metric);
                }
                _ = ticker_1m.tick() => {
                    if let Err(e) = self.process_1m().await {
                        eprintln!("Error processing 1m metrics: {}", e);
                    }
                }
                _ = ticker_5m.tick() => {
                    if let Err(e) = self.aggregate_and_store("metrics_1m", "metrics_5m", 5).await {
                         eprintln!("Error processing 5m metrics: {}", e);
                    }
                    let _ = self.db.cleanup("metrics_1m", 3600 * 2).await;
                }
                _ = ticker_15m.tick() => {
                    if let Err(e) = self.aggregate_and_store("metrics_5m", "metrics_15m", 3).await {
                         eprintln!("Error processing 15m metrics: {}", e);
                    }
                    let _ = self.db.cleanup("metrics_5m", 86400 * 2).await;
                }
                _ = ticker_1h.tick() => {
                    if let Err(e) = self.aggregate_and_store("metrics_15m", "metrics_1h", 4).await {
                         eprintln!("Error processing 1h metrics: {}", e);
                    }
                    let _ = self.db.cleanup("metrics_15m", 86400 * 7).await;
                    let _ = self.db.cleanup("metrics_1h", 86400 * 90).await;
                }
            }
        }
    }

    async fn process_1m(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let count = self.buffer.len() as f64;
        let avg_cpu: f64 = self.buffer.iter().map(|m| m.cpu_usage as f64).sum::<f64>() / count;
        let avg_mem: f64 = self.buffer.iter().map(|m| m.used_memory as f64).sum::<f64>() / count;
        let avg_tx: f64 = self.buffer.iter().map(|m| m.network_tx as f64).sum::<f64>() / count;
        let avg_rx: f64 = self.buffer.iter().map(|m| m.network_rx as f64).sum::<f64>() / count;

        let total_memory = self.buffer.last().map(|m| m.total_memory).unwrap_or(0);

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

        let point = MetricPoint {
            timestamp,
            cpu_usage: avg_cpu,
            used_memory: avg_mem as i64,
            network_tx: avg_tx as i64,
            network_rx: avg_rx as i64,
        };

        self.db.insert_metric("metrics_1m", point).await?;
        self.buffer.clear();

        let threshold_mem_bytes =
            (total_memory as f64 * self.alert_ram_threshold as f64 / 100.0) as i64;

        let alert_triggered = self
            .db
            .check_alert_condition("metrics_1m", self.alert_cpu_threshold as f64, threshold_mem_bytes, 300)
            .await?;

        if alert_triggered {
            self.send_alert(avg_cpu, avg_mem, total_memory, threshold_mem_bytes).await?;
        }

        Ok(())
    }

    async fn aggregate_and_store(
        &self,
        source_table: &str,
        target_table: &str,
        limit_minutes: i64,
    ) -> Result<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let start_ts = now - (limit_minutes * 60);

        if let Some(metric) = self.db.get_average(source_table, start_ts).await? {
            let point = MetricPoint { timestamp: now, ..metric };
            self.db.insert_metric(target_table, point).await?;
        }

        Ok(())
    }

    async fn send_alert(
        &mut self,
        avg_cpu: f64,
        avg_mem: f64,
        total_memory: u64,
        threshold_mem_bytes: i64,
    ) -> Result<()> {
        let now_sys = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let now_inst = Instant::now();

        let mem_percent = if total_memory > 0 {
            (avg_mem / total_memory as f64) * 100.0
        } else {
            0.0
        };

        // L-4: Suppress repeated alerts during sustained high load.
        let cpu_on_cooldown = self
            .last_cpu_alert_at
            .map_or(false, |t| now_inst.duration_since(t).as_secs() < ALERT_COOLDOWN_SECS);

        if avg_cpu > self.alert_cpu_threshold as f64 && !cpu_on_cooldown {
            let alert = AlertEvent {
                kind: "cpu".to_string(),
                message: format!(
                    "CPU usage at {:.1}% (threshold: {}%)",
                    avg_cpu, self.alert_cpu_threshold
                ),
                value: avg_cpu,
                threshold: self.alert_cpu_threshold as f64,
                timestamp: now_sys,
            };
            let _ = self.alert_tx.send(alert);
            self.last_cpu_alert_at = Some(now_inst);
        }

        let ram_on_cooldown = self
            .last_ram_alert_at
            .map_or(false, |t| now_inst.duration_since(t).as_secs() < ALERT_COOLDOWN_SECS);

        if avg_mem > threshold_mem_bytes as f64 && !ram_on_cooldown {
            let alert = AlertEvent {
                kind: "ram".to_string(),
                message: format!(
                    "Memory usage at {:.1}% (threshold: {}%)",
                    mem_percent, self.alert_ram_threshold
                ),
                value: mem_percent,
                threshold: self.alert_ram_threshold as f64,
                timestamp: now_sys,
            };
            let _ = self.alert_tx.send(alert);
            self.last_ram_alert_at = Some(now_inst);
        }

        // H-4: Only POST to HTTPS endpoints; M-4: enforce a 10-second timeout.
        if let Some(url) = &self.webhook_url {
            if !url.starts_with("https://") {
                tracing::warn!("Webhook URL must use HTTPS — skipping delivery. URL: {}", url);
            } else {
                let payload = serde_json::json!({
                    "content": format!(
                        "🚨 Astral Alert: CPU at {:.1}%, Memory at {:.1}%",
                        avg_cpu, mem_percent
                    )
                });
                match reqwest::Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()
                {
                    Ok(client) => {
                        if let Err(e) = client.post(url).json(&payload).send().await {
                            eprintln!("Failed to send webhook: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to build webhook client: {}", e),
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "worker_test.rs"]
mod tests;
