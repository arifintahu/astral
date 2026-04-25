# Astral — Task Breakdown

Derived from PM + UX/UI review of the current dashboard.
Priority: P0 (critical) → P1 (important) → P2 (nice to have).

---

## P0 — Broken First-Run Experience

### T-01 Fix empty TOP PROCESSES panel ✅
- **Fix:** Added empty state with icon + explanation text + CLI flag hint in `ProcessList.svelte`. Shows when `processes` array is empty.

### T-02 Expose process list toggle in Settings ✅
- **Fix:** Settings dropdown fetches `GET /api/settings` and sends `POST /api/settings` with `enable_process_list`. Backend updates `SharedConfig` at runtime — no restart needed.

### T-03 Expose alert thresholds in Settings ✅
- **Fix:** CPU % and RAM % inputs in Settings with individual Save buttons. Updates `alert_cpu` / `alert_ram` via `POST /api/settings`. Worker reads thresholds from `SharedConfig` on each 1-minute evaluation.

---

## P0 — Visual Clarity Failures

### T-04 Fix CPU gauge arc visibility ✅
- **Fix:** Background arc changed from `rgba(255,255,255,0.04)` to `rgba(255,255,255,0.12)` in `CpuCard.svelte`. The full track is always visible.

### T-05 Fix card label contrast ✅
- **Fix:** All four card labels changed from `metric-label` (dim) to `text-[11px] font-bold text-slate-300 uppercase tracking-[0.15em]`. Each card has a distinct top-border accent color (cyan/purple/amber/emerald).

### T-06 Fix history chart legend ✅
- **Fix:** uPlot's built-in legend hidden (`show: false`). Static custom legend above the chart always shows color swatches + labels. Hover values remain in uPlot's internal tooltip.

---

## P1 — Missing Operational Features

### T-07 Add alert history log ✅
- **Fix:** Backend: `AlertHistory` ring buffer (50 alerts) in `worker.rs`, `GET /api/alerts/history` endpoint. Frontend: `AlertHistoryDrawer.svelte` slide-out panel, triggered from alert badge in TopBar.

### T-08 Add network I/O history to chart ✅
- **Fix:** `network_tx` and `network_rx` were already stored in DB as bytes/s averages. History API already returns them. Added "Network" view to `HistoryChart` with TX/RX series and shared bytes/s y-axis.

### T-09 Add disk I/O history to chart ✅
- **Fix:** Added `disk_read_rate` and `disk_write_rate` columns to all four metric tables (idempotent `ALTER TABLE` migration). Worker computes per-disk aggregates. History API returns them. Added "Disk I/O" view to chart.

### T-10 Add per-disk breakdown in Storage card ✅
- **Fix:** DiskCard shows a collapsible list of all mounts with individual progress bars when >1 disk is present. Toggle button in card header.

### T-11 Fix network unit consistency ✅
- **Fix:** NetworkCard computes `sharedUnitIndex = max(txUnitIndex, rxUnitIndex)` and divides both values by the same divisor. Unit label shown once in the card header.

### T-12 Fix disk I/O zero on first render ✅
- **Fix:** DiskCard now shows `formatRate()` (B/s, KB/s, …) instead of `formatBytes()`. This makes `0 B/s` clearly a rate — not a broken absolute value.

### T-13 Add alert badge to Top Bar status indicator ✅
- **Fix:** TopBar receives `alerts` prop. When `alerts.length > 0`, the green LIVE indicator switches to a pulsing red "Alert" badge with count. Clicking it opens the AlertHistoryDrawer.

### T-14 Improve card visual separation ✅
- **Fix:** All four metric cards have `border border-white/[0.08]` plus a colored top accent (`border-t-{color}/30`). Each card has a distinct accent matching its color theme.

---

## P2 — Growth & Polish

### T-15 Add data retention control in Settings ✅
- **Fix:** Three-button selector in Settings (7d / 30d / 90d). Sends `POST /api/settings` with `retention_days`. Worker reads value from `SharedConfig` on hourly cleanup tick.

### T-16 Add historical data export (CSV) ✅
- **Fix:** `GET /api/history/export?window=6h` endpoint returns CSV with all metric columns. Download button (↓ icon) in HistoryChart header triggers a direct browser download.

### T-17 Add native Slack alerting integration ✅
- **Fix:** Slack webhook input in Settings (persisted via `POST /api/settings`). Worker sends Slack Block Kit payload alongside the existing generic webhook when an alert fires. `--slack-webhook` CLI arg sets the initial value.

### T-18 Multi-server support — SKIPPED
- Requires architecture decision (agent mode vs. SSH polling). Out of scope for this iteration.

---

## Effort Legend

| Size | Estimate |
|------|----------|
| XS | < 1 hour |
| S | 1–3 hours |
| M | 3–8 hours |
| L | 1–2 days |
| XL | 3+ days |

---

## Files Changed

**New files:**
- `src/config.rs` — `DynamicConfig` + `SharedConfig` type
- `web/src/lib/components/AlertHistoryDrawer.svelte` — T-07 drawer

**Modified backend:**
- `src/api.rs` — AlertHistory, settings/alerts-history/export endpoints
- `src/db.rs` — disk_read_rate/disk_write_rate columns + queries
- `src/metrics.rs` — SharedConfig-driven enable_process_list
- `src/worker.rs` — SharedConfig, AlertHistory, disk I/O rates, Slack alerting
- `src/main.rs` — wire SharedConfig, AlertHistory, --slack-webhook arg

**Modified frontend:**
- `web/src/lib/types.ts` — MetricPoint, DynamicConfig interfaces
- `web/src/App.svelte` — AlertHistoryDrawer, alert count to TopBar
- `web/src/lib/components/TopBar.svelte` — alert badge (T-13)
- `web/src/lib/components/Settings.svelte` — process toggle, thresholds, retention, Slack (T-02/03/15/17)
- `web/src/lib/components/CpuCard.svelte` — gauge track, label, border (T-04/05/14)
- `web/src/lib/components/MemoryCard.svelte` — label, border (T-05/14)
- `web/src/lib/components/NetworkCard.svelte` — label, unit consistency, border (T-05/11/14)
- `web/src/lib/components/DiskCard.svelte` — label, per-disk breakdown, I/O rate, border (T-05/10/12/14)
- `web/src/lib/components/ProcessList.svelte` — empty state (T-01)
- `web/src/lib/components/HistoryChart.svelte` — legend, network/disk views, export (T-06/08/09/16)
