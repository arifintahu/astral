use serde::Serialize;
use std::time::Instant;
use sysinfo::{
    CpuRefreshKind, Disks, MemoryRefreshKind, Networks, ProcessRefreshKind, ProcessesToUpdate,
    RefreshKind, System,
};
use tokio::sync::broadcast;
use tokio::time::{Duration, interval};

use crate::config::SharedConfig;

#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub uptime: u64,
    pub cpu_usage: f32,
    pub cpu_cores: usize,
    pub cpu_load: [f64; 3],
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub network_tx: u64,
    pub network_rx: u64,
    pub disks: Vec<DiskInfo>,
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub read_bytes: u64,
    pub written_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
}

/// How many processes the live feed carries.
const TOP_PROCESSES: usize = 15;

pub struct MetricsCollector {
    system: System,
    networks: Networks,
    disks: Disks,
    // These never change while Astral runs, so read them once instead of on every sample.
    hostname: String,
    os_name: String,
    os_version: String,
    last_collect: Instant,
}

/// The `n` items with the highest `key`, best first. Selects before sorting, so only `n` items
/// are ever sorted, however many there are.
fn top_by<T>(mut items: Vec<T>, n: usize, key: impl Fn(&T) -> f32) -> Vec<T> {
    let desc = |a: &T, b: &T| {
        key(b)
            .partial_cmp(&key(a))
            .unwrap_or(std::cmp::Ordering::Equal)
    };
    if items.len() > n {
        items.select_nth_unstable_by(n, desc);
        items.truncate(n);
    }
    items.sort_by(desc);
    items
}

/// A counter delta over `secs` seconds as a per-second rate. Samples are 1 s apart while someone
/// is watching and much further apart when not, and the dashboard always shows bytes per second.
fn per_second(delta: u64, secs: f64) -> u64 {
    (delta as f64 / secs.max(0.001)).round() as u64
}

impl MetricsCollector {
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();

        Self {
            system,
            networks,
            disks,
            hostname: System::host_name().unwrap_or_default(),
            os_name: System::name().unwrap_or_default(),
            os_version: System::os_version().unwrap_or_default(),
            last_collect: Instant::now(),
        }
    }

    pub fn collect(&mut self, enable_process_list: bool) -> SystemMetrics {
        let now = Instant::now();
        let secs = now.duration_since(self.last_collect).as_secs_f64();
        self.last_collect = now;

        // Not refresh_all(): that also walks every process, which is only needed for the list.
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.networks.refresh(true);
        self.disks.refresh(true);

        let cpu_usage = self.system.global_cpu_usage();
        let cpu_cores = self.system.cpus().len();
        let load = System::load_average();

        let mut network_tx = 0u64;
        let mut network_rx = 0u64;
        for (_, network) in &self.networks {
            network_tx += network.transmitted();
            network_rx += network.received();
        }

        let disks = self
            .disks
            .iter()
            .map(|disk| {
                let usage = disk.usage();
                DiskInfo {
                    name: disk.name().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    total_space: disk.total_space(),
                    available_space: disk.available_space(),
                    read_bytes: per_second(usage.read_bytes, secs),
                    written_bytes: per_second(usage.written_bytes, secs),
                }
            })
            .collect();

        let processes = if enable_process_list {
            self.top_processes()
        } else {
            vec![]
        };

        SystemMetrics {
            hostname: self.hostname.clone(),
            os_name: self.os_name.clone(),
            os_version: self.os_version.clone(),
            uptime: System::uptime(),
            cpu_usage,
            cpu_cores,
            cpu_load: [load.one, load.five, load.fifteen],
            total_memory: self.system.total_memory(),
            used_memory: self.system.used_memory(),
            total_swap: self.system.total_swap(),
            used_swap: self.system.used_swap(),
            network_tx: per_second(network_tx, secs),
            network_rx: per_second(network_rx, secs),
            disks,
            processes,
        }
    }

    fn top_processes(&mut self) -> Vec<ProcessInfo> {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_cpu().with_memory(),
        );
        let busiest = top_by(
            self.system.processes().iter().collect(),
            TOP_PROCESSES,
            |(_, p)| p.cpu_usage(),
        );
        // Names are copied only for the few that are sent.
        busiest
            .into_iter()
            .map(|(pid, p)| ProcessInfo {
                pid: pid.as_u32(),
                name: p.name().to_string_lossy().to_string(),
                cpu_usage: p.cpu_usage(),
                memory: p.memory(),
            })
            .collect()
    }
}

pub async fn run_metrics_collector(tx: broadcast::Sender<SystemMetrics>, config: SharedConfig) {
    let mut collector = MetricsCollector::new();
    let mut tick = interval(Duration::from_secs(1));

    loop {
        tick.tick().await;
        let enable = config.read().await.enable_process_list;
        let metrics = collector.collect(enable);
        let _ = tx.send(metrics);
    }
}

#[cfg(test)]
#[path = "metrics_test.rs"]
mod tests;
