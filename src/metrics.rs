use serde::Serialize;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, ProcessesToUpdate, RefreshKind, System};
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

pub struct MetricsCollector {
    system: System,
    networks: Networks,
    disks: Disks,
}

impl MetricsCollector {
    pub fn new() -> Self {
        // sysinfo checks HOST_PROC and HOST_SYS environment variables automatically
        // but we need to ensure they are set before System::new() if they are passed in differently
        // In our case, we set them in docker-compose command, so they should be available.
        // However, sysinfo might not respect them unless we use System::new_with_specifics properly or if the library version supports it.
        // sysinfo 0.30+ respects HOST_PROC/HOST_SYS env vars on Linux.
        
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

    pub fn collect(&mut self, enable_process_list: bool) -> SystemMetrics {
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

        let disks = self.disks.iter().map(|disk| {
            let usage = disk.usage();
            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
                read_bytes: usage.read_bytes,
                written_bytes: usage.written_bytes,
            }
        }).collect();

        // M-5: Process list is opt-in — collect only when --enable-process-list is set.
        let procs = if enable_process_list {
            self.system.refresh_processes(ProcessesToUpdate::All, true);
            let mut p: Vec<ProcessInfo> = self.system.processes().iter().map(|(pid, proc_)| {
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: proc_.name().to_string_lossy().to_string(),
                    cpu_usage: proc_.cpu_usage(),
                    memory: proc_.memory(),
                }
            }).collect();
            p.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
            p.truncate(15);
            p
        } else {
            vec![]
        };

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
            processes: procs,
        }
    }
}

pub async fn run_metrics_collector(
    tx: broadcast::Sender<SystemMetrics>,
    enable_process_list: bool,
) {
    let mut collector = MetricsCollector::new();
    let mut interval = interval(Duration::from_secs(1));

    loop {
        interval.tick().await;
        let metrics = collector.collect(enable_process_list);
        let _ = tx.send(metrics);
    }
}
