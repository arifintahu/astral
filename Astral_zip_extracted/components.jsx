// components.jsx — Top bar + 4 metric cards + history chart + process list
// Exposes components on window for cross-script Babel scope.

const { useEffect, useMemo, useRef, useState } = React;

// ─── Formatters ─────────────────────────────────────────────────────────────
function formatBytes(bytes, fixed = 1) {
  if (!bytes || bytes <= 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), sizes.length - 1);
  const v = bytes / Math.pow(k, i);
  return `${v.toFixed(v >= 100 ? 0 : v >= 10 ? 1 : fixed)} ${sizes[i]}`;
}
function formatRate(bytes) { return `${formatBytes(bytes)}/s`; }
function formatUptime(seconds) {
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  return [d ? `${d}d` : null, `${h}h`, `${m}m`].filter(Boolean).join(" ");
}
function formatRelative(ts) {
  const dt = (Date.now() - ts) / 1000;
  if (dt < 60) return `${Math.floor(dt)}s ago`;
  if (dt < 3600) return `${Math.floor(dt / 60)}m ago`;
  if (dt < 86400) return `${Math.floor(dt / 3600)}h ago`;
  return `${Math.floor(dt / 86400)}d ago`;
}

// ─── Severity color helpers ────────────────────────────────────────────────
function severity(v, warn = 70, crit = 90) {
  if (v >= crit) return "crit";
  if (v >= warn) return "warn";
  return "ok";
}
function sevColor(sev) {
  return sev === "crit" ? "var(--crit)" : sev === "warn" ? "var(--warm)" : "var(--accent)";
}

