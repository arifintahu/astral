# Astral — Task Breakdown

Derived from PM + UX/UI review of the current dashboard.
Priority: P0 (critical) → P1 (important) → P2 (nice to have).

---

## P0 — Broken First-Run Experience

### T-01 Fix empty TOP PROCESSES panel
- **Problem:** Panel renders column headers with nothing below when `--enable-process-list` is not set. Looks broken.
- **Fix:** Add an empty state component — icon + explanation text + the CLI flag to enable it.
- **Scope:** `web/src/lib/components/ProcessList.svelte`
- **Effort:** XS

### T-02 Expose process list toggle in Settings
- **Problem:** Users can't enable process collection from the UI; requires restarting with a CLI flag.
- **Fix:** Add a toggle in the Settings dropdown that calls a new `POST /api/settings` endpoint. Backend reacts by enabling/disabling process collection at runtime.
- **Scope:** `web/src/lib/components/Settings.svelte`, `src/main.rs`, `src/metrics.rs`
- **Effort:** M

### T-03 Expose alert thresholds in Settings
- **Problem:** CPU and RAM alert thresholds are CLI-only. Operators can't tune them without restarting.
- **Fix:** Add threshold inputs (CPU %, RAM %) in Settings dropdown. Persist to a settings row in SQLite or an in-memory `Arc<RwLock<Config>>` and reload on change.
- **Scope:** `web/src/lib/components/Settings.svelte`, `src/main.rs`, `src/worker.rs`
- **Effort:** M

---

## P0 — Visual Clarity Failures

### T-04 Fix CPU gauge arc visibility
- **Problem:** The circular gauge arc is nearly invisible against the dark background. At 0% it looks like a rendering failure, not a metric.
- **Fix:** Always render the full arc as a dim background track (e.g., 15% white opacity). The filled arc overlays it. Matches standard gauge conventions.
- **Scope:** `web/src/lib/components/CpuCard.svelte`
- **Effort:** XS

### T-05 Fix card label contrast
- **Problem:** "PROCESSOR", "MEMORY", "NETWORK", "STORAGE" labels are too dim to scan at a glance. Monitoring dashboards get glanced at, not read.
- **Fix:** Increase label opacity and font weight. Optionally add a 2px top-border accent in the card's theme color for instant spatial identity.
- **Scope:** All four metric card components
- **Effort:** XS

### T-06 Fix history chart legend
- **Problem:** `Time: -  CPU: -  Memory: -` shows dashes by default, reads as broken placeholder text.
- **Fix:** Show static color swatches + labels always. Move hover values to a floating tooltip overlay, not the legend line.
- **Scope:** `web/src/lib/components/HistoryChart.svelte`
- **Effort:** XS

---

## P1 — Missing Operational Features

### T-07 Add alert history log
- **Problem:** Toast alerts auto-dismiss in 10 seconds. If the operator wasn't watching, the alert is lost with no record.
- **Fix:** Maintain an in-memory ring buffer (last 50 alerts) on the backend. Add `GET /api/alerts/history` endpoint. Add an "Alerts" section in the Settings panel or a dedicated slide-out drawer.
- **Scope:** `src/worker.rs`, `src/api.rs` (or new `src/alerts.rs`), `web/src/lib/components/`
- **Effort:** M

### T-08 Add network I/O history to chart
- **Problem:** TX/RX speeds are live-only. Can't answer "was there a traffic spike at 3am?"
- **Fix:** Store `network_tx_rate` and `network_rx_rate` (delta bytes/s) in all aggregation tables. Extend history API response. Add network series toggle to the History chart.
- **Scope:** `src/db.rs`, `src/worker.rs`, `src/api.rs`, `web/src/lib/components/HistoryChart.svelte`, `web/src/lib/types.ts`
- **Effort:** L

### T-09 Add disk I/O history to chart
- **Problem:** Read/write bytes are shown as a live snapshot only. Historical I/O trends are invisible.
- **Fix:** Same approach as T-08 — store disk read/write rates in aggregation tables and expose in history API.
- **Scope:** Same modules as T-08
- **Effort:** L

### T-10 Add per-disk breakdown in Storage card
- **Problem:** With 5 mounts, you can't see which specific disk is full — only an aggregate.
- **Fix:** Expand Storage card to show a collapsible list of mounts with individual usage bars when >1 disk is present.
- **Scope:** `web/src/lib/components/DiskCard.svelte`
- **Effort:** S

### T-11 Fix network unit consistency
- **Problem:** Upload shows `1 KB/s` while download shows `66 B/s` side-by-side. Different units prevent visual comparison.
- **Fix:** Normalize TX and RX to the same unit within the card at a given moment (use the larger value's unit for both).
- **Scope:** `web/src/lib/components/NetworkCard.svelte`
- **Effort:** XS

### T-12 Fix disk I/O zero on first render
- **Problem:** "Read 0 B / Write 0 B" on fresh load looks like a bug — these are deltas that start at zero.
- **Fix:** Show `—` until at least two ticks have passed and a real delta exists. Or switch to a B/s rate display like network does.
- **Scope:** `web/src/lib/components/DiskCard.svelte`, `web/src/App.svelte`
- **Effort:** XS

### T-13 Add alert badge to Top Bar status indicator
- **Problem:** When an alert fires, the Top Bar stays green (LIVE). The only signal is a toast in the corner that disappears in 10s.
- **Fix:** When one or more active alerts exist, change the status indicator color to red and show a count badge. Clear when alerts are dismissed or resolved.
- **Scope:** `web/src/lib/components/TopBar.svelte`, `web/src/App.svelte`
- **Effort:** S

### T-14 Improve card visual separation
- **Problem:** Cards barely separate from the background. Under ambient light or lower-contrast monitors, the grid reads as one undivided block — spatial memory is harder.
- **Fix:** Add `border border-white/10` or a subtle top-left highlight to each card. Reference: Grafana, Netdata dark themes.
- **Scope:** All card components
- **Effort:** XS

---

## P2 — Growth & Polish

### T-15 Add data retention control in Settings
- **Problem:** `--retention` flag controls 1h data retention but requires restart to change.
- **Fix:** Add a retention period selector in Settings (7d / 30d / 90d). Persist to SQLite settings table.
- **Scope:** `web/src/lib/components/Settings.svelte`, `src/main.rs`, `src/db.rs`
- **Effort:** S

### T-16 Add historical data export (CSV)
- **Problem:** Operators can't pull historical data for incident postmortems.
- **Fix:** Add `GET /api/history/export?window=24h&format=csv` endpoint. Add an export button in the History chart header.
- **Scope:** `src/api.rs`, `web/src/lib/components/HistoryChart.svelte`
- **Effort:** S

### T-17 Add native Slack alerting integration
- **Problem:** Webhook-only alerting requires operators to set up their own relay. Slack is the most common ops notification channel.
- **Fix:** Add `--slack-webhook <URL>` CLI arg (or UI config). Format Slack Block Kit message with metric context.
- **Scope:** `src/worker.rs`, `src/main.rs`
- **Effort:** S

### T-18 Multi-server support (future)
- **Problem:** Astral can only monitor the host it runs on. Fleet operators need a single pane of glass.
- **Fix:** Design options: (a) agent mode — each host runs Astral, a central instance aggregates; (b) remote SSH polling. Requires API contract design before implementation.
- **Scope:** Architecture decision first
- **Effort:** XL

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

## Quick Wins (ship first)

T-04, T-05, T-06, T-11, T-12, T-14 — all XS, frontend-only, no backend changes needed.
Total estimated effort: ~2 hours.
