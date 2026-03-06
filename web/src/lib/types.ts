export interface DiskInfo {
    name: string;
    mount_point: string;
    total_space: number;
    available_space: number;
    read_bytes: number;
    written_bytes: number;
}

export interface ProcessInfo {
    pid: number;
    name: string;
    cpu_usage: number;
    memory: number;
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
    processes: ProcessInfo[];
}

export interface AlertEvent {
    kind: string;
    message: string;
    value: number;
    threshold: number;
    timestamp: number;
}
