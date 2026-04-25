use super::*;
use crate::api::create_alert_channel;
use crate::db::{Db, MetricPoint};
use crate::metrics::SystemMetrics;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;

fn make_metrics(cpu: f32, mem: u64, tx: u64, rx: u64) -> SystemMetrics {
    SystemMetrics {
        hostname: "test".into(),
        os_name: "Linux".into(),
        os_version: "5.15".into(),
        uptime: 1000,
        cpu_usage: cpu,
        cpu_cores: 4,
        total_memory: 8_000_000_000,
        used_memory: mem,
        total_swap: 0,
        used_swap: 0,
        network_tx: tx,
        network_rx: rx,
        disks: vec![],
        processes: vec![],
    }
}

#[tokio::test]
async fn test_process_1m_averages_buffer() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    let (metrics_tx, metrics_rx) = broadcast::channel::<SystemMetrics>(100);
    let alert_tx = create_alert_channel();

    let mut worker = Worker::new(db.clone(), metrics_rx, 90.0, 90.0, None, alert_tx);

    worker.buffer.push(make_metrics(20.0, 1000, 100, 200));
    worker.buffer.push(make_metrics(40.0, 3000, 300, 400));

    worker.process_1m().await.unwrap();

    assert!(worker.buffer.is_empty());

    let history = db.get_history("metrics_1m", 0).await.unwrap();
    assert_eq!(history.len(), 1);
    assert!((history[0].cpu_usage - 30.0).abs() < 0.1);
    assert_eq!(history[0].used_memory, 2000);

    drop(metrics_tx);
}

#[tokio::test]
async fn test_process_1m_skips_empty_buffer() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    let (_tx, rx) = broadcast::channel::<SystemMetrics>(100);
    let alert_tx = create_alert_channel();

    let mut worker = Worker::new(db.clone(), rx, 90.0, 90.0, None, alert_tx);

    worker.process_1m().await.unwrap();
    let history = db.get_history("metrics_1m", 0).await.unwrap();
    assert_eq!(history.len(), 0);
}

#[tokio::test]
async fn test_aggregate_and_store() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    let (_tx, rx) = broadcast::channel::<SystemMetrics>(100);
    let alert_tx = create_alert_channel();

    let worker = Worker::new(db.clone(), rx, 90.0, 90.0, None, alert_tx);

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    for i in 0..3 {
        let point = MetricPoint {
            timestamp: now - 60 + i,
            cpu_usage: 50.0 + (i as f64 * 10.0),
            used_memory: 2000,
            network_tx: 100,
            network_rx: 200,
        };
        db.insert_metric("metrics_1m", point).await.unwrap();
    }

    worker.aggregate_and_store("metrics_1m", "metrics_5m", 5).await.unwrap();

    let history = db.get_history("metrics_5m", 0).await.unwrap();
    assert_eq!(history.len(), 1);
}

#[tokio::test]
async fn test_alert_not_triggered_below_threshold() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    let (_tx, rx) = broadcast::channel::<SystemMetrics>(100);
    let alert_tx = create_alert_channel();

    let mut worker = Worker::new(db.clone(), rx, 90.0, 90.0, None, alert_tx);

    for _ in 0..10 {
        worker.buffer.push(make_metrics(10.0, 100_000, 50, 50));
    }

    worker.process_1m().await.unwrap();
}
