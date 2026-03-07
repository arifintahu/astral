use anyhow::Result;
use sqlx::{sqlite::SqlitePool, FromRow};
use std::time::{SystemTime, UNIX_EPOCH};

const VALID_TABLES: &[&str] = &["metrics_1m", "metrics_5m", "metrics_15m", "metrics_1h"];

fn validate_table<'a>(table: &'a str) -> Result<&'a str> {
    if VALID_TABLES.contains(&table) {
        Ok(table)
    } else {
        anyhow::bail!("Invalid table name")
    }
}

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

#[derive(Debug, FromRow, serde::Serialize)]
pub struct MetricPoint {
    pub timestamp: i64,
    pub cpu_usage: f64,
    pub used_memory: i64,
    pub network_tx: i64,
    pub network_rx: i64,
}

impl Db {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        let db = Self { pool };
        db.init().await?;
        Ok(db)
    }

    async fn init(&self) -> Result<()> {
        let tables = ["metrics_1m", "metrics_5m", "metrics_15m", "metrics_1h"];
        for table in tables {
            let table = validate_table(table)?;
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    timestamp INTEGER PRIMARY KEY,
                    cpu_usage REAL,
                    used_memory INTEGER,
                    network_tx INTEGER,
                    network_rx INTEGER
                )",
                table
            ))
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn insert_metric(&self, table: &str, metric: MetricPoint) -> Result<()> {
        let table = validate_table(table)?;
        let query = format!(
            "INSERT OR REPLACE INTO {} (timestamp, cpu_usage, used_memory, network_tx, network_rx) VALUES (?, ?, ?, ?, ?)",
            table
        );
        sqlx::query(&query)
            .bind(metric.timestamp)
            .bind(metric.cpu_usage)
            .bind(metric.used_memory)
            .bind(metric.network_tx)
            .bind(metric.network_rx)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_history(&self, table: &str, start_ts: i64) -> Result<Vec<MetricPoint>> {
        let table = validate_table(table)?;
        let query = format!(
            "SELECT timestamp, cpu_usage, used_memory, network_tx, network_rx FROM {} WHERE timestamp >= ? ORDER BY timestamp ASC",
            table
        );
        let rows = sqlx::query_as::<_, MetricPoint>(&query)
            .bind(start_ts)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    pub async fn cleanup(&self, table: &str, retention_seconds: i64) -> Result<()> {
        let table = validate_table(table)?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let cutoff = now - retention_seconds;
        let query = format!("DELETE FROM {} WHERE timestamp < ?", table);
        sqlx::query(&query).bind(cutoff).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_average(&self, table: &str, start_ts: i64) -> Result<Option<MetricPoint>> {
        let table = validate_table(table)?;
        let query = format!(
            "SELECT 
                AVG(cpu_usage) as cpu_usage, 
                CAST(AVG(used_memory) as INTEGER) as used_memory, 
                CAST(AVG(network_tx) as INTEGER) as network_tx, 
                CAST(AVG(network_rx) as INTEGER) as network_rx 
            FROM {} WHERE timestamp >= ?",
            table
        );
        
        let row: (Option<f64>, Option<i64>, Option<i64>, Option<i64>) = sqlx::query_as(&query)
            .bind(start_ts)
            .fetch_one(&self.pool)
            .await?;
            
        if let (Some(cpu), Some(mem), Some(tx), Some(rx)) = row {
             Ok(Some(MetricPoint {
                 timestamp: start_ts, // This timestamp marks the start of the window
                 cpu_usage: cpu,
                 used_memory: mem,
                 network_tx: tx,
                 network_rx: rx,
             }))
        } else {
            Ok(None)
        }
    }

    pub async fn check_alert_condition(&self, table: &str, threshold_cpu: f64, threshold_mem: i64, duration_seconds: i64) -> Result<bool> {
        let table = validate_table(table)?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let start_ts = now - duration_seconds;
        
        let query = format!(
            "SELECT 
                COUNT(*) as total,
                COALESCE(SUM(CASE WHEN cpu_usage > ? THEN 1 ELSE 0 END), 0) as high_cpu,
                COALESCE(SUM(CASE WHEN used_memory > ? THEN 1 ELSE 0 END), 0) as high_mem
            FROM {} WHERE timestamp >= ?",
            table
        );

        let row: (i64, i64, i64) = sqlx::query_as(&query)
            .bind(threshold_cpu)
            .bind(threshold_mem)
            .bind(start_ts)
            .fetch_one(&self.pool)
            .await?;
            
        let (total, high_cpu, high_mem) = row;
        
        if total == 0 {
            return Ok(false);
        }
        
        if total >= 5 && (high_cpu == total || high_mem == total) {
            return Ok(true);
        }
        
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
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
}
