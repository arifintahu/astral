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
        disk_read_rate: 0,
        disk_write_rate: 0,
    }
}

#[tokio::test]
async fn test_db_init_creates_tables() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    // Inserting into all tables should work
    for table in &["metrics_1m", "metrics_5m", "metrics_15m", "metrics_1h"] {
        db.insert_metric(table, make_point(1000, 50.0))
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn test_insert_and_get_history() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    db.insert_metric("metrics_1m", make_point(1000, 50.0))
        .await
        .unwrap();
    db.insert_metric("metrics_1m", make_point(1001, 60.0))
        .await
        .unwrap();

    let history = db.get_history("metrics_1m", 0).await.unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].timestamp, 1000);
    assert_eq!(history[1].timestamp, 1001);
    assert!((history[0].cpu_usage - 50.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_get_history_respects_start_ts() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    db.insert_metric("metrics_1m", make_point(100, 10.0))
        .await
        .unwrap();
    db.insert_metric("metrics_1m", make_point(200, 20.0))
        .await
        .unwrap();
    db.insert_metric("metrics_1m", make_point(300, 30.0))
        .await
        .unwrap();

    let history = db.get_history("metrics_1m", 200).await.unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].timestamp, 200);
}

#[tokio::test]
async fn test_insert_invalid_table_fails() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    assert!(
        db.insert_metric("hacked", make_point(1, 1.0))
            .await
            .is_err()
    );
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
    db.insert_metric("metrics_1m", make_point(1, 50.0))
        .await
        .unwrap();
    // Cleanup with 1 second retention (cutoff = now - 1, which is >> 1)
    db.cleanup("metrics_1m", 1).await.unwrap();
    let history = db.get_history("metrics_1m", 0).await.unwrap();
    assert_eq!(history.len(), 0);
}

#[tokio::test]
async fn test_get_average() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    db.insert_metric("metrics_1m", make_point(100, 20.0))
        .await
        .unwrap();
    db.insert_metric("metrics_1m", make_point(101, 40.0))
        .await
        .unwrap();
    db.insert_metric("metrics_1m", make_point(102, 60.0))
        .await
        .unwrap();

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
    db.insert_metric("metrics_1m", make_point(100, 20.0))
        .await
        .unwrap();
    db.insert_metric("metrics_1m", make_point(100, 80.0))
        .await
        .unwrap();

    let history = db.get_history("metrics_1m", 0).await.unwrap();
    assert_eq!(history.len(), 1);
    assert!((history[0].cpu_usage - 80.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_check_alert_condition_no_data() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    let result = db
        .check_alert_condition("metrics_1m", 90.0, 2048, 300)
        .await
        .unwrap();
    assert!(!result);
}

fn temp_db_path(tag: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("astral-{tag}-{}-{nanos}.db", std::process::id()))
}

fn remove_db_files(path: &std::path::Path) {
    for suffix in ["", "-wal", "-shm"] {
        let mut name = path.as_os_str().to_owned();
        name.push(suffix);
        let _ = std::fs::remove_file(name);
    }
}

#[tokio::test]
async fn test_wal_file_stays_small() {
    // One row a minute for days is thousands of tiny transactions. SQLite's defaults let the
    // write-ahead log grow to about 4 MB and stay that size, 40 times the data on a small server.
    let path = temp_db_path("wal");
    let db = Db::new(&format!("sqlite:{}?mode=rwc", path.display()))
        .await
        .unwrap();
    for i in 0..3000 {
        db.insert_metric("metrics_1m", make_point(1000 + i, 50.0))
            .await
            .unwrap();
    }

    let wal = path.with_file_name(format!(
        "{}-wal",
        path.file_name().unwrap().to_string_lossy()
    ));
    let wal_bytes = std::fs::metadata(&wal).map(|m| m.len()).unwrap_or(0);
    drop(db);
    remove_db_files(&path);

    assert!(
        wal_bytes < 1_500_000,
        "write-ahead log grew to {wal_bytes} bytes"
    );
}

#[tokio::test]
async fn test_wal_settings_apply_to_every_pooled_connection() {
    // Pragmas such as journal_size_limit belong to a connection, and the pool opens two.
    let path = temp_db_path("pragma");
    let db = Db::new(&format!("sqlite:{}?mode=rwc", path.display()))
        .await
        .unwrap();

    let mut first = db.pool.acquire().await.unwrap();
    let mut second = db.pool.acquire().await.unwrap();
    for conn in [&mut first, &mut second] {
        let limit: i64 = sqlx::query_scalar("PRAGMA journal_size_limit")
            .fetch_one(&mut **conn)
            .await
            .unwrap();
        assert_eq!(limit, 1_048_576);
        let mode: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&mut **conn)
            .await
            .unwrap();
        assert_eq!(mode, "wal");
    }
    drop((first, second));
    drop(db);
    remove_db_files(&path);
}
