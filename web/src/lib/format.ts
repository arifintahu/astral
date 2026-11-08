const KB = 1024;
const MB = KB * 1024;
const GB = MB * 1024;
const TB = GB * 1024;

export function fmtBytes(b: number): string {
    if (b >= TB) return (b / TB).toFixed(1) + ' TB';
    if (b >= GB) return (b / GB).toFixed(1) + ' GB';
    if (b >= MB) return Math.round(b / MB) + ' MB';
    if (b >= KB) return Math.round(b / KB) + ' KB';
    return Math.round(b) + ' B';
}

export function rateParts(b: number): { v: string; u: string } {
    if (b >= MB) return { v: (b / MB).toFixed(1), u: 'MB/s' };
    if (b >= KB) return { v: String(Math.round(b / KB)), u: 'KB/s' };
    return { v: String(Math.round(b)), u: 'B/s' };
}

export function fmtRate(b: number): string {
    const p = rateParts(b);
    return `${p.v} ${p.u}`;
}

/** Axis-friendly: 1.5M, 400K, 12 */
export function fmtRateShort(b: number): string {
    if (b >= MB) return +(b / MB).toFixed(1) + 'M';
    if (b >= KB) return Math.round(b / KB) + 'K';
    return String(Math.round(b));
}

export function fmtPct(v: number): string {
    return `${v < 10 ? v.toFixed(1) : Math.round(v)}%`;
}

export function fmtUptime(s: number): string {
    const d = Math.floor(s / 86400);
    const h = Math.floor((s % 86400) / 3600);
    const m = Math.floor((s % 3600) / 60);
    const parts: string[] = [];
    if (d > 0) parts.push(`${d}d`);
    parts.push(`${h}h`, `${m}m`);
    return parts.join(' ');
}

/** Severity colour for a 0–100 usage value. */
export function zoneColor(p: number): string {
    if (p >= 90) return 'var(--crit)';
    if (p >= 70) return 'var(--warm)';
    return 'var(--accent)';
}

/** Round the chart ceiling up to a friendly number so the axis reads cleanly. */
export function niceMaxPct(peak: number): number {
    const t = peak * 1.25;
    for (const s of [10, 20, 25, 40, 50, 60, 80, 100]) if (s >= t) return s;
    return 100;
}

export function niceMaxRate(peak: number): number {
    if (peak <= 0) return KB;
    const t = peak * 1.2;
    const mag = Math.pow(10, Math.floor(Math.log10(t)));
    for (const s of [1, 1.5, 2, 2.5, 4, 5, 8, 10]) if (s * mag >= t) return s * mag;
    return 10 * mag;
}

/** Catmull-Rom → cubic Bézier. Gives the chart its fluid, "alive" line. */
export function smoothPath(p: { x: number; y: number }[]): string {
    if (!p.length) return '';
    let d = `M${p[0].x.toFixed(1)},${p[0].y.toFixed(1)}`;
    for (let i = 0; i < p.length - 1; i++) {
        const p0 = p[i - 1] ?? p[i];
        const p1 = p[i];
        const p2 = p[i + 1];
        const p3 = p[i + 2] ?? p2;
        const c1x = p1.x + (p2.x - p0.x) / 6;
        const c1y = p1.y + (p2.y - p0.y) / 6;
        const c2x = p2.x - (p3.x - p1.x) / 6;
        const c2y = p2.y - (p3.y - p1.y) / 6;
        d += ` C${c1x.toFixed(1)},${c1y.toFixed(1)} ${c2x.toFixed(1)},${c2y.toFixed(1)} ${p2.x.toFixed(1)},${p2.y.toFixed(1)}`;
    }
    return d;
}

/** Keep long history windows light: stride-sample down to ~cap points, always keeping the last. */
export function thin<T>(arr: T[], cap = 180): T[] {
    if (arr.length <= cap) return arr;
    const step = Math.ceil(arr.length / cap);
    const out = arr.filter((_, i) => i % step === 0);
    if (out[out.length - 1] !== arr[arr.length - 1]) out.push(arr[arr.length - 1]);
    return out;
}
