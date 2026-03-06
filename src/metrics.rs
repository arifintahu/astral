use serde::Serialize;
use std::sync::{Arc, Mutex};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System};
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub uptime: u64,
    pub cpu_usage: f32,
    pub cpu_cores: usize,
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub network_tx: u64,
    pub network_rx: u64,
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
}

pub struct MetricsCollector {
    system: System,
    networks: Networks,
    disks: Disks,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        
        // Initial refresh
        system.refresh_all();

        Self {
            system,
            networks,
            disks,
        }
    }

    pub fn collect(&mut self) -> SystemMetrics {
        self.system.refresh_all();
        self.networks.refresh(true);
        self.disks.refresh(true);

        let cpu_usage = self.system.global_cpu_usage();
        let cpu_cores = self.system.cpus().len();
        
        let mut network_tx = 0;
        let mut network_rx = 0;
        for (_, network) in &self.networks {
            network_tx += network.transmitted();
            network_rx += network.received();
        }

        let disks = self.disks.iter().map(|disk| DiskInfo {
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
        }).collect();

        SystemMetrics {
            hostname: System::host_name().unwrap_or_default(),
            os_name: System::name().unwrap_or_default(),
            os_version: System::os_version().unwrap_or_default(),
            uptime: System::uptime(),
            cpu_usage,
            cpu_cores,
            total_memory: self.system.total_memory(),
            used_memory: self.system.used_memory(),
            total_swap: self.system.total_swap(),
            used_swap: self.system.used_swap(),
            network_tx,
            network_rx,
            disks,
        }
    }
}

pub async fn run_metrics_collector(
    tx: broadcast::Sender<SystemMetrics>,
) {
    let mut collector = MetricsCollector::new();
    let mut interval = interval(Duration::from_secs(1));

    loop {
        interval.tick().await;
        let metrics = collector.collect();
        // Ignore error if no active subscribers
        let _ = tx.send(metrics);
    }
}
