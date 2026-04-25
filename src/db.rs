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
    pub disk_read_rate: i64,
    pub disk_write_rate: i64,
}

impl Db {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        let db = Self { pool };
        db.init().await?;
        Ok(db)
    }

    async fn init(&self) -> Result<()> {
        sqlx::query("PRAGMA journal_mode=WAL").execute(&self.pool).await?;

        let tables = ["metrics_1m", "metrics_5m", "metrics_15m", "metrics_1h"];
        for table in tables {
            let table = validate_table(table)?;
            sqlx::query(&format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    timestamp INTEGER PRIMARY KEY,
                    cpu_usage REAL,
                    used_memory INTEGER,
                    network_tx INTEGER,
                    network_rx INTEGER,
                    disk_read_rate INTEGER DEFAULT 0,
                    disk_write_rate INTEGER DEFAULT 0
                )",
                table
            ))
            .execute(&self.pool)
            .await?;

            // Idempotent migration: add new columns to existing tables if missing.
            let _ = sqlx::query(&format!(
                "ALTER TABLE {} ADD COLUMN disk_read_rate INTEGER DEFAULT 0",
                table
            ))
            .execute(&self.pool)
            .await;
            let _ = sqlx::query(&format!(
                "ALTER TABLE {} ADD COLUMN disk_write_rate INTEGER DEFAULT 0",
                table
            ))
            .execute(&self.pool)
            .await;
        }
        Ok(())
    }

    pub async fn insert_metric(&self, table: &str, metric: MetricPoint) -> Result<()> {
        let table = validate_table(table)?;
        let query = format!(
            "INSERT OR REPLACE INTO {} \
             (timestamp, cpu_usage, used_memory, network_tx, network_rx, disk_read_rate, disk_write_rate) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            table
        );
        sqlx::query(&query)
            .bind(metric.timestamp)
            .bind(metric.cpu_usage)
            .bind(metric.used_memory)
            .bind(metric.network_tx)
            .bind(metric.network_rx)
            .bind(metric.disk_read_rate)
            .bind(metric.disk_write_rate)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_history(&self, table: &str, start_ts: i64) -> Result<Vec<MetricPoint>> {
        let table = validate_table(table)?;
        let query = format!(
            "SELECT timestamp, cpu_usage, used_memory, network_tx, network_rx, \
             COALESCE(disk_read_rate, 0) as disk_read_rate, \
             COALESCE(disk_write_rate, 0) as disk_write_rate \
             FROM {} WHERE timestamp >= ? ORDER BY timestamp ASC",
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
                CAST(AVG(network_rx) as INTEGER) as network_rx,
                CAST(AVG(COALESCE(disk_read_rate, 0)) as INTEGER) as disk_read_rate,
                CAST(AVG(COALESCE(disk_write_rate, 0)) as INTEGER) as disk_write_rate
            FROM {} WHERE timestamp >= ?",
            table
        );

        let row: (Option<f64>, Option<i64>, Option<i64>, Option<i64>, Option<i64>, Option<i64>) =
            sqlx::query_as(&query)
                .bind(start_ts)
                .fetch_one(&self.pool)
                .await?;

        if let (Some(cpu), Some(mem), Some(tx), Some(rx), Some(dr), Some(dw)) = row {
            Ok(Some(MetricPoint {
                timestamp: start_ts,
                cpu_usage: cpu,
                used_memory: mem,
                network_tx: tx,
                network_rx: rx,
                disk_read_rate: dr,
                disk_write_rate: dw,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn check_alert_condition(
        &self,
        table: &str,
        threshold_cpu: f64,
        threshold_mem: i64,
        duration_seconds: i64,
    ) -> Result<bool> {
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
#[path = "db_test.rs"]
mod tests;
