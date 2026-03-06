export interface DiskInfo {
    name: string;
    mount_point: string;
    total_space: number;
    available_space: number;
}

export interface SystemMetrics {
    hostname: string;
    os_name: string;
    os_version: string;
    uptime: number;
    cpu_usage: number;
    cpu_cores: number;
    total_memory: number;
    used_memory: number;
    total_swap: number;
    used_swap: number;
    network_tx: number;
    network_rx: number;
    disks: DiskInfo[];
}
