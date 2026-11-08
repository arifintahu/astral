import type { SystemMetrics } from './types';

/** Alert thresholds in percent, from DynamicConfig.alert_cpu / alert_ram (GET /api/settings). */
export interface Thresholds {
    cpu: number;
    ram: number;
}

export const DEFAULT_THRESHOLDS: Thresholds = { cpu: 90, ram: 90 };

/** The backend has no disk alert; keep the old fixed line. */
const DISK_CRIT = 90;
/** "Running warm" starts this many points below the alert line (70% for a 90% alert). */
const WARM_MARGIN = 20;

export interface Status {
    level: 0 | 1 | 2;
    headline: string;
    sub: string;
    color: string;
}

export function diskPercent(m: SystemMetrics): number {
    const d = m.disks[0];
    if (!d || d.total_space <= 0) return 0;
    return ((d.total_space - d.available_space) / d.total_space) * 100;
}

export function memPercent(m: SystemMetrics): number {
    return m.total_memory > 0 ? (m.used_memory / m.total_memory) * 100 : 0;
}

/** One human sentence about the host, driven by whichever of CPU / memory / disk is closest to its alert line. */
export function deriveStatus(m: SystemMetrics, t: Thresholds = DEFAULT_THRESHOLDS): Status {
    const rank = (v: number, crit: number) => (v >= crit ? 2 : v >= crit - WARM_MARGIN ? 1 : 0);
    const items = [
        { k: 'CPU', v: m.cpu_usage, crit: t.cpu },
        { k: 'Memory', v: memPercent(m), crit: t.ram },
        { k: 'Disk', v: diskPercent(m), crit: DISK_CRIT },
    ].sort((a, b) => rank(b.v, b.crit) - rank(a.v, a.crit) || b.v - a.v);
    const worst = items[0];
    const v = Math.round(worst.v);
    const crit = Math.round(worst.crit);

    if (worst.v >= worst.crit) {
        const byMem = worst.k === 'Memory';
        const top = worst.k === 'Disk'
            ? undefined
            : [...m.processes].sort((a, b) => (byMem ? b.memory - a.memory : b.cpu_usage - a.cpu_usage))[0];
        return {
            level: 2,
            headline: worst.k === 'Disk' ? 'Disk is almost full.' : `${worst.k} is running hot.`,
            sub: `${worst.k} is at ${v}% — above the ${crit}% alert threshold.` + (top ? ` The busiest process is ${top.name}.` : ''),
            color: 'var(--crit)',
        };
    }
    if (worst.v >= worst.crit - WARM_MARGIN) {
        return {
            level: 1,
            headline: `${worst.k} is running warm.`,
            sub: `${worst.k} is at ${v}% — worth keeping an eye on before it reaches the ${crit}% alert line.`,
            color: 'var(--warm)',
        };
    }
    return {
        level: 0,
        headline: 'All systems nominal.',
        sub: 'CPU, memory and disk are well within limits.',
        color: 'var(--accent)',
    };
}
