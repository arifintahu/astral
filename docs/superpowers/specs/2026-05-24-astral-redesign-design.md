# Astral UI Redesign — Design Spec

**Date:** 2026-05-24  
**Accent colour:** Sky-400 `#38bdf8`  
**Approach:** Exact prototype translation (CSS variable token system + custom SVG chart + full feature port)

---

## 1. Scope

Full port of the React prototype (from `Astral.zip`) to the existing Svelte 5 codebase. Covers:

- New CSS design-token system replacing the current glassmorphism style
- Geist + Geist Mono fonts replacing Inter + JetBrains Mono
- Rewritten Svelte components matching the prototype layout exactly
- History chart replaced: uPlot removed, custom SVG renderer added
- New features: multi-metric history tabs, process memory sort, full Settings drawer
- Split login screen with ambient SVG
- Backend: persistent settings API, sessions revoke endpoint, webhook delivery, configurable alert thresholds and retention

---

## 2. Design Token System

### CSS custom properties (replaces all of `app.css`)

```css
:root {
  color-scheme: dark;

  /* Text */
  --ink:   #fafafa;
  --ink-2: rgba(250, 250, 250, 0.62);
  --ink-3: rgba(250, 250, 250, 0.42);
  --ink-4: rgba(250, 250, 250, 0.22);

  /* Backgrounds */
  --bg:   #08090b;   /* page */
  --bg-1: #0d0e11;   /* surface (card) */
  --bg-2: #131418;   /* surface-2 (inset) */

  /* Borders */
  --line:   rgba(255, 255, 255, 0.07);
  --line-2: rgba(255, 255, 255, 0.12);

  /* Accent — Sky-400 */
  --accent:      #38bdf8;
  --accent-soft: rgba(56, 189, 248, 0.14);
  --accent-line: rgba(56, 189, 248, 0.38);

  /* Semantic */
  --warm:      #f59e0b;
  --warm-soft: rgba(245, 158, 11, 0.14);
  --warm-line: rgba(245, 158, 11, 0.38);
  --crit:      #f43f5e;
  --crit-soft: rgba(244, 63, 94, 0.14);
  --ok:        #34d399;
}
```

### CSS utility classes (defined in `app.css`)

| Class | Purpose |
|---|---|
| `.surface` | Card shell: `bg-1`, 1px `--line` border, 14px radius |
| `.surface-2` | Inset element: `bg-2`, 1px `--line` border, 10–12px radius |
| `.eyebrow` | Label: 10.5px, 500 weight, 0.12em tracking, uppercase, `--ink-3` |
| `.tnum` | `font-variant-numeric: tabular-nums` |
| `.btn` | Base button: 12px, 500 weight, `bg-2`, `--line` border, 8px radius |
| `.btn-primary` | `--accent` background, `#0b0d12` text |
| `.btn-ghost` | Transparent background and border |
| `.focus-ring` | Outline: 3px `--accent-soft` on focus |
| `.nice-scroll` | Styled scrollbar using `--line-2` |
| `.card-hover` | Border transitions to `--line-2` on hover |
| `.skeleton` | Shimmer animation using `--bg-2` / `--line` gradient |

### Typography

- Body: **Geist** (300–700), loaded via Google Fonts in `index.html`  
- Monospace: **Geist Mono** (400–600), loaded via Google Fonts  
- `font-feature-settings: "ss01", "cv11"` on `html`

### Animations (defined in `app.css`)

| Name | Use |
|---|---|
| `fadeUp` | Staggered card entrance (0ms, 40ms, 80ms, 120ms, 160ms, 200ms delays) |
| `pulseDot` | Live/alert status indicator dot |
| `leadPulse` | Animated leading dot on history chart |
| `tickPulse` | Metric value flash on update |
| `shimmer` | Skeleton loading |
| `drawerIn` | Settings / alert drawer slide-in from right |
| `backdropIn` | Backdrop fade-in |
| `toastIn` | Toast notification slide-down |

---

## 3. Frontend Components

### 3.1 `index.html`

Add Google Fonts `<link>` tags for Geist + Geist Mono. No other changes.

