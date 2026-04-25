use super::*;

#[test]
fn test_validate_table_accepts_valid_names() {
    assert_eq!(validate_table("metrics_1m").unwrap(), "metrics_1m");
    assert_eq!(validate_table("metrics_5m").unwrap(), "metrics_5m");
    assert_eq!(validate_table("metrics_15m").unwrap(), "metrics_15m");
    assert_eq!(validate_table("metrics_1h").unwrap(), "metrics_1h");
}

#[test]
fn test_validate_table_rejects_invalid_names() {
    assert!(validate_table("").is_err());
    assert!(validate_table("users").is_err());
    assert!(validate_table("metrics_2m").is_err());
    assert!(validate_table("metrics_1m; DROP TABLE metrics_1m").is_err());
    assert!(validate_table("metrics_1m' OR '1'='1").is_err());
}

fn make_point(ts: i64, cpu: f64) -> MetricPoint {
    MetricPoint {
        timestamp: ts,
        cpu_usage: cpu,
        used_memory: 1024,
        network_tx: 100,
        network_rx: 200,
    }
}

#[tokio::test]
async fn test_db_init_creates_tables() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    // Inserting into all tables should work
    for table in &["metrics_1m", "metrics_5m", "metrics_15m", "metrics_1h"] {
        db.insert_metric(table, make_point(1000, 50.0)).await.unwrap();
    }
}

#[tokio::test]
async fn test_insert_and_get_history() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    db.insert_metric("metrics_1m", make_point(1000, 50.0)).await.unwrap();
    db.insert_metric("metrics_1m", make_point(1001, 60.0)).await.unwrap();

    let history = db.get_history("metrics_1m", 0).await.unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].timestamp, 1000);
    assert_eq!(history[1].timestamp, 1001);
    assert!((history[0].cpu_usage - 50.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_get_history_respects_start_ts() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    db.insert_metric("metrics_1m", make_point(100, 10.0)).await.unwrap();
    db.insert_metric("metrics_1m", make_point(200, 20.0)).await.unwrap();
    db.insert_metric("metrics_1m", make_point(300, 30.0)).await.unwrap();

    let history = db.get_history("metrics_1m", 200).await.unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].timestamp, 200);
}

#[tokio::test]
async fn test_insert_invalid_table_fails() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    assert!(db.insert_metric("hacked", make_point(1, 1.0)).await.is_err());
}

#[tokio::test]
async fn test_get_history_invalid_table_fails() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    assert!(db.get_history("hacked", 0).await.is_err());
}

#[tokio::test]
async fn test_cleanup_removes_old_data() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    // Insert with timestamp = 1 (very old)
    db.insert_metric("metrics_1m", make_point(1, 50.0)).await.unwrap();
    // Cleanup with 1 second retention (cutoff = now - 1, which is >> 1)
    db.cleanup("metrics_1m", 1).await.unwrap();
    let history = db.get_history("metrics_1m", 0).await.unwrap();
    assert_eq!(history.len(), 0);
}

#[tokio::test]
async fn test_get_average() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    db.insert_metric("metrics_1m", make_point(100, 20.0)).await.unwrap();
    db.insert_metric("metrics_1m", make_point(101, 40.0)).await.unwrap();
    db.insert_metric("metrics_1m", make_point(102, 60.0)).await.unwrap();

    let avg = db.get_average("metrics_1m", 0).await.unwrap().unwrap();
    assert!((avg.cpu_usage - 40.0).abs() < 0.1);
    assert_eq!(avg.used_memory, 1024);
}

#[tokio::test]
async fn test_get_average_empty() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    let avg = db.get_average("metrics_1m", 0).await.unwrap();
    assert!(avg.is_none());
}

#[tokio::test]
async fn test_insert_or_replace_overwrites() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    db.insert_metric("metrics_1m", make_point(100, 20.0)).await.unwrap();
    db.insert_metric("metrics_1m", make_point(100, 80.0)).await.unwrap();

    let history = db.get_history("metrics_1m", 0).await.unwrap();
    assert_eq!(history.len(), 1);
    assert!((history[0].cpu_usage - 80.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_check_alert_condition_no_data() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    let result = db.check_alert_condition("metrics_1m", 90.0, 2048, 300).await.unwrap();
    assert!(!result);
}
