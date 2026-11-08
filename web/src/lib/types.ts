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
    cpu_load: [number, number, number];
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

export interface MetricPoint {
    timestamp: number;
    cpu_usage: number;
    used_memory: number;
    network_tx: number;
    network_rx: number;
    disk_read_rate: number;
    disk_write_rate: number;
}

export interface DynamicConfig {
    enable_process_list: boolean;
    alert_cpu: number;
    alert_ram: number;
    retention_days: number;
    slack_webhook: string | null;
}

/* ── UI-only shapes (not serialised by the backend) ───────────────── */

/** Which metric the hero chart is showing; the four tiles are its tabs. */
export type MetricKey = 'cpu' | 'mem' | 'net' | 'disk';

/** 'live' is the client-side 60-sample buffer; the rest map to /api/history?window= */
export type WindowKey = 'live' | '6h' | '24h' | '7d' | 'all';

/** Rolling buffers fed from the SSE stream, one sample per refresh. */
export interface LiveSeries {
    cpu: number[]; // %
    mem: number[]; // % of total_memory
    rx: number[];  // bytes/s
    tx: number[];  // bytes/s
    rd: number[];  // bytes/s
    wr: number[];  // bytes/s
}

export interface TileData {
    id: MetricKey;
    label: string;
    meta: string;
    value: string;
    unit: string;
    sub: string;
    pct: number;
    zones: boolean;
    meterColor?: string;
}