// ─── Mini chart variants (tweak: sparkline / bar / dot / radial) ───────────
function MiniChart({ data, variant, w = 140, h = 48, max = 100, color }) {
  if (!data || data.length < 2) return <div style={{ width: w, height: h }} />;
  const stroke = color || "var(--accent)";

  if (variant === "bar") {
    const step = w / data.length;
    return (
      <svg width="100%" height={h} viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="none">
        {data.map((v, i) => {
          const bh = Math.max(1.5, (v / max) * (h - 3));
          return (
            <rect key={i} x={i * step + 0.5} y={h - bh} width={step - 1.2} height={bh}
                  fill={stroke} opacity={0.18 + 0.6 * (v / max)} rx={1} />
          );
        })}
      </svg>
    );
  }

  if (variant === "dot") {
    const step = w / (data.length - 1);
    return (
      <svg width="100%" height={h} viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="none">
        {data.map((v, i) => {
          const x = i * step;
          const y = h - (v / max) * h * 0.92 - 2;
          return <circle key={i} cx={x} cy={y} r={1.4} fill={stroke} opacity={0.35 + 0.6 * (v / max)} />;
        })}
      </svg>
    );
  }

  if (variant === "radial") {
    // Last 24 cells as radial bars from center
    const slice = data.slice(-24);
    const cx = w / 2, cy = h / 2, r0 = 6, rMax = Math.min(w, h) / 2 - 2;
    return (
      <svg width="100%" height={h} viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="xMidYMid meet">
        {slice.map((v, i) => {
          const a = (i / slice.length) * Math.PI * 2 - Math.PI / 2;
          const r = r0 + (v / max) * (rMax - r0);
          const x1 = cx + Math.cos(a) * r0;
          const y1 = cy + Math.sin(a) * r0;
          const x2 = cx + Math.cos(a) * r;
          const y2 = cy + Math.sin(a) * r;
          return <line key={i} x1={x1} y1={y1} x2={x2} y2={y2} stroke={stroke} strokeWidth={1.6}
                       strokeLinecap="round" opacity={0.3 + 0.6 * (v / max)} />;
        })}
      </svg>
    );
  }

  // sparkline (default)
  const step = w / (data.length - 1);
  const pts = data.map((v, i) => [i * step, h - (v / max) * h * 0.92 - 2]);
  const pathD = `M ${pts.map(p => p.join(",")).join(" L ")}`;
  const areaD = `M 0,${h} L ${pts.map(p => p.join(",")).join(" L ")} L ${w},${h} Z`;
  const last = pts[pts.length - 1];
  return (
    <svg width="100%" height={h} viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="none" style={{ overflow: "visible" }}>
      <defs>
        <linearGradient id={`mini-grad-${stroke.replace(/[^a-z0-9]/gi, "")}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={stroke} stopOpacity="0.22" />
          <stop offset="100%" stopColor={stroke} stopOpacity="0" />
        </linearGradient>
      </defs>
      <path d={areaD} fill={`url(#mini-grad-${stroke.replace(/[^a-z0-9]/gi, "")})`} />
      <path d={pathD} fill="none" stroke={stroke} strokeWidth="1.5" vectorEffect="non-scaling-stroke" strokeLinejoin="round" />
      <circle cx={last[0]} cy={last[1]} r="2.5" fill={stroke} />
    </svg>
  );
}

// ─── TopBar ─────────────────────────────────────────────────────────────────
function TopBar({ metrics, alerts, refreshRate, onShowAlerts, onShowSettings, onLogout, lastUpdate }) {
  const active = alerts.length > 0;
  return (
    <header className="fade-up" style={{ marginBottom: 22 }}>
      <div className="flex items-center justify-between gap-6 flex-wrap">
        <div className="flex items-center gap-3">
          <Sigil />
          <div>
            <div className="flex items-baseline gap-2">
              <span className="text-[15px] font-semibold tracking-tight" style={{ color: "var(--ink)" }}>Astral</span>
              <span className="text-[11px] tnum font-mono" style={{ color: "var(--ink-4)" }}>v1.1.0</span>
            </div>
            <div className="flex items-center gap-2 mt-0.5 text-[12px]" style={{ color: "var(--ink-3)" }}>
              <span className="font-mono" style={{ color: "var(--ink-2)" }}>{metrics ? metrics.hostname : "—"}</span>
              {metrics && <span style={{ color: "var(--ink-4)" }}>·</span>}
              {metrics && <span>{metrics.os_name} {metrics.os_version}</span>}
              <span style={{ color: "var(--ink-4)" }}>·</span>
              <span>fra1 · edge</span>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {/* Status pill */}
          {active ? (
            <button className="btn" onClick={onShowAlerts}
              style={{ borderColor: "var(--crit-soft)", color: "var(--crit)", background: "var(--crit-soft)" }}>
              <span className="live-dot inline-block w-1.5 h-1.5 rounded-full" style={{ background: "var(--crit)", color: "var(--crit)" }}></span>
              <span className="font-medium">{alerts.length} alert{alerts.length === 1 ? "" : "s"}</span>
            </button>
          ) : (
            <div className="btn" style={{ background: "transparent", borderColor: "var(--line)" }}>
              <span className="live-dot inline-block w-1.5 h-1.5 rounded-full" style={{ background: "var(--ok)", color: "var(--ok)" }}></span>
              <span className="tnum">Live · {refreshRate}s</span>
            </div>
          )}

          {/* Uptime — visible meta */}
          {metrics && (
            <div className="btn" style={{ background: "transparent", borderColor: "var(--line)" }}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" style={{ color: "var(--ink-3)" }}>
                <circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" strokeLinecap="round" />
              </svg>
              <span className="tnum font-mono">{formatUptime(metrics.uptime)}</span>
            </div>
          )}

          <button className="btn btn-ghost" title="Alert history" onClick={onShowAlerts}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8">
              <path d="M18 8a6 6 0 1 0-12 0c0 7-3 8-3 8h18s-3-1-3-8" strokeLinejoin="round" />
              <path d="M13.7 21a2 2 0 0 1-3.4 0" strokeLinecap="round" />
            </svg>
          </button>
          <button className="btn btn-ghost" title="Settings" onClick={onShowSettings}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8">
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1A1.7 1.7 0 0 0 4.6 9a1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z" />
            </svg>
          </button>
          <button className="btn btn-ghost" title="Sign out" onClick={onLogout}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
              <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
              <polyline points="16 17 21 12 16 7" />
              <line x1="21" y1="12" x2="9" y2="12" />
            </svg>
          </button>
        </div>
      </div>
    </header>
  );
}

function Sigil() {
  return (
    <div className="relative flex items-center justify-center" style={{ width: 36, height: 36, borderRadius: 10, background: "var(--bg-2)", border: "1px solid var(--line-2)" }}>
      <svg width="22" height="22" viewBox="0 0 24 24" fill="none" style={{ color: "var(--accent)" }}>
        {/* Concentric arcs evoking an orbit */}
        <circle cx="12" cy="12" r="2" fill="currentColor" />
        <path d="M12 4.5a7.5 7.5 0 0 1 7.5 7.5" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" />
        <path d="M12 19.5a7.5 7.5 0 0 1-7.5-7.5" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" opacity="0.65" />
        <path d="M18 12a6 6 0 0 1-3 5.2" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" opacity="0.35" />
      </svg>
    </div>
  );
}

// ─── Card shell ─────────────────────────────────────────────────────────────
function MetricCard({ label, accent = "var(--accent)", children, badge, density }) {
  const pad = density === "compact" ? 16 : density === "comfy" ? 24 : 20;
  return (
    <div className="surface card-hover relative overflow-hidden" style={{ padding: pad, minHeight: density === "compact" ? 132 : 156 }}>
      {/* top hairline accent */}
      <div style={{ position: "absolute", left: pad, right: pad, top: 0, height: 1, background: "linear-gradient(90deg, transparent 0%, " + accent + " 12%, " + accent + " 88%, transparent 100%)", opacity: 0.4 }} />
      <div className="flex items-center justify-between mb-3">
        <div className="eyebrow">{label}</div>
        {badge && <div className="text-[11px] font-mono tnum" style={{ color: "var(--ink-3)" }}>{badge}</div>}
      </div>
      {children}
    </div>
  );
}

// ─── CPU Card ──────────────────────────────────────────────────────────────
function CpuCard({ metrics, history, chartStyle, density }) {
  const usage = metrics ? metrics.cpu_usage : 0;
  const sev = severity(usage);
  const accent = sevColor(sev);

  // Mini radial ring
  const r = 26, C = 2 * Math.PI * r;
  const offset = C - (usage / 100) * C;

  return (
    <MetricCard label="CPU" accent={accent} badge={`${metrics ? metrics.cpu_cores : "—"} cores`} density={density}>
      <div className="flex items-end justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="relative" style={{ width: 64, height: 64 }}>
            <svg width="64" height="64" viewBox="0 0 64 64" style={{ transform: "rotate(-90deg)" }}>
              <circle cx="32" cy="32" r={r} fill="none" stroke="var(--line-2)" strokeWidth="3" />
              <circle cx="32" cy="32" r={r} fill="none" stroke={accent} strokeWidth="3"
                strokeLinecap="round" strokeDasharray={C} strokeDashoffset={offset}
                style={{ transition: "stroke-dashoffset .7s cubic-bezier(.2,.7,.2,1), stroke .35s ease" }} />
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center">
              <span className="text-[18px] font-semibold tnum tracking-tight">{usage.toFixed(0)}</span>
              <span className="text-[9.5px]" style={{ color: "var(--ink-4)", marginTop: -2 }}>%</span>
            </div>
          </div>
          <div className="flex flex-col gap-0.5">
            <div className="text-[10.5px]" style={{ color: "var(--ink-3)" }}>Load avg</div>
            <div className="font-mono text-[12px] tnum" style={{ color: "var(--ink-2)" }}>
              {metrics ? metrics.cpu_load.map(v => v.toFixed(2)).join(" · ") : "—"}
            </div>
          </div>
        </div>
        <div className="flex-1 self-stretch min-w-0" style={{ maxHeight: 52 }}>
          <MiniChart data={history} variant={chartStyle} max={100} color={accent} />
        </div>
      </div>
    </MetricCard>
  );
}

// ─── Memory Card ───────────────────────────────────────────────────────────
function MemoryCard({ metrics, chartStyle, density }) {
  const used = metrics ? metrics.used_memory : 0;
  const total = metrics ? metrics.total_memory : 1;
  const pct = (used / total) * 100;
  const sev = severity(pct, 75, 90);
  const accent = sevColor(sev);
  const swapPct = metrics ? (metrics.used_swap / metrics.total_swap) * 100 : 0;

  return (
    <MetricCard label="Memory" accent={accent} badge={`Swap ${formatBytes(metrics ? metrics.used_swap : 0)}`} density={density}>
      <div className="flex items-baseline gap-1.5 mb-2">
        <span className="text-[34px] font-semibold tracking-tight tnum" style={{ letterSpacing: "-0.02em" }}>{pct.toFixed(0)}</span>
        <span className="text-[15px]" style={{ color: "var(--ink-3)" }}>%</span>
        <span className="ml-auto text-[12px] font-mono tnum" style={{ color: "var(--ink-2)" }}>
          {formatBytes(used)} <span style={{ color: "var(--ink-4)" }}>/ {formatBytes(total)}</span>
        </span>
      </div>
      {/* Stacked bar: used + swap */}
      <div className="relative w-full" style={{ height: 6, background: "var(--bg-2)", border: "1px solid var(--line)", borderRadius: 999, overflow: "hidden" }}>
        <div style={{ position: "absolute", left: 0, top: 0, bottom: 0, width: `${pct}%`, background: accent, transition: "width .7s cubic-bezier(.2,.7,.2,1)" }} />
        {swapPct > 1 && (
          <div title={`Swap ${swapPct.toFixed(0)}%`} style={{ position: "absolute", left: `${pct}%`, top: 0, bottom: 0, width: `${Math.min(swapPct / 4, 100 - pct)}%`, background: "var(--warm)", opacity: 0.65 }} />
        )}
      </div>
      <div className="mt-2.5 flex items-center justify-between text-[10.5px]" style={{ color: "var(--ink-3)" }}>
        <span><span className="inline-block w-2 h-2 rounded-sm align-middle" style={{ background: accent }} /> Used</span>
        <span><span className="inline-block w-2 h-2 rounded-sm align-middle" style={{ background: "var(--warm)" }} /> Swap</span>
        <span><span className="inline-block w-2 h-2 rounded-sm align-middle" style={{ background: "var(--bg-2)", border: "1px solid var(--line-2)" }} /> Free</span>
      </div>
    </MetricCard>
  );
}

// ─── Network Card ──────────────────────────────────────────────────────────
function NetworkCard({ metrics, txHistory, rxHistory, chartStyle, density }) {
  const tx = metrics ? metrics.network_tx : 0;
  const rx = metrics ? metrics.network_rx : 0;
  const txMax = Math.max(...txHistory, 1);
  const rxMax = Math.max(...rxHistory, 1);

  return (
    <MetricCard label="Network" accent="var(--accent)" badge="eth0" density={density}>
      <div className="grid grid-cols-2 gap-3">
        <NetRow label="Out" value={tx} arrow="up" color="var(--accent)" data={txHistory} max={txMax} chartStyle={chartStyle} />
        <NetRow label="In" value={rx} arrow="down" color="var(--warm)" data={rxHistory} max={rxMax} chartStyle={chartStyle} />
      </div>
    </MetricCard>
  );
}

function NetRow({ label, value, arrow, color, data, max, chartStyle }) {
  return (
    <div className="surface-2" style={{ padding: 10 }}>
      <div className="flex items-center gap-1.5 mb-1">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round">
          {arrow === "up" ? <path d="M12 19V5M5 12l7-7 7 7" /> : <path d="M12 5v14M5 12l7 7 7-7" />}
        </svg>
        <span className="text-[10.5px]" style={{ color: "var(--ink-3)" }}>{label}</span>
      </div>
      <div className="text-[18px] font-semibold tnum tracking-tight" style={{ letterSpacing: "-0.01em" }}>
        {formatRate(value)}
      </div>
      <div className="mt-1.5" style={{ height: 22 }}>
        <MiniChart data={data} variant={chartStyle} w={80} h={22} max={max} color={color} />
      </div>
    </div>
  );
}

// ─── Storage Card ──────────────────────────────────────────────────────────
function StorageCard({ metrics, density }) {
  const disks = metrics ? metrics.disks : [];
  const root = disks.find(d => d.mount_point === "/") || disks[0];
  const totalSpace = root ? root.total_space : 0;
  const used = root ? root.total_space - root.available_space : 0;
  const pct = totalSpace > 0 ? (used / totalSpace) * 100 : 0;
  const sev = severity(pct, 75, 90);
  const accent = sevColor(sev);

  const totalRead = disks.reduce((s, d) => s + d.read_bytes, 0);
  const totalWrite = disks.reduce((s, d) => s + d.written_bytes, 0);

  return (
    <MetricCard label="Storage" accent={accent} badge={`${disks.length} mounts`} density={density}>
      <div className="flex items-baseline gap-1.5 mb-2">
        <span className="text-[34px] font-semibold tracking-tight tnum" style={{ letterSpacing: "-0.02em" }}>{pct.toFixed(0)}</span>
        <span className="text-[15px]" style={{ color: "var(--ink-3)" }}>%</span>
        <span className="ml-auto text-[12px] font-mono tnum" style={{ color: "var(--ink-2)" }}>
          {formatBytes(used)} <span style={{ color: "var(--ink-4)" }}>/ {formatBytes(totalSpace)}</span>
        </span>
      </div>
      {/* Per-mount stacked bar */}
      <div className="flex w-full gap-1" style={{ height: 6 }}>
        {disks.map((d, i) => {
          const dPct = (d.total_space - d.available_space) / d.total_space * 100;
          const flex = d.total_space / disks.reduce((s, x) => s + x.total_space, 0);
          return (
            <div key={d.name} style={{ flex, background: "var(--bg-2)", border: "1px solid var(--line)", borderRadius: 3, overflow: "hidden", position: "relative" }} title={`${d.mount_point} · ${dPct.toFixed(0)}%`}>
              <div style={{ position: "absolute", inset: 0, width: `${dPct}%`, background: i === 0 ? accent : "var(--ink-3)", opacity: i === 0 ? 1 : 0.4 }} />
            </div>
          );
        })}
      </div>
      <div className="mt-2 flex items-center gap-3 text-[10.5px] font-mono tnum" style={{ color: "var(--ink-3)" }}>
        <span><span style={{ color: "var(--accent)" }}>↓</span> {formatRate(totalRead)}</span>
        <span><span style={{ color: "var(--warm)" }}>↑</span> {formatRate(totalWrite)}</span>
        <span className="ml-auto truncate" style={{ color: "var(--ink-4)" }}>{root ? root.mount_point : ""}</span>
      </div>
    </MetricCard>
  );
}

// ─── History Chart ─────────────────────────────────────────────────────────
function HistoryChart({ range, setRange, metric, setMetric, history, live }) {
  const ranges = ["6h", "24h", "7d", "all"];
  const metrics = [
    { id: "cpu", label: "CPU", unit: "%", color: "var(--accent)", get: p => p.cpu_usage, max: 100 },
    { id: "mem", label: "Memory", unit: "%", color: "var(--warm)", get: p => (p.used_memory / (15.5 * 1024 ** 3)) * 100, max: 100 },
    { id: "net", label: "Network", unit: "MB/s", color: "var(--accent)", get: p => (p.network_tx + p.network_rx) / (1024 * 1024), max: null },
    { id: "disk", label: "Disk I/O", unit: "MB/s", color: "var(--warm)", get: p => (p.disk_read_rate + p.disk_write_rate) / (1024 * 1024), max: null },
  ];
  const m = metrics.find(x => x.id === metric) || metrics[0];

  const points = useMemo(() => history.map(m.get), [history, metric]);
  const maxVal = m.max != null ? m.max : Math.max(...points, 1) * 1.15;
  const minVal = 0;
  const cur = points[points.length - 1] || 0;
  const avg = points.reduce((s, v) => s + v, 0) / Math.max(1, points.length);
  const peak = Math.max(...points, 0);

  // SVG dims (responsive via viewBox)
  const W = 800, H = 220, PAD_L = 38, PAD_R = 14, PAD_T = 14, PAD_B = 22;
  const plotW = W - PAD_L - PAD_R;
  const plotH = H - PAD_T - PAD_B;
  const stepX = plotW / Math.max(1, points.length - 1);

  function xy(i, v) {
    const x = PAD_L + i * stepX;
    const y = PAD_T + (1 - (v - minVal) / (maxVal - minVal)) * plotH;
    return [x, y];
  }
  const pts = points.map((v, i) => xy(i, v));
  const pathD = pts.length > 1 ? `M ${pts.map(p => p.join(",")).join(" L ")}` : "";
  const areaD = pts.length > 1 ? `M ${pts[0][0]},${PAD_T + plotH} L ${pts.map(p => p.join(",")).join(" L ")} L ${pts[pts.length - 1][0]},${PAD_T + plotH} Z` : "";
  const last = pts[pts.length - 1] || [PAD_L, PAD_T + plotH];

  // gridlines: 4 horizontal
  const gridY = [0, 0.25, 0.5, 0.75, 1].map(f => PAD_T + f * plotH);
  const yLabels = [0, 0.25, 0.5, 0.75, 1].map(f => {
    const val = minVal + (1 - f) * (maxVal - minVal);
    if (m.unit === "%") return val.toFixed(0);
    return val < 10 ? val.toFixed(1) : val.toFixed(0);
  });

  // x labels
  const xTicks = [0, 0.25, 0.5, 0.75, 1].map(f => Math.round(f * (history.length - 1)));

  return (
    <div className="surface fade-up" style={{ padding: 22 }}>
      <div className="flex items-center justify-between flex-wrap gap-3 mb-4">
        <div className="flex items-center gap-3">
          <div className="eyebrow">History</div>
          <div className="flex rounded-[8px]" style={{ background: "var(--bg-2)", border: "1px solid var(--line)", padding: 2 }}>
            {metrics.map(opt => (
              <button key={opt.id} className="text-[11.5px] font-medium" onClick={() => setMetric(opt.id)}
                style={{
                  padding: "5px 10px", borderRadius: 6,
                  background: metric === opt.id ? "var(--bg-1)" : "transparent",
                  color: metric === opt.id ? "var(--ink)" : "var(--ink-3)",
                  border: metric === opt.id ? "1px solid var(--line-2)" : "1px solid transparent",
                  cursor: "default",
                  transition: "all .15s ease"
                }}>{opt.label}</button>
            ))}
          </div>
        </div>
        <div className="flex items-center gap-3">
          <Stat label="now" value={fmtMetric(cur, m)} color={m.color} pulse />
          <span className="w-px h-4" style={{ background: "var(--line)" }} />
          <Stat label="avg" value={fmtMetric(avg, m)} />
          <Stat label="peak" value={fmtMetric(peak, m)} />
          <span className="w-px h-4" style={{ background: "var(--line)" }} />
          <div className="flex rounded-[8px]" style={{ background: "var(--bg-2)", border: "1px solid var(--line)", padding: 2 }}>
            {ranges.map(r => (
              <button key={r} className="text-[11px] font-medium font-mono tnum" onClick={() => setRange(r)}
                style={{
                  padding: "4px 8px", borderRadius: 6,
                  background: range === r ? "var(--bg-1)" : "transparent",
                  color: range === r ? "var(--ink)" : "var(--ink-3)",
                  border: range === r ? "1px solid var(--line-2)" : "1px solid transparent",
                  cursor: "default",
                }}>{r}</button>
            ))}
          </div>
        </div>
      </div>

      <div className="chart-grid" style={{ borderRadius: 10, overflow: "hidden" }}>
        <svg width="100%" viewBox={`0 0 ${W} ${H}`} preserveAspectRatio="none" style={{ display: "block", height: 240 }}>
          <defs>
            <linearGradient id="hist-fill" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor={m.color} stopOpacity="0.28" />
              <stop offset="100%" stopColor={m.color} stopOpacity="0" />
            </linearGradient>
          </defs>
          {/* Grid lines */}
          {gridY.map((y, i) => (
            <line key={i} x1={PAD_L} x2={W - PAD_R} y1={y} y2={y} stroke="var(--line)" strokeWidth="1" />
          ))}
          {/* Y labels */}
          {yLabels.map((lbl, i) => (
            <text key={i} x={PAD_L - 8} y={gridY[i]} textAnchor="end" dominantBaseline="middle"
                  fontFamily="Geist Mono, monospace" fontSize="10" fill="var(--ink-4)">{lbl}</text>
          ))}
          {/* X labels */}
          {xTicks.map((i, idx) => {
            const p = history[i]; if (!p) return null;
            const x = PAD_L + (i / (history.length - 1)) * plotW;
            const t = new Date(p.timestamp);
            let label;
            if (range === "7d") label = ["Sun","Mon","Tue","Wed","Thu","Fri","Sat"][t.getDay()];
            else label = `${t.getHours().toString().padStart(2,"0")}:${t.getMinutes().toString().padStart(2,"0")}`;
            return <text key={idx} x={x} y={H - 4} textAnchor="middle"
                         fontFamily="Geist Mono, monospace" fontSize="10" fill="var(--ink-4)">{label}</text>;
          })}
          {/* Area + line */}
          <path d={areaD} fill="url(#hist-fill)" />
          <path d={pathD} fill="none" stroke={m.color} strokeWidth="1.6" vectorEffect="non-scaling-stroke" strokeLinejoin="round" />
          {/* Leading dot */}
          {live && (
            <>
              <circle className="lead-dot" cx={last[0]} cy={last[1]} r="4" fill={m.color} />
              <circle cx={last[0]} cy={last[1]} r="9" fill={m.color} opacity="0.18" />
            </>
          )}
        </svg>
      </div>
    </div>
  );
}

function fmtMetric(v, m) {
  if (m.unit === "%") return `${v.toFixed(1)}%`;
  return `${v.toFixed(v < 10 ? 2 : 1)} ${m.unit}`;
}

function Stat({ label, value, color, pulse }) {
  return (
    <div className="flex flex-col items-end" style={{ minWidth: 60 }}>
      <div className="eyebrow" style={{ fontSize: 9.5 }}>{label}</div>
      <div className="text-[13px] font-mono tnum font-medium" style={{ color: color || "var(--ink)" }}>
        {value}
        {pulse && <span className="live-dot inline-block w-1.5 h-1.5 rounded-full ml-1.5 align-middle" style={{ background: color, color: color }} />}
      </div>
    </div>
  );
}

// ─── Process List ──────────────────────────────────────────────────────────
function ProcessList({ processes, totalMemory, enabled = true }) {
  const [sortBy, setSortBy] = useState("cpu");
  const sorted = useMemo(() => {
    if (!processes) return [];
    return [...processes].sort((a, b) =>
      sortBy === "cpu" ? b.cpu_usage - a.cpu_usage : b.memory - a.memory
    ).slice(0, 8);
  }, [processes, sortBy]);

  return (
    <div className="surface fade-up" style={{ padding: 22, display: "flex", flexDirection: "column", height: "100%" }}>
      <div className="flex items-center justify-between mb-4">
        <div className="eyebrow">Top Processes</div>
        {enabled && (
          <div className="flex rounded-[8px]" style={{ background: "var(--bg-2)", border: "1px solid var(--line)", padding: 2 }}>
            {["cpu", "mem"].map(k => (
              <button key={k} className="text-[11px] font-medium uppercase tracking-wider" onClick={() => setSortBy(k)}
                style={{
                  padding: "4px 10px", borderRadius: 6,
                  background: sortBy === k ? "var(--bg-1)" : "transparent",
                  color: sortBy === k ? "var(--ink)" : "var(--ink-3)",
                  border: sortBy === k ? "1px solid var(--line-2)" : "1px solid transparent",
                  cursor: "default",
                }}>{k}</button>
            ))}
          </div>
        )}
      </div>

      {!enabled ? (
        <ProcessEmpty />
      ) : (
        <>
          <div className="grid items-center px-2 pb-2 text-[10.5px] eyebrow"
               style={{ gridTemplateColumns: "1fr 60px 80px 70px", gap: 8, borderBottom: "1px solid var(--line)" }}>
            <span>Process</span>
            <span className="text-right">PID</span>
            <span className="text-right">CPU</span>
            <span className="text-right">Memory</span>
          </div>
          <div className="nice-scroll" style={{ flex: 1, overflowY: "auto", minHeight: 0 }}>
            {sorted.map((p, i) => {
              const memPct = (p.memory / totalMemory) * 100;
              const sev = severity(p.cpu_usage, 50, 80);
              return (
                <div key={`${p.pid}-${i}`} className="grid items-center px-2 py-2.5 text-[12.5px]"
                     style={{ gridTemplateColumns: "1fr 60px 80px 70px", gap: 8, borderBottom: "1px solid var(--line)" }}>
                  <div className="flex items-center gap-2 min-w-0">
                    <span className="text-[10px] tnum font-mono" style={{ color: "var(--ink-4)", width: 14 }}>{i + 1}</span>
                    <span className="truncate" style={{ color: "var(--ink)" }}>{p.name}</span>
                  </div>
                  <span className="text-right tnum font-mono text-[11px]" style={{ color: "var(--ink-3)" }}>{p.pid}</span>
                  <div className="flex items-center justify-end gap-2">
                    <div style={{ width: 24, height: 4, background: "var(--bg-2)", borderRadius: 999, overflow: "hidden" }}>
                      <div style={{ width: `${Math.min(p.cpu_usage, 100)}%`, height: "100%", background: sevColor(sev) }} />
                    </div>
                    <span className="tnum font-mono text-[12px]" style={{ color: sevColor(sev), minWidth: 36, textAlign: "right" }}>{p.cpu_usage.toFixed(1)}%</span>
                  </div>
                  <div className="flex items-center justify-end gap-2">
                    <div style={{ width: 24, height: 4, background: "var(--bg-2)", borderRadius: 999, overflow: "hidden" }}>
                      <div style={{ width: `${Math.min(memPct * 4, 100)}%`, height: "100%", background: "var(--warm)", opacity: 0.6 }} />
                    </div>
                    <span className="tnum font-mono text-[12px]" style={{ color: "var(--ink-2)", minWidth: 50, textAlign: "right" }}>{formatBytes(p.memory)}</span>
                  </div>
                </div>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}

function ProcessEmpty() {
  return (
    <div className="flex-1 flex flex-col items-center justify-center text-center px-6 gap-3 py-12">
      <div className="surface-2 flex items-center justify-center" style={{ width: 44, height: 44, borderRadius: 12 }}>
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" strokeWidth="1.6">
          <path d="M9 3H5a2 2 0 0 0-2 2v4M9 3h10a2 2 0 0 1 2 2v4M9 3v18M9 21H5a2 2 0 0 1-2-2V9M3 9h18M21 9v10a2 2 0 0 1-2 2H9" strokeLinejoin="round" />
        </svg>
      </div>
      <div>
        <div className="text-[13px] font-medium" style={{ color: "var(--ink-2)" }}>Process monitoring is off</div>
        <div className="text-[11.5px] mt-1" style={{ color: "var(--ink-3)" }}>
          Enable in <span style={{ color: "var(--ink-2)" }}>Settings → Processes</span> or start with{" "}
          <code className="font-mono px-1 rounded" style={{ background: "var(--bg-2)", color: "var(--accent)", border: "1px solid var(--line)" }}>--enable-process-list</code>
        </div>
      </div>
    </div>
  );
}

// ─── Toast (live alert flash) ──────────────────────────────────────────────
function Toast({ alerts, onDismiss }) {
  if (!alerts.length) return null;
  return (
    <div style={{ position: "fixed", top: 20, left: "50%", transform: "translateX(-50%)", zIndex: 60, display: "flex", flexDirection: "column", gap: 8, alignItems: "center" }}>
      {alerts.slice(-3).map((a, i) => (
        <div key={a.timestamp} className="toast-in surface" style={{ padding: "10px 14px", display: "flex", alignItems: "center", gap: 12, minWidth: 320, background: "var(--bg-1)", borderColor: "var(--warm-line)" }}>
          <div className="flex items-center justify-center" style={{ width: 28, height: 28, borderRadius: 8, background: "var(--warm-soft)" }}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--warm)" strokeWidth="2">
              <path d="M10.3 3.86 1.82 18a2 2 0 0 0 1.72 3h16.92a2 2 0 0 0 1.72-3L13.71 3.86a2 2 0 0 0-3.42 0z" strokeLinejoin="round" />
              <line x1="12" y1="9" x2="12" y2="13" strokeLinecap="round" />
              <line x1="12" y1="17" x2="12.01" y2="17" strokeLinecap="round" />
            </svg>
          </div>
          <div className="flex-1">
            <div className="text-[12.5px] font-medium">{a.message}</div>
            <div className="text-[11px] mt-0.5 font-mono tnum" style={{ color: "var(--ink-3)" }}>
              <span style={{ color: "var(--warm)" }}>{a.value.toFixed(1)}%</span> · threshold {a.threshold}%
            </div>
          </div>
          <button className="btn-ghost btn" style={{ padding: 4 }} onClick={() => onDismiss(i)}>
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      ))}
    </div>
  );
}

// Expose to global scope so other Babel scripts can reference.
Object.assign(window, {
  TopBar, CpuCard, MemoryCard, NetworkCard, StorageCard,
  HistoryChart, ProcessList, Toast,
  formatBytes, formatRate, formatUptime, formatRelative,
  severity, sevColor, MiniChart, Sigil
});
