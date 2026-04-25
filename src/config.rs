use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicConfig {
    pub enable_process_list: bool,
    pub alert_cpu: f32,
    pub alert_ram: f32,
    pub retention_days: u64,
    pub slack_webhook: Option<String>,
}

pub type SharedConfig = Arc<RwLock<DynamicConfig>>;