### 3.2 `app.css`

Complete replacement with the token system described in §2. Remove all glassmorphism classes (`glass-panel`, `glass-card`), old palette variables, and Inter/JetBrains Mono references.

### 3.3 `App.svelte`

- Max-width wrapper: `max-width: 1440px; margin: 0 auto; padding: 26px 28px`
- Card grid: `repeat(auto-fit, minmax(240px, 1fr))` with `gap: 20px`
- Bottom row: `2fr 1fr` grid for HistoryChart + ProcessList
- Footer hairline: hostname · process state · retention · updated timestamp · refresh rate
- Skeleton loading state: 4 card skeletons + chart + process skeletons

### 3.4 `TopBar.svelte`

Layout: Sigil logo | name + hostname/OS | right-side pills + icon buttons

**Left side:**
- `Sigil` — 36×36px box, `bg-2` background, `--line-2` border, 10px radius. SVG: concentric arcs in `--accent`
- App name (`Astral`, 15px, 600 weight) + version badge (`v1.1.0`, mono, `--ink-4`)
- Hostname (mono, `--ink-2`) · OS name+version · region tag

**Right side:**
- Live pill (green dot + `Live · {rate}s`) or alert pill (red dot + `{n} alert(s)`, `--crit-soft` background) — clicking alert pill opens AlertHistoryDrawer
- Uptime pill (clock icon + formatted uptime)
- Bell icon button → opens AlertHistoryDrawer
- Gear icon button → opens Settings drawer
- Logout icon button

### 3.5 `CpuCard.svelte`

- Eyebrow label: `CPU` | badge: `{cores} cores`
- Top accent hairline: gradient using `sevColor(usage)`
- Left cluster: 64×64 SVG ring gauge (stroke-dasharray animated) + number overlay + load avg beneath
- Right: `MiniChart` sparkline (flex-1, height 52px)
- Severity thresholds: warn ≥ 70%, crit ≥ 90% → color shifts from `--accent` to `--warm` to `--crit`

### 3.6 `MemoryCard.svelte`

- Eyebrow: `Memory` | badge: `Swap {formatBytes(used_swap)}`
- Large percentage number (34px, 600 weight) + `used / total` right-aligned
- Stacked progress bar (6px height): used in `sevColor`, swap segment in `--warm` at 0.65 opacity, appended after used
- Legend row: Used (accent swatch) · Swap (warm swatch) · Free (bg-2 swatch)

### 3.7 `NetworkCard.svelte`

- Eyebrow: `Network` | badge: interface name (e.g. `eth0`)
- 2-column grid of `NetRow` inset cards
- **TX row** (Out ↑): rate value, `MiniChart` in `--accent`
- **RX row** (In ↓): rate value, `MiniChart` in `--warm`
- Max for each sparkline computed from the respective history array

### 3.8 `DiskCard.svelte`

- Eyebrow: `Storage` | badge: `{n} mounts`
- Large percentage + `used / total` (primary mount)
- Per-mount stacked bar: proportional widths, primary mount in `sevColor`, others in `--ink-3` at 0.4 opacity
- Footer: read rate (↓ accent) · write rate (↑ warm) · primary mount path

### 3.9 `MiniChart.svelte` *(new)*

Reusable SVG mini chart. Props: `data: number[]`, `max: number`, `color: string`, `w = 140`, `h = 48`.

Single variant: sparkline with gradient area fill + line + trailing dot. (The prototype's bar/dot/radial variants are design-tool tweaks; they are not needed in production.)

### 3.10 `HistoryChart.svelte`

**Remove uPlot entirely.** Replace with custom SVG renderer.

**Metric tabs** (single-metric, one at a time):

| Tab | Field | Unit | Color |
|---|---|---|---|
| CPU | `cpu_usage` | % | `--accent` |
| Memory | `used_memory / total_memory * 100` | % | `--warm` |
| Network | `(network_tx + network_rx) / 1e6` | MB/s | `--accent` |
| Disk I/O | `(disk_read_rate + disk_write_rate) / 1e6` | MB/s | `--warm` |

**Range tabs:** 6h → `metrics_5m`, 24h → `metrics_15m`, 7d → `metrics_1h`, all → `metrics_1h` (existing API mapping, unchanged).

**Header row:** eyebrow `History` | metric seg-control | `now` / `avg` / `peak` stats | range seg-control  
**Chart body:** dotted-grid background, SVG area + line, animated leading dot when live  
**Y-axis:** 5 gridlines, labels left-aligned in Geist Mono  
**X-axis:** 5 time labels (HH:MM or day-of-week for 7d)

CSV export button is removed (not present in new design).

### 3.11 `ProcessList.svelte`

- Header: eyebrow `Top Processes` | seg-control `cpu` / `mem`
- Column headers: Process · PID · CPU · Memory (grid: `1fr 60px 80px 70px`)
- Each row: rank number + process name (truncated) · PID (mono) · CPU % with mini bar · memory with mini bar
- Sort by CPU: order by `cpu_usage` desc. Sort by mem: order by `memory` desc. Top 8 shown.

### 3.12 `Login.svelte`

- Full-viewport 2-column grid: left form, right ambient panel
- **Left:** Sigil + app name, "Welcome back." heading, username + password fields, submit button, TLS disclaimer
- **Right (`AmbientPanel`):** `bg-1` background, radial gradient blobs in `--accent` and `--warm`, dotted grid SVG, two animated sine-wave curves, two floating metric chips (CPU %, Memory GB)

### 3.13 `Settings.svelte` → full drawer

Replace current popover with a right-side drawer (matches `AlertHistoryDrawer` pattern):

- Fixed position, right edge, full viewport height, 440px wide, `drawerIn` animation
- Backdrop overlay with `backdropIn`
- **General:** refresh rate seg (1s/2s/5s/10s), process monitoring toggle
- **Alerts:** CPU threshold slider (50–100%), memory threshold slider (50–100%), Slack webhook text input
- **Data:** retention seg (7d/30d/90d), DB stats + Export CSV button
- **Account:** sessions info + Revoke all (danger), Sign out (danger)
- Footer: version string | Done button (primary)
- All values loaded from `GET /api/settings` on mount; saved on Done via `POST /api/settings`

### 3.14 `AlertHistoryDrawer.svelte`

Update styling only — no functional changes. Replace `glass-panel` / Tailwind classes with new token system. Timeline rail, day grouping, and alert items stay the same.

### 3.15 `Toast.svelte`

Update styling only. Replace glassmorphism classes with `.surface` + `--warm-line` border. Keep fixed-top positioning and dismiss logic.

---

## 4. Backend Changes

### 4.1 Settings data model (`src/db.rs`)

New SQLite table created at startup alongside existing metric tables:

```sql
CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```

Default values inserted on first run:

| key | default |
|---|---|
| `refresh_rate` | `1` |
| `process_list` | `true` |
| `alert_cpu` | `90` |
| `alert_ram` | `90` |
| `webhook_url` | `""` |
| `retention_days` | `30` |

Helper functions: `get_setting(key) -> String`, `set_setting(key, value)`, `get_all_settings() -> HashMap<String, String>`.

### 4.2 Settings struct (`src/settings.rs`) *(new file)*

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub refresh_rate:   u32,
    pub process_list:   bool,
    pub alert_cpu:      f64,
    pub alert_ram:      f64,
    pub webhook_url:    String,
    pub retention_days: u32,
}
```

Loaded from the DB at startup and stored in an `Arc<RwLock<AppSettings>>` shared across handlers and the worker.

### 4.3 Settings API (`src/api.rs`)

```
GET  /api/settings         → JSON AppSettings
POST /api/settings         → body: partial or full AppSettings JSON → persists each field, returns updated AppSettings
```

Both endpoints require the existing Basic Auth middleware.

### 4.4 Sessions revoke (`src/api.rs`)

```
POST /api/sessions/revoke  → clears all active session tokens from the DB (new `sessions` table or existing auth store)
```

Returns `204 No Content`. Requires auth.

### 4.5 Worker alert thresholds (`src/worker.rs`)

Worker receives `Arc<RwLock<AppSettings>>`. On each alert-check tick it reads `settings.alert_cpu` and `settings.alert_ram` instead of hardcoded values.

### 4.6 Webhook delivery (`src/worker.rs`)

When an alert fires and `settings.webhook_url` is non-empty:
- POST JSON payload to the webhook URL using `reqwest` (already a transitive dep via `sqlx`; add explicitly if needed)
- Payload: `{ "text": "CPU usage 94.6% — threshold 90% — edge-fra-01" }` (Slack-compatible)
- 15-minute per-kind cooldown enforced in worker state (not persisted across restarts)
- Fire-and-forget — errors logged but not fatal

### 4.7 Retention enforcement (`src/worker.rs`)

Worker reads `settings.retention_days` on each hourly cleanup tick. Deletes rows older than `now - retention_days * 86400` from all four metric tables. Currently the retention is hardcoded; this makes it dynamic.

### 4.8 CSV export endpoint (`src/api.rs`)

The existing `/api/history/export` endpoint is retained as-is (the Export CSV button in the Settings drawer calls it). No changes needed.

---

## 5. File Change Summary

### Frontend (`web/`)

| File | Action |
|---|---|
| `index.html` | Add Geist font `<link>` tags |
| `src/app.css` | Full replacement — new token system |
| `src/App.svelte` | Rewrite layout, grid, footer, skeleton |
| `src/lib/components/TopBar.svelte` | Rewrite — Sigil, pills, icon buttons |
| `src/lib/components/CpuCard.svelte` | Rewrite — ring gauge, MiniChart |
| `src/lib/components/MemoryCard.svelte` | Rewrite — stacked bar, legend |
| `src/lib/components/NetworkCard.svelte` | Rewrite — TX/RX rows |
| `src/lib/components/DiskCard.svelte` | Rewrite — per-mount bar |
| `src/lib/components/HistoryChart.svelte` | Rewrite — remove uPlot, custom SVG, 4 tabs |
| `src/lib/components/ProcessList.svelte` | Rewrite — mem sort, new grid |
| `src/lib/components/Login.svelte` | Rewrite — split layout, AmbientPanel |
| `src/lib/components/Settings.svelte` | Rewrite — full drawer, API-wired |
| `src/lib/components/AlertHistoryDrawer.svelte` | Restyle only |
| `src/lib/components/Toast.svelte` | Restyle only |
| `src/lib/components/MiniChart.svelte` | **New** — reusable sparkline |
| `web/package.json` | Remove `uplot` dependency |

### Backend (`src/`)

| File | Action |
|---|---|
| `src/db.rs` | Add `settings` table init + helper fns |
| `src/settings.rs` | **New** — `AppSettings` struct + `Arc<RwLock>` setup |
| `src/api.rs` | Add `GET/POST /api/settings`, `POST /api/sessions/revoke` |
| `src/worker.rs` | Read thresholds + retention from settings, webhook delivery |
| `src/main.rs` | Wire `AppSettings` into app state, pass to worker |
| `Cargo.toml` | Add `reqwest` with `json` feature if not already present |

---

## 6. Build Order

1. Backend changes (settings table, API, worker) — verify with `curl`
2. `app.css` token replacement
3. `MiniChart.svelte` (dependency of card components)
4. Metric cards (Cpu, Memory, Network, Disk) — can be done in parallel
5. `TopBar.svelte`
6. `HistoryChart.svelte` (most complex frontend piece)
7. `ProcessList.svelte`
8. `Login.svelte`
9. `Settings.svelte` drawer
10. `AlertHistoryDrawer.svelte` + `Toast.svelte` restyle
11. `App.svelte` layout + skeleton
12. Remove uPlot from `package.json`, run `npm install`
13. Full build + smoke test

---

## 7. Out of Scope

- Light mode (the prototype includes it as a tweak; not implemented)
- Chart style variants (bar / dot / radial mini charts — prototype-only tweak, not in production)
- Tweaks panel (developer tool in the prototype, not shipped)
- Multi-server fleet view
