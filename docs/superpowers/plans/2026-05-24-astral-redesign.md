# Astral UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the React prototype design (Sky-400 accent, CSS token system, custom SVG chart, new components) to the Svelte 5 codebase, with backend sessions-revoke endpoint.

**Architecture:** CSS custom property token system replaces glassmorphism; Geist/Geist Mono replace Inter/JetBrains Mono; uPlot removed in favour of a hand-rolled SVG chart; Settings moves from a popover-in-TopBar to a standalone right drawer wired to App.svelte state.

**Tech Stack:** Svelte 5 runes, Tailwind v4, Rust/Axum, SQLite/sqlx, sysinfo, Geist (Google Fonts)

---

### Task 1: Backend — POST /api/sessions/revoke

**Files:**
- Modify: `src/api.rs` — add handler + wire AppState
- Modify: `src/main.rs` — share revoke_all_at Arc between AppState and AuthConfig

- [ ] **Step 1: Add `revoke_all_at` to AppState in `src/api.rs`**

Add `pub revoke_all_at: Arc<Mutex<u64>>` field to `AppState` and register the new route.

```rust
// In api.rs — add to imports at top:
use std::time::{SystemTime, UNIX_EPOCH};

// AppState struct becomes:
#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub tx: broadcast::Sender<SystemMetrics>,
    pub alert_tx: broadcast::Sender<AlertEvent>,
    pub config: SharedConfig,
    pub alert_history: AlertHistory,
    pub revoke_all_at: Arc<Mutex<u64>>,
}

// In pub fn app(state: AppState) -> Router, add route:
.route("/api/sessions/revoke", axum::routing::post(revoke_sessions_handler))

// New handler at bottom of file:
async fn revoke_sessions_handler(State(state): State<AppState>) -> impl IntoResponse {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    *state.revoke_all_at.lock().await = now;
    StatusCode::NO_CONTENT
}
```

- [ ] **Step 2: Add `revoke_all_at` to `AuthConfig` and wire in `main.rs`**

```rust
// AuthConfig struct:
#[derive(Clone)]
struct AuthConfig {
    username: String,
    password_hash: String,
    secret: String,
    revocation: RevocationSet,
    revoke_all_at: Arc<Mutex<u64>>,
}

// In main(), after creating revocation:
let revoke_all_at: Arc<Mutex<u64>> = Arc::new(Mutex::new(0u64));
let auth_config = AuthConfig {
    username,
    password_hash,
    secret,
    revocation,
    revoke_all_at: revoke_all_at.clone(),
};

// AppState construction:
let app_state = AppState {
    db,
    tx,
    alert_tx,
    config: shared_config,
    alert_history,
    revoke_all_at,
};
```

- [ ] **Step 3: Update `auth_middleware` to check `revoke_all_at`**

```rust
async fn auth_middleware(req: Request<Body>, next: Next, config: AuthConfig) -> Response {
    if let Some(token) = get_cookie_token(req.headers()) {
        if verify_session_token(&config.secret, &config.username, &token) {
            let sig = token_sig(&token).unwrap_or_default();
            if !config.revocation.lock().await.contains(&sig) {
                // Check revoke_all_at: token timestamp must be after last bulk revocation
                let parts: Vec<&str> = token.splitn(3, '.').collect();
                let token_ts: u64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                let revoke_ts = *config.revoke_all_at.lock().await;
                if token_ts > revoke_ts {
                    return next.run(req).await;
                }
            }
        }
    }
    StatusCode::UNAUTHORIZED.into_response()
}
```

- [ ] **Step 4: Build and verify**

```bash
cargo build 2>&1 | head -40
```

Expected: Compiles with no errors.

- [ ] **Step 5: Commit**

```bash
git add src/api.rs src/main.rs
git commit -m "feat: add POST /api/sessions/revoke endpoint"
```

---

### Task 2: Backend — Add cpu_load to SystemMetrics

**Files:**
- Modify: `src/metrics.rs` — add `cpu_load` field
- Modify: `web/src/lib/types.ts` — mirror the field

- [ ] **Step 1: Add `cpu_load` to `SystemMetrics` struct**

```rust
// In src/metrics.rs, SystemMetrics struct:
#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub uptime: u64,
    pub cpu_usage: f32,
    pub cpu_cores: usize,
    pub cpu_load: [f64; 3],   // <-- add this: [1min, 5min, 15min]
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub network_tx: u64,
    pub network_rx: u64,
    pub disks: Vec<DiskInfo>,
    pub processes: Vec<ProcessInfo>,
}
```

- [ ] **Step 2: Collect load average in `collect()`**

```rust
// At the start of collect(), before the return:
let load = System::load_average();

// In the SystemMetrics { ... } construction:
cpu_load: [load.one, load.five, load.fifteen],
```

- [ ] **Step 3: Update TypeScript types**

```typescript
// In web/src/lib/types.ts, SystemMetrics interface:
export interface SystemMetrics {
    hostname: string;
    os_name: string;
    os_version: string;
    uptime: number;
    cpu_usage: number;
    cpu_cores: number;
    cpu_load: [number, number, number];  // <-- add
    total_memory: number;
    used_memory: number;
    total_swap: number;
    used_swap: number;
    network_tx: number;
    network_rx: number;
    disks: DiskInfo[];
    processes: ProcessInfo[];
}
```

- [ ] **Step 4: Build and commit**

```bash
cargo build && git add src/metrics.rs web/src/lib/types.ts && git commit -m "feat: add cpu_load to SystemMetrics"
```

---

### Task 3: Frontend foundation — CSS tokens, fonts, CSP

**Files:**
- Modify: `web/index.html` — swap to Geist fonts
- Modify: `web/src/app.css` — full token-system replacement
- Modify: `src/main.rs` — update CSP to allow Google Fonts

- [ ] **Step 1: Update `web/index.html` font links**

Replace the Inter+JetBrains Mono `<link>` with:
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Geist:wght@300;400;500;600;700&family=Geist+Mono:wght@400;500;600&display=swap" rel="stylesheet">
```

Also update `<meta name="theme-color" content="#38bdf8" />` (Sky-400).

- [ ] **Step 2: Update CSP in `src/main.rs`**

Replace the CSP header value with:
```
"default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; connect-src 'self'; img-src 'self' data:"
```

- [ ] **Step 3: Replace `web/src/app.css` entirely**

```css
@import "tailwindcss";

:root {
  color-scheme: dark;

  /* Text */
  --ink:   #fafafa;
  --ink-2: rgba(250, 250, 250, 0.62);
  --ink-3: rgba(250, 250, 250, 0.42);
  --ink-4: rgba(250, 250, 250, 0.22);

  /* Backgrounds */
  --bg:   #08090b;
  --bg-1: #0d0e11;
  --bg-2: #131418;

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

html {
  font-feature-settings: "ss01", "cv11";
}

body {
  font-family: 'Geist', system-ui, -apple-system, sans-serif;
  background-color: var(--bg);
  color: var(--ink);
  min-height: 100vh;
  -webkit-font-smoothing: antialiased;
}

.font-mono, code, pre {
  font-family: 'Geist Mono', ui-monospace, monospace;
}

.tnum { font-variant-numeric: tabular-nums; }

/* Surfaces */
.surface {
  background: var(--bg-1);
  border: 1px solid var(--line);
  border-radius: 14px;
}
.surface-2 {
  background: var(--bg-2);
  border: 1px solid var(--line);
  border-radius: 10px;
}

/* Typography */
.eyebrow {
  font-size: 10.5px;
  font-weight: 500;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--ink-3);
}

/* Buttons */
.btn {
  font-size: 12px;
  font-weight: 500;
  background: var(--bg-2);
  border: 1px solid var(--line);
  border-radius: 8px;
  padding: 5px 10px;
  color: var(--ink-2);
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s;
  line-height: 1;
}
.btn:hover { border-color: var(--line-2); }
.btn-primary { background: var(--accent); color: #0b0d12; border-color: transparent; }
.btn-primary:hover { background: #7dd3fc; }
.btn-ghost { background: transparent; border-color: transparent; }

.focus-ring:focus-visible {
  outline: 3px solid var(--accent-soft);
  outline-offset: 2px;
}

/* Scrollbar */
.nice-scroll {
  scrollbar-width: thin;
  scrollbar-color: var(--line-2) transparent;
}
.nice-scroll::-webkit-scrollbar { width: 4px; }
.nice-scroll::-webkit-scrollbar-thumb {
  background: var(--line-2);
  border-radius: 2px;
}

/* Card hover */
.card-hover {
  transition: border-color 0.15s;
}
.card-hover:hover { border-color: var(--line-2); }

/* Seg control */
.seg-control {
  display: inline-flex;
  background: var(--bg-2);
  border: 1px solid var(--line);
  border-radius: 8px;
  padding: 3px;
  gap: 2px;
}
.seg-btn {
  font-size: 11px;
  font-weight: 500;
  padding: 4px 10px;
  border-radius: 6px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink-3);
  cursor: pointer;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
  line-height: 1;
}
.seg-btn.active {
  background: var(--bg-1);
  color: var(--ink);
  border-color: var(--line-2);
}
.seg-btn:not(.active):hover { color: var(--ink-2); }

/* Skeleton */
@keyframes shimmer {
  0%   { background-position: -200% 0; }
  100% { background-position:  200% 0; }
}
.skeleton {
  background: linear-gradient(90deg, var(--bg-2) 25%, var(--line) 50%, var(--bg-2) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: 6px;
}

/* Animations */
@keyframes fadeUp {
  from { opacity: 0; transform: translateY(10px); }
  to   { opacity: 1; transform: translateY(0); }
}
@keyframes pulseDot {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.35; }
}
@keyframes drawerIn {
  from { transform: translateX(100%); }
  to   { transform: translateX(0); }
}
@keyframes backdropIn {
  from { opacity: 0; }
  to   { opacity: 1; }
}
@keyframes toastIn {
  from { opacity: 0; transform: translateY(-8px); }
  to   { opacity: 1; transform: translateY(0); }
}
@keyframes waveScroll {
  from { transform: translateX(0); }
  to   { transform: translateX(400px); }
}

.anim-fade-up  { animation: fadeUp    0.3s ease both; }
.anim-drawer   { animation: drawerIn  0.25s ease both; }
.anim-backdrop { animation: backdropIn 0.2s ease both; }
.anim-toast    { animation: toastIn   0.2s ease both; }

.stagger-1 { animation-delay:   0ms; }
.stagger-2 { animation-delay:  40ms; }
.stagger-3 { animation-delay:  80ms; }
.stagger-4 { animation-delay: 120ms; }
.stagger-5 { animation-delay: 160ms; }
.stagger-6 { animation-delay: 200ms; }
```

- [ ] **Step 4: Verify Vite build compiles**

```bash
cd web && npm run build 2>&1 | tail -20
```

- [ ] **Step 5: Commit**

```bash
git add web/index.html web/src/app.css src/main.rs
git commit -m "feat: CSS design token system, Geist fonts, CSP update"
```

---

### Task 4: MiniChart.svelte (new)

**Files:**
- Create: `web/src/lib/components/MiniChart.svelte`

- [ ] **Step 1: Create `MiniChart.svelte`**

```svelte
<script lang="ts">
  let idSeq = 0;

  let { data, max, color, w = 140, h = 48 }: {
    data: number[];
    max: number;
    color: string;
    w?: number;
    h?: number;
  } = $props();

  const uid = `mc${idSeq++}`;

  let paths = $derived.by(() => {
    if (data.length < 2) return { line: '', area: '', dot: null as {x:number,y:number}|null };
    const cap = max > 0 ? max : 1;
    const step = w / (data.length - 1);
    const pts = data.map((v, i) => ({
      x: i * step,
      y: 2 + (1 - Math.min(v, cap) / cap) * (h - 4),
    }));
    const coords = pts.map(p => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' L ');
    return {
      line: `M ${coords}`,
      area: `M ${coords} L ${w},${h} L 0,${h} Z`,
      dot: pts[pts.length - 1],
    };
  });
</script>

<svg width={w} height={h} viewBox="0 0 {w} {h}" preserveAspectRatio="none" style="overflow:visible">
  <defs>
    <linearGradient id={uid} x1="0" x2="0" y1="0" y2="1">
      <stop offset="0%"   stop-color={color} stop-opacity="0.28" />
      <stop offset="100%" stop-color={color} stop-opacity="0.02" />
    </linearGradient>
  </defs>
  {#if paths.area}
    <path d={paths.area} fill="url(#{uid})" />
  {/if}
  {#if paths.line}
    <path d={paths.line} fill="none" stroke={color} stroke-width="1.5"
          vector-effect="non-scaling-stroke" />
  {/if}
  {#if paths.dot}
    <circle cx={paths.dot.x} cy={paths.dot.y} r="2.5" fill={color}
            style="animation: pulseDot 2s ease-in-out infinite" />
  {/if}
</svg>
```

- [ ] **Step 2: Build check**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
```

Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add web/src/lib/components/MiniChart.svelte && git commit -m "feat: add MiniChart reusable sparkline component"
```

---

### Task 5: CpuCard.svelte rewrite

**Files:**
- Modify: `web/src/lib/components/CpuCard.svelte`

- [ ] **Step 1: Rewrite CpuCard**

```svelte
<script lang="ts">
  import MiniChart from './MiniChart.svelte';

  let { usage, cores, history, load }: {
    usage: number;
    cores: number;
    history: number[];
    load: [number, number, number];
  } = $props();

  function sevColor(v: number): string {
    if (v >= 90) return 'var(--crit)';
    if (v >= 70) return 'var(--warm)';
    return 'var(--accent)';
  }

  const R = 26;
  const CIRC = 2 * Math.PI * R;
  let dashOffset = $derived(CIRC - (usage / 100) * CIRC);
  let accentColor = $derived(sevColor(usage));
</script>

<div class="surface card-hover h-full flex flex-col relative overflow-hidden" style="padding: 18px 20px">
  <!-- Top accent hairline -->
  <div class="absolute top-0 left-0 right-0 h-[2px] rounded-t-[14px]"
       style="background: linear-gradient(90deg, {accentColor}60, {accentColor}00)"></div>

  <!-- Header -->
  <div class="flex items-center justify-between mb-4">
    <span class="eyebrow">CPU</span>
    <span class="tnum font-mono text-[10px] px-2 py-0.5 rounded" style="background:var(--bg-2);color:var(--ink-4);border:1px solid var(--line)">{cores} cores</span>
  </div>

  <!-- Body -->
  <div class="flex items-center gap-4 flex-1 min-h-0">
    <!-- Ring gauge -->
    <div class="relative flex-shrink-0" style="width:64px;height:64px">
      <svg width="64" height="64" viewBox="0 0 64 64" style="transform:rotate(-90deg)">
        <circle cx="32" cy="32" r={R} fill="none" stroke="var(--line-2)" stroke-width="5" />
        <circle cx="32" cy="32" r={R} fill="none"
                stroke={accentColor}
                stroke-width="5"
                stroke-linecap="round"
                stroke-dasharray={CIRC}
                stroke-dashoffset={dashOffset}
                style="transition: stroke-dashoffset 0.6s ease, stroke 0.4s ease" />
      </svg>
      <div class="absolute inset-0 flex flex-col items-center justify-center">
        <span class="tnum font-mono text-base font-semibold" style="color:var(--ink);line-height:1">{usage.toFixed(0)}</span>
        <span style="font-size:9px;color:var(--ink-4)">%</span>
      </div>
    </div>

    <!-- Right: sparkline + load avg -->
    <div class="flex-1 flex flex-col gap-2 min-w-0">
      <div class="flex-1" style="min-height:40px">
        <MiniChart data={history} max={100} color={accentColor} w={140} h={44} />
      </div>
      {#if load[0] > 0}
        <div class="flex gap-3">
          {#each [['1m', load[0]], ['5m', load[1]], ['15m', load[2]]] as [label, val]}
            <div class="flex flex-col items-center">
              <span class="tnum font-mono text-[11px]" style="color:var(--ink-2)">{(val as number).toFixed(2)}</span>
              <span style="font-size:9px;color:var(--ink-4)">{label}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>
```

- [ ] **Step 2: Update CpuCard call in App.svelte**

In `App.svelte`, change:
```svelte
<CpuCard usage={metrics.cpu_usage} cores={metrics.cpu_cores} history={cpuHistory} />
```
to:
```svelte
<CpuCard usage={metrics.cpu_usage} cores={metrics.cpu_cores} history={cpuHistory} load={metrics.cpu_load} />
```

- [ ] **Step 3: Build check + commit**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
git add web/src/lib/components/CpuCard.svelte web/src/App.svelte
git commit -m "feat: rewrite CpuCard with ring gauge, MiniChart, load avg"
```

---

### Task 6: MemoryCard.svelte rewrite

**Files:**
- Modify: `web/src/lib/components/MemoryCard.svelte`

- [ ] **Step 1: Rewrite**

```svelte
<script lang="ts">
  let { used, total, swap_used, swap_total }: {
    used: number; total: number;
    swap_used: number; swap_total: number;
  } = $props();

  function sevColor(v: number): string {
    if (v >= 90) return 'var(--crit)';
    if (v >= 70) return 'var(--warm)';
    return 'var(--accent)';
  }

  function fmt(b: number): string {
    if (b >= 1073741824) return (b / 1073741824).toFixed(1) + ' GB';
    if (b >= 1048576) return (b / 1048576).toFixed(0) + ' MB';
    return (b / 1024).toFixed(0) + ' KB';
  }

  let pct = $derived(total > 0 ? (used / total) * 100 : 0);
  let swapPct = $derived(swap_total > 0 ? (swap_used / swap_total) * 100 : 0);
  let barColor = $derived(sevColor(pct));
  let usedW = $derived(total > 0 ? (used / total) * 100 : 0);
  let swapW = $derived(swap_total > 0 ? (swap_used / total) * 100 : 0);
</script>

<div class="surface card-hover h-full flex flex-col relative overflow-hidden" style="padding:18px 20px">
  <div class="absolute top-0 left-0 right-0 h-[2px] rounded-t-[14px]"
       style="background:linear-gradient(90deg,{barColor}60,{barColor}00)"></div>

  <div class="flex items-center justify-between mb-4">
    <span class="eyebrow">Memory</span>
    <span class="tnum font-mono text-[10px] px-2 py-0.5 rounded"
          style="background:var(--bg-2);color:var(--ink-4);border:1px solid var(--line)">Swap {fmt(swap_used)}</span>
  </div>

  <!-- Large % number -->
  <div class="flex items-end justify-between mb-4">
    <div class="flex items-baseline gap-1">
      <span class="tnum font-mono font-semibold" style="font-size:34px;color:var(--ink);line-height:1">{pct.toFixed(0)}</span>
      <span style="font-size:14px;color:var(--ink-3)">%</span>
    </div>
    <div class="text-right">
      <div class="tnum font-mono text-[11px]" style="color:var(--ink-2)">{fmt(used)}</div>
      <div class="tnum font-mono text-[11px]" style="color:var(--ink-4)">of {fmt(total)}</div>
    </div>
  </div>

  <!-- Stacked bar -->
  <div class="rounded-full overflow-hidden mb-3" style="height:6px;background:var(--bg-2);border:1px solid var(--line)">
    <div class="h-full flex">
      <div class="h-full rounded-l-full transition-all duration-700"
           style="width:{usedW}%;background:{barColor}"></div>
      <div class="h-full transition-all duration-700"
           style="width:{swapW}%;background:var(--warm);opacity:0.6"></div>
    </div>
  </div>

  <!-- Legend -->
  <div class="flex gap-4">
    {#each [
      { color: barColor, label: 'Used' },
      { color: 'var(--warm)', label: 'Swap', opacity: '0.7' },
      { color: 'var(--bg-2)', label: 'Free', border: true },
    ] as item}
      <div class="flex items-center gap-1.5">
        <div class="rounded-sm" style="width:10px;height:6px;background:{item.color};{item.opacity ? 'opacity:'+item.opacity : ''};{item.border ? 'border:1px solid var(--line-2)' : ''}"></div>
        <span style="font-size:10px;color:var(--ink-3)">{item.label}</span>
      </div>
    {/each}
  </div>
</div>
```

- [ ] **Step 2: Build + commit**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
git add web/src/lib/components/MemoryCard.svelte && git commit -m "feat: rewrite MemoryCard"
```

---

### Task 7: NetworkCard.svelte rewrite

**Files:**
- Modify: `web/src/lib/components/NetworkCard.svelte`

- [ ] **Step 1: Rewrite**

```svelte
<script lang="ts">
  import MiniChart from './MiniChart.svelte';

  let { tx, rx, txHistory, rxHistory }: {
    tx: number; rx: number;
    txHistory: number[]; rxHistory: number[];
  } = $props();

  function fmtRate(bps: number): string {
    if (bps >= 1048576) return (bps / 1048576).toFixed(1) + ' MB/s';
    if (bps >= 1024) return (bps / 1024).toFixed(0) + ' KB/s';
    return bps.toFixed(0) + ' B/s';
  }

  let txMax = $derived(Math.max(...txHistory, 1));
  let rxMax = $derived(Math.max(...rxHistory, 1));
</script>

<div class="surface card-hover h-full flex flex-col" style="padding:18px 20px">
  <div class="flex items-center justify-between mb-4">
    <span class="eyebrow">Network</span>
  </div>

  <div class="flex flex-col gap-3 flex-1">
    {#each [
      { label: 'Out ↑', value: tx, history: txHistory, max: txMax, color: 'var(--accent)' },
      { label: 'In ↓',  value: rx, history: rxHistory, max: rxMax, color: 'var(--warm)' },
    ] as row}
      <div class="surface-2 flex items-center gap-3" style="padding:10px 12px;flex:1">
        <div style="min-width:48px">
          <div class="eyebrow" style="margin-bottom:2px">{row.label}</div>
          <div class="tnum font-mono font-medium text-[13px]" style="color:var(--ink)">{fmtRate(row.value)}</div>
        </div>
        <div class="flex-1" style="min-height:36px">
          <MiniChart data={row.history} max={row.max} color={row.color} w={120} h={36} />
        </div>
      </div>
    {/each}
  </div>
</div>
```

- [ ] **Step 2: Build + commit**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
git add web/src/lib/components/NetworkCard.svelte && git commit -m "feat: rewrite NetworkCard"
```

---

### Task 8: DiskCard.svelte rewrite

**Files:**
- Modify: `web/src/lib/components/DiskCard.svelte`

- [ ] **Step 1: Read current DiskCard**

Read `web/src/lib/components/DiskCard.svelte` to understand props and current logic.

- [ ] **Step 2: Rewrite**

```svelte
<script lang="ts">
  import type { DiskInfo } from '../types';

  let { disks }: { disks: DiskInfo[] } = $props();

  function sevColor(v: number): string {
    if (v >= 90) return 'var(--crit)';
    if (v >= 70) return 'var(--warm)';
    return 'var(--accent)';
  }

  function fmt(b: number): string {
    if (b >= 1099511627776) return (b / 1099511627776).toFixed(1) + ' TB';
    if (b >= 1073741824) return (b / 1073741824).toFixed(1) + ' GB';
    if (b >= 1048576) return (b / 1048576).toFixed(0) + ' MB';
    return (b / 1024).toFixed(0) + ' KB';
  }

  function fmtRate(b: number): string {
    if (b >= 1048576) return (b / 1048576).toFixed(1) + ' MB/s';
    if (b >= 1024) return (b / 1024).toFixed(0) + ' KB/s';
    return b + ' B/s';
  }

  let primary = $derived(disks[0] ?? null);
  let usedPct = $derived(primary && primary.total_space > 0
    ? ((primary.total_space - primary.available_space) / primary.total_space) * 100 : 0);
  let totalUsed = $derived(primary ? primary.total_space - primary.available_space : 0);
</script>

<div class="surface card-hover h-full flex flex-col relative overflow-hidden" style="padding:18px 20px">
  {#if primary}
    {@const barColor = sevColor(usedPct)}
    <div class="absolute top-0 left-0 right-0 h-[2px] rounded-t-[14px]"
         style="background:linear-gradient(90deg,{barColor}60,{barColor}00)"></div>
  {/if}

  <div class="flex items-center justify-between mb-4">
    <span class="eyebrow">Storage</span>
    <span class="tnum font-mono text-[10px] px-2 py-0.5 rounded"
          style="background:var(--bg-2);color:var(--ink-4);border:1px solid var(--line)">{disks.length} mount{disks.length !== 1 ? 's' : ''}</span>
  </div>

  {#if primary}
    {@const barColor = sevColor(usedPct)}
    <div class="flex items-end justify-between mb-3">
      <div class="flex items-baseline gap-1">
        <span class="tnum font-mono font-semibold" style="font-size:34px;color:var(--ink);line-height:1">{usedPct.toFixed(0)}</span>
        <span style="font-size:14px;color:var(--ink-3)">%</span>
      </div>
      <div class="text-right">
        <div class="tnum font-mono text-[11px]" style="color:var(--ink-2)">{fmt(totalUsed)}</div>
        <div class="tnum font-mono text-[11px]" style="color:var(--ink-4)">of {fmt(primary.total_space)}</div>
      </div>
    </div>

    <!-- Per-mount bar -->
    <div class="rounded-full overflow-hidden mb-3" style="height:6px;background:var(--bg-2);border:1px solid var(--line);display:flex">
      {#each disks as disk, i}
        {@const pct = disk.total_space > 0 ? ((disk.total_space - disk.available_space) / primary.total_space) * 100 : 0}
        <div class="h-full transition-all duration-700"
             style="width:{pct}%;background:{i === 0 ? sevColor(usedPct) : 'var(--ink-3)'};opacity:{i === 0 ? 1 : 0.4}"></div>
      {/each}
    </div>

    <!-- I/O footer -->
    <div class="flex items-center gap-4 mt-auto">
      <div class="flex items-center gap-1.5">
        <span style="font-size:10px;color:var(--accent)">↓</span>
        <span class="tnum font-mono text-[11px]" style="color:var(--ink-2)">{fmtRate(primary.read_bytes)}</span>
      </div>
      <div class="flex items-center gap-1.5">
        <span style="font-size:10px;color:var(--warm)">↑</span>
        <span class="tnum font-mono text-[11px]" style="color:var(--ink-2)">{fmtRate(primary.written_bytes)}</span>
      </div>
      <span class="font-mono text-[10px] truncate ml-auto" style="color:var(--ink-4)">{primary.mount_point}</span>
    </div>
  {:else}
    <div class="flex-1 flex items-center justify-center" style="color:var(--ink-4);font-size:12px">No disks detected</div>
  {/if}
</div>
```

- [ ] **Step 3: Build + commit**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
git add web/src/lib/components/DiskCard.svelte && git commit -m "feat: rewrite DiskCard"
```

---

### Task 9: TopBar.svelte rewrite

**Files:**
- Modify: `web/src/lib/components/TopBar.svelte`

- [ ] **Step 1: Rewrite TopBar — remove embedded Settings, add onShowSettings prop**

```svelte
<script lang="ts">
  import type { SystemMetrics, AlertEvent } from '../types';

  let { metrics, refreshRate, alerts, onShowSettings, onLogout, onShowAlertHistory }: {
    metrics: SystemMetrics | null;
    refreshRate: number;
    alerts: AlertEvent[];
    onShowSettings: () => void;
    onLogout: () => void;
    onShowAlertHistory: () => void;
  } = $props();

  let alertCount = $derived(alerts.length);

  function fmtUptime(s: number): string {
    const d = Math.floor(s / 86400);
    const h = Math.floor((s % 86400) / 3600);
    const m = Math.floor((s % 3600) / 60);
    const parts: string[] = [];
    if (d > 0) parts.push(`${d}d`);
    parts.push(`${h}h`);
    parts.push(`${m}m`);
    return parts.join(' ');
  }
</script>

<header class="surface anim-fade-up mb-6" style="border-radius:14px">
  <div class="flex items-center justify-between" style="padding:14px 20px">

    <!-- Left: Sigil + name + host -->
    <div class="flex items-center gap-3">
      <!-- Sigil -->
      <div class="flex items-center justify-center rounded-[10px]"
           style="width:36px;height:36px;background:var(--bg-2);border:1px solid var(--line-2);flex-shrink:0">
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
          <circle cx="10" cy="10" r="2.5" stroke="var(--accent)" stroke-width="1.5"/>
          <circle cx="10" cy="10" r="6"   stroke="var(--accent)" stroke-width="1" stroke-opacity="0.5"/>
          <circle cx="10" cy="10" r="9"   stroke="var(--accent)" stroke-width="0.75" stroke-opacity="0.25"/>
        </svg>
      </div>

      <div>
        <div class="flex items-center gap-2">
          <span style="font-size:15px;font-weight:600;color:var(--ink)">Astral</span>
          <span class="tnum font-mono" style="font-size:10px;color:var(--ink-4)">v1.1.0</span>
        </div>
        {#if metrics}
          <div class="flex items-center gap-1.5 mt-0.5">
            <span class="font-mono" style="font-size:12px;color:var(--ink-2)">{metrics.hostname}</span>
            <span style="color:var(--ink-4)">·</span>
            <span style="font-size:12px;color:var(--ink-3)">{metrics.os_name} {metrics.os_version}</span>
          </div>
        {/if}
      </div>
    </div>

    <!-- Right: pills + icon buttons -->
    <div class="flex items-center gap-2">
      {#if metrics}
        <!-- Live / Alert pill -->
        {#if alertCount > 0}
          <button onclick={onShowAlertHistory}
                  class="flex items-center gap-1.5 px-3 py-1.5 rounded-full cursor-pointer"
                  style="background:var(--crit-soft);border:1px solid rgba(244,63,94,0.25)">
            <div style="width:6px;height:6px;border-radius:50%;background:var(--crit);animation:pulseDot 1.5s infinite"></div>
            <span style="font-size:11px;font-weight:500;color:var(--crit)">{alertCount} alert{alertCount !== 1 ? 's' : ''}</span>
          </button>
        {:else}
          <div class="flex items-center gap-1.5 px-3 py-1.5 rounded-full"
               style="background:var(--bg-2);border:1px solid var(--line)">
            <div style="width:6px;height:6px;border-radius:50%;background:var(--ok);animation:pulseDot 2s infinite"></div>
            <span style="font-size:11px;font-weight:500;color:var(--ok)">Live · {refreshRate}s</span>
          </div>
        {/if}

        <!-- Uptime pill -->
        <div class="flex items-center gap-1.5 px-3 py-1.5 rounded-full"
             style="background:var(--bg-2);border:1px solid var(--line)">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" stroke-width="2">
            <circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>
          </svg>
          <span class="tnum font-mono" style="font-size:11px;color:var(--ink-2)">{fmtUptime(metrics.uptime)}</span>
        </div>
      {/if}

      <!-- Bell -->
      <button onclick={onShowAlertHistory} aria-label="Alert history"
              class="btn btn-ghost flex items-center justify-center"
              style="width:32px;height:32px;padding:0;border-radius:8px;border:1px solid var(--line)">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" stroke-width="1.5">
          <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 01-3.46 0"/>
        </svg>
      </button>

      <!-- Gear -->
      <button onclick={onShowSettings} aria-label="Settings"
              class="btn btn-ghost flex items-center justify-center"
              style="width:32px;height:32px;padding:0;border-radius:8px;border:1px solid var(--line)">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" stroke-width="1.5">
          <circle cx="12" cy="12" r="3"/>
          <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/>
        </svg>
      </button>

      <!-- Logout -->
      <button onclick={onLogout} aria-label="Logout"
              class="btn btn-ghost flex items-center justify-center"
              style="width:32px;height:32px;padding:0;border-radius:8px;border:1px solid var(--line)">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" stroke-width="1.5">
          <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4"/><polyline points="16 17 21 12 16 7"/>
          <line x1="21" y1="12" x2="9" y2="12"/>
        </svg>
      </button>
    </div>
  </div>
</header>
```

- [ ] **Step 2: Build + commit**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
git add web/src/lib/components/TopBar.svelte && git commit -m "feat: rewrite TopBar with Sigil, pills, settings prop"
```

---

### Task 10: HistoryChart.svelte rewrite (most complex)

**Files:**
- Modify: `web/src/lib/components/HistoryChart.svelte`

- [ ] **Step 1: Rewrite HistoryChart — remove uPlot, custom SVG renderer, 4 metric tabs**

Key prop: `totalMemory: number` (passed from App.svelte, needed for memory % calc).

Full implementation:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import type { MetricPoint } from '../types';

  let { totalMemory }: { totalMemory: number } = $props();

  type MetricKey = 'cpu' | 'mem' | 'net' | 'disk';
  type WindowKey = '6h' | '24h' | '7d' | 'all';

  let metricKey = $state<MetricKey>('cpu');
  let windowKey = $state<WindowKey>('6h');
  let data: MetricPoint[] = $state([]);
  let isLoading = $state(true);
  let containerEl: HTMLDivElement;
  let containerW = $state(600);
  let containerH = $state(200);
  let interval: ReturnType<typeof setInterval>;

  const METRICS: Record<MetricKey, { label: string; color: string; unit: string; fixedMax: number | null; fmt: (v: number) => string }> = {
    cpu:  { label: 'CPU',     color: 'var(--accent)', unit: '%',    fixedMax: 100,  fmt: v => v.toFixed(1) + '%' },
    mem:  { label: 'Memory',  color: 'var(--warm)',   unit: '%',    fixedMax: 100,  fmt: v => v.toFixed(1) + '%' },
    net:  { label: 'Network', color: 'var(--accent)', unit: 'MB/s', fixedMax: null, fmt: v => v.toFixed(2) + ' MB/s' },
    disk: { label: 'Disk I/O',color: 'var(--warm)',   unit: 'MB/s', fixedMax: null, fmt: v => v.toFixed(2) + ' MB/s' },
  };

  const WINDOWS: { id: WindowKey; label: string }[] = [
    { id: '6h', label: '6H' }, { id: '24h', label: '24H' },
    { id: '7d', label: '7D' }, { id: 'all', label: 'All' },
  ];

  function extractValue(p: MetricPoint, key: MetricKey): number {
    switch (key) {
      case 'cpu':  return p.cpu_usage;
      case 'mem':  return totalMemory > 0 ? (p.used_memory / totalMemory) * 100 : 0;
      case 'net':  return (p.network_tx + p.network_rx) / 1e6;
      case 'disk': return (p.disk_read_rate + p.disk_write_rate) / 1e6;
    }
  }

  let values = $derived(data.map(p => extractValue(p, metricKey)));
  let nowVal  = $derived(values.length ? values[values.length - 1] : 0);
  let avgVal  = $derived(values.length ? values.reduce((a, b) => a + b, 0) / values.length : 0);
  let peakVal = $derived(values.length ? Math.max(...values) : 0);

  const PAD_L = 52, PAD_R = 12, PAD_T = 14, PAD_B = 26;

  let plotW = $derived(Math.max(containerW - PAD_L - PAD_R, 1));
  let plotH = $derived(Math.max(containerH - PAD_T - PAD_B, 1));

  let maxVal = $derived.by(() => {
    const cfg = METRICS[metricKey];
    if (cfg.fixedMax !== null) return cfg.fixedMax;
    const m = Math.max(...values, 0.001);
    const mag = Math.pow(10, Math.floor(Math.log10(m)));
    return Math.ceil(m / mag) * mag;
  });

  let gridLines = $derived.by(() => {
    return Array.from({ length: 5 }, (_, i) => {
      const frac = i / 4;
      const y = PAD_T + frac * plotH;
      const val = maxVal * (1 - frac);
      return { y, val };
    });
  });

  let svgPath = $derived.by(() => {
    if (values.length < 2) return { line: '', area: '', dotX: 0, dotY: 0 };
    const n = values.length;
    const pts = values.map((v, i) => ({
      x: PAD_L + (i / (n - 1)) * plotW,
      y: PAD_T + (1 - Math.min(v, maxVal) / maxVal) * plotH,
    }));
    const coords = pts.map(p => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' L ');
    const last = pts[pts.length - 1];
    return {
      line: `M ${coords}`,
      area: `M ${coords} L ${(PAD_L + plotW).toFixed(1)},${(PAD_T + plotH).toFixed(1)} L ${PAD_L},${(PAD_T + plotH).toFixed(1)} Z`,
      dotX: last.x,
      dotY: last.y,
    };
  });

  let xLabels = $derived.by(() => {
    if (data.length < 2) return [];
    const n = data.length;
    return Array.from({ length: 5 }, (_, i) => {
      const idx = Math.round(i * (n - 1) / 4);
      const ts = data[idx].timestamp;
      const d = new Date(ts * 1000);
      const label = windowKey === '7d'
        ? d.toLocaleDateString([], { weekday: 'short' })
        : d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      return { x: PAD_L + (idx / (n - 1)) * plotW, label };
    });
  });

  async function fetchData() {
    isLoading = true;
    try {
      const res = await fetch(`/api/history?window=${windowKey}`);
      if (res.ok) data = await res.json();
    } catch { /**/ } finally {
      isLoading = false;
    }
  }

  function fmtStat(v: number): string {
    return METRICS[metricKey].fmt(v);
  }

  onMount(() => {
    const ro = new ResizeObserver(entries => {
      for (const e of entries) {
        containerW = e.contentRect.width;
        containerH = e.contentRect.height;
      }
    });
    if (containerEl) ro.observe(containerEl);
    fetchData();
    interval = setInterval(fetchData, 60000);
    return () => { clearInterval(interval); ro.disconnect(); };
  });

  $effect(() => {
    metricKey; windowKey; fetchData();
  });
</script>

<div class="surface h-full flex flex-col" style="padding:18px 20px">
  <!-- Header -->
  <div class="flex flex-wrap items-center gap-3 mb-4 flex-shrink-0" style="min-height:32px">
    <span class="eyebrow">History</span>

    <!-- Metric tabs -->
    <div class="seg-control">
      {#each Object.entries(METRICS) as [key, cfg]}
        <button class="seg-btn {metricKey === key ? 'active' : ''}"
                onclick={() => metricKey = key as MetricKey}>{cfg.label}</button>
      {/each}
    </div>

    <!-- Stats -->
    {#if values.length > 0}
      <div class="flex items-center gap-4 ml-2">
        {#each [['now', nowVal], ['avg', avgVal], ['peak', peakVal]] as [lbl, val]}
          <div class="flex flex-col">
            <span class="eyebrow" style="font-size:9px">{lbl}</span>
            <span class="tnum font-mono text-[12px]" style="color:var(--ink)">{fmtStat(val as number)}</span>
          </div>
        {/each}
      </div>
    {/if}

    <!-- Window tabs -->
    <div class="seg-control ml-auto">
      {#each WINDOWS as w}
        <button class="seg-btn {windowKey === w.id ? 'active' : ''}"
                onclick={() => windowKey = w.id as WindowKey}>{w.label}</button>
      {/each}
    </div>
  </div>

  <!-- Chart area -->
  <div class="flex-1 min-h-0 relative" bind:this={containerEl}
       bind:clientWidth={containerW} bind:clientHeight={containerH}>
    {#if isLoading}
      <div class="absolute inset-0 flex items-center justify-center"
           style="color:var(--ink-4);font-size:12px">Loading…</div>
    {:else if data.length < 2}
      <div class="absolute inset-0 flex items-center justify-center"
           style="color:var(--ink-4);font-size:12px">No data for this period</div>
    {:else}
      <svg width={containerW} height={containerH} style="display:block">
        <defs>
          <linearGradient id="chart-grad" x1="0" x2="0" y1="0" y2="1">
            <stop offset="0%"   stop-color={METRICS[metricKey].color} stop-opacity="0.25"/>
            <stop offset="100%" stop-color={METRICS[metricKey].color} stop-opacity="0.02"/>
          </linearGradient>
        </defs>

        <!-- Dotted grid background -->
        <pattern id="dots" x={PAD_L} y={PAD_T} width="16" height="16" patternUnits="userSpaceOnUse">
          <circle cx="1" cy="1" r="0.8" fill="var(--line)"/>
        </pattern>
        <rect x={PAD_L} y={PAD_T} width={plotW} height={plotH} fill="url(#dots)" opacity="0.6"/>

        <!-- Gridlines + Y labels -->
        {#each gridLines as gl}
          <line x1={PAD_L} y1={gl.y} x2={PAD_L + plotW} y2={gl.y}
                stroke="var(--line)" stroke-width="1"/>
          <text x={PAD_L - 6} y={gl.y + 3.5} text-anchor="end"
                font-family="'Geist Mono', monospace" font-size="9"
                fill="var(--ink-4)">{METRICS[metricKey].fmt(gl.val)}</text>
        {/each}

        <!-- Area fill -->
        <path d={svgPath.area} fill="url(#chart-grad)"/>

        <!-- Line -->
        <path d={svgPath.line} fill="none"
              stroke={METRICS[metricKey].color} stroke-width="1.5"
              stroke-linecap="round" stroke-linejoin="round"/>

        <!-- Leading dot -->
        <circle cx={svgPath.dotX} cy={svgPath.dotY} r="3"
                fill={METRICS[metricKey].color}
                style="animation: pulseDot 2s ease-in-out infinite"/>

        <!-- X-axis labels -->
        {#each xLabels as xl}
          <text x={xl.x} y={PAD_T + plotH + 18} text-anchor="middle"
                font-family="'Geist Mono', monospace" font-size="9"
                fill="var(--ink-4)">{xl.label}</text>
        {/each}
      </svg>
    {/if}
  </div>
</div>
```

- [ ] **Step 2: Update HistoryChart call in App.svelte**

Change:
```svelte
<HistoryChart />
```
to:
```svelte
<HistoryChart totalMemory={metrics?.total_memory ?? 0} />
```

- [ ] **Step 3: Build + commit**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
git add web/src/lib/components/HistoryChart.svelte web/src/App.svelte
git commit -m "feat: rewrite HistoryChart with custom SVG, 4 metric tabs"
```

---

### Task 11: ProcessList.svelte rewrite

**Files:**
- Modify: `web/src/lib/components/ProcessList.svelte`

- [ ] **Step 1: Rewrite**

```svelte
<script lang="ts">
  import type { ProcessInfo } from '../types';

  let { processes, totalMemory }: { processes: ProcessInfo[]; totalMemory: number } = $props();

  let sortBy = $state<'cpu' | 'mem'>('cpu');

  let sorted = $derived(
    [...processes]
      .sort((a, b) => sortBy === 'cpu' ? b.cpu_usage - a.cpu_usage : b.memory - a.memory)
      .slice(0, 8)
  );

  let maxCpu = $derived(Math.max(...sorted.map(p => p.cpu_usage), 1));
  let maxMem = $derived(Math.max(...sorted.map(p => p.memory), 1));

  function fmtMem(b: number): string {
    if (b >= 1073741824) return (b / 1073741824).toFixed(1) + 'G';
    if (b >= 1048576) return (b / 1048576).toFixed(0) + 'M';
    return (b / 1024).toFixed(0) + 'K';
  }
</script>

<div class="surface h-full flex flex-col nice-scroll" style="padding:18px 20px">
  <!-- Header -->
  <div class="flex items-center justify-between mb-4 flex-shrink-0">
    <span class="eyebrow">Top Processes</span>
    {#if processes.length > 0}
      <div class="seg-control">
        <button class="seg-btn {sortBy === 'cpu' ? 'active' : ''}" onclick={() => sortBy = 'cpu'}>CPU</button>
        <button class="seg-btn {sortBy === 'mem' ? 'active' : ''}" onclick={() => sortBy = 'mem'}>Mem</button>
      </div>
    {/if}
  </div>

  {#if processes.length === 0}
    <div class="flex-1 flex flex-col items-center justify-center gap-3 text-center">
      <div style="width:40px;height:40px;border-radius:10px;background:var(--bg-2);border:1px solid var(--line);display:flex;align-items:center;justify-content:center">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--ink-4)" stroke-width="1.5">
          <rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/>
        </svg>
      </div>
      <div>
        <div style="font-size:12px;color:var(--ink-3);font-weight:500;margin-bottom:4px">Process monitoring is off</div>
        <div style="font-size:11px;color:var(--ink-4)">Enable in Settings → Processes</div>
      </div>
    </div>
  {:else}
    <!-- Column headers -->
    <div style="display:grid;grid-template-columns:1fr 60px 80px 70px;gap:8px;padding:4px 6px;margin-bottom:4px">
      <span class="eyebrow" style="font-size:9px">Process</span>
      <span class="eyebrow" style="font-size:9px;text-align:right">PID</span>
      <span class="eyebrow" style="font-size:9px;text-align:right">CPU</span>
      <span class="eyebrow" style="font-size:9px;text-align:right">Memory</span>
    </div>

    <!-- Rows -->
    <div class="flex-1 min-h-0 overflow-y-auto nice-scroll">
      {#each sorted as proc, i}
        <div style="display:grid;grid-template-columns:1fr 60px 80px 70px;gap:8px;padding:5px 6px;border-radius:6px;align-items:center"
             class="card-hover">
          <!-- Name -->
          <div class="flex items-center gap-2 min-w-0">
            <span class="tnum font-mono" style="font-size:10px;color:var(--ink-4);width:16px;flex-shrink:0">{i+1}</span>
            <span style="font-size:12px;color:var(--ink-2);overflow:hidden;text-overflow:ellipsis;white-space:nowrap" title="{proc.name}">{proc.name}</span>
          </div>
          <!-- PID -->
          <div class="tnum font-mono text-right" style="font-size:11px;color:var(--ink-4)">{proc.pid}</div>
          <!-- CPU -->
          <div class="flex flex-col items-end gap-0.5">
            <span class="tnum font-mono" style="font-size:11px;color:{proc.cpu_usage >= 50 ? 'var(--crit)' : proc.cpu_usage >= 20 ? 'var(--warm)' : 'var(--ink-2)'}">{proc.cpu_usage.toFixed(1)}%</span>
            <div style="width:100%;height:2px;background:var(--bg-2);border-radius:1px;overflow:hidden">
              <div style="width:{(proc.cpu_usage / maxCpu) * 100}%;height:100%;background:var(--accent);border-radius:1px"></div>
            </div>
          </div>
          <!-- Memory -->
          <div class="flex flex-col items-end gap-0.5">
            <span class="tnum font-mono" style="font-size:11px;color:var(--ink-2)">{fmtMem(proc.memory)}</span>
            <div style="width:100%;height:2px;background:var(--bg-2);border-radius:1px;overflow:hidden">
              <div style="width:{(proc.memory / maxMem) * 100}%;height:100%;background:var(--warm);border-radius:1px"></div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
```

- [ ] **Step 2: Build + commit**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
git add web/src/lib/components/ProcessList.svelte && git commit -m "feat: rewrite ProcessList with mem sort, mini bars"
```

---

### Task 12: Login.svelte rewrite (split layout)

**Files:**
- Modify: `web/src/lib/components/Login.svelte`

- [ ] **Step 1: Rewrite with split layout + ambient panel**

```svelte
<script lang="ts">
  let { onLogin }: { onLogin: () => void } = $props();

  let username = $state('');
  let password = $state('');
  let error    = $state('');
  let loading  = $state(false);

  async function submit(e: Event) {
    e.preventDefault();
    error = ''; loading = true;
    try {
      const res = await fetch('/api/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      });
      if (res.ok) onLogin();
      else error = 'Invalid credentials';
    } catch { error = 'Connection failed'; }
    finally { loading = false; }
  }
</script>

<div style="min-height:100vh;display:grid;grid-template-columns:1fr 1fr;background:var(--bg)">
  <!-- Left: form -->
  <div class="flex items-center justify-center" style="padding:48px">
    <div style="width:100%;max-width:360px" class="anim-fade-up">
      <!-- Sigil + name -->
      <div class="flex items-center gap-3 mb-10">
        <div class="flex items-center justify-center rounded-[10px]"
             style="width:40px;height:40px;background:var(--bg-2);border:1px solid var(--line-2)">
          <svg width="22" height="22" viewBox="0 0 20 20" fill="none">
            <circle cx="10" cy="10" r="2.5" stroke="var(--accent)" stroke-width="1.5"/>
            <circle cx="10" cy="10" r="6"   stroke="var(--accent)" stroke-width="1" stroke-opacity="0.5"/>
            <circle cx="10" cy="10" r="9"   stroke="var(--accent)" stroke-width="0.75" stroke-opacity="0.25"/>
          </svg>
        </div>
        <span style="font-size:18px;font-weight:600;color:var(--ink)">Astral</span>
      </div>

      <h1 style="font-size:26px;font-weight:600;color:var(--ink);margin-bottom:8px">Welcome back.</h1>
      <p style="font-size:13px;color:var(--ink-3);margin-bottom:32px">Sign in to your monitoring dashboard</p>

      <form onsubmit={submit} style="display:flex;flex-direction:column;gap:16px">
        {#if error}
          <div style="font-size:12px;color:var(--crit);background:var(--crit-soft);border:1px solid rgba(244,63,94,0.25);border-radius:8px;padding:8px 12px">
            {error}
          </div>
        {/if}

        {#each [['username','Username','text'], ['password','Password','password']] as [id, label, type]}
          <div>
            <label for={id} class="eyebrow" style="display:block;margin-bottom:6px">{label}</label>
            {#if id === 'username'}
              <input {id} type={type} bind:value={username} autocomplete="username" required
                     class="focus-ring"
                     style="width:100%;background:var(--bg-2);border:1px solid var(--line);border-radius:8px;padding:10px 14px;font-size:13px;color:var(--ink);outline:none;box-sizing:border-box;transition:border-color 0.15s"
                     placeholder="admin" />
            {:else}
              <input {id} type={type} bind:value={password} autocomplete="current-password" required
                     class="focus-ring"
                     style="width:100%;background:var(--bg-2);border:1px solid var(--line);border-radius:8px;padding:10px 14px;font-size:13px;color:var(--ink);outline:none;box-sizing:border-box;transition:border-color 0.15s"
                     placeholder="••••••••" />
            {/if}
          </div>
        {/each}

        <button type="submit" disabled={loading}
                style="width:100%;background:var(--accent);color:#0b0d12;border:none;border-radius:8px;padding:11px;font-size:13px;font-weight:600;cursor:pointer;transition:background 0.15s;margin-top:4px"
                onmouseover={e => (e.currentTarget as HTMLElement).style.background = '#7dd3fc'}
                onfocus={e => (e.currentTarget as HTMLElement).style.background = '#7dd3fc'}
                onmouseout={e => (e.currentTarget as HTMLElement).style.background = 'var(--accent)'}
                onblur={e => (e.currentTarget as HTMLElement).style.background = 'var(--accent)'}>
          {loading ? 'Signing in…' : 'Sign in'}
        </button>
      </form>

      <p style="font-size:11px;color:var(--ink-4);margin-top:24px;text-align:center">
        Connection encrypted · session expires in 24h
      </p>
    </div>
  </div>

  <!-- Right: ambient panel -->
  <div class="relative overflow-hidden" style="background:var(--bg-1);border-left:1px solid var(--line)">
    <svg class="absolute inset-0" width="100%" height="100%" viewBox="0 0 600 800" preserveAspectRatio="xMidYMid slice">
      <!-- Dot grid -->
      <defs>
        <pattern id="lgrid" x="0" y="0" width="20" height="20" patternUnits="userSpaceOnUse">
          <circle cx="1" cy="1" r="0.8" fill="var(--line)"/>
        </pattern>
      </defs>
      <rect width="600" height="800" fill="url(#lgrid)"/>

      <!-- Gradient blobs -->
      <ellipse cx="450" cy="200" rx="280" ry="280" fill="rgba(56,189,248,0.06)"/>
      <ellipse cx="150" cy="600" rx="250" ry="250" fill="rgba(245,158,11,0.05)"/>

      <!-- Wave 1 — scrolling sine (accent) -->
      <g style="animation: waveScroll 8s linear infinite">
        <path d="M-600,300 Q-450,270 -300,300 Q-150,330 0,300 Q150,270 300,300 Q450,330 600,300 Q750,270 900,300 Q1050,330 1200,300"
              fill="none" stroke="rgba(56,189,248,0.2)" stroke-width="1.5"/>
      </g>
      <!-- Wave 2 — scrolling sine (warm) -->
      <g style="animation: waveScroll 12s linear infinite reverse">
        <path d="M-600,400 Q-450,378 -300,400 Q-150,422 0,400 Q150,378 300,400 Q450,422 600,400 Q750,378 900,400 Q1050,422 1200,400"
              fill="none" stroke="rgba(245,158,11,0.15)" stroke-width="1"/>
      </g>
    </svg>

    <!-- Floating metric chips -->
    <div class="absolute" style="top:30%;left:50%;transform:translateX(-50%);display:flex;flex-direction:column;gap:12px;align-items:center">
      <div class="surface tnum font-mono" style="padding:10px 18px;font-size:13px;color:var(--accent)">CPU 24.3%</div>
      <div class="surface tnum font-mono" style="padding:10px 18px;font-size:13px;color:var(--warm)">MEM 6.1 GB</div>
    </div>
  </div>
</div>
```

- [ ] **Step 2: Build + commit**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
git add web/src/lib/components/Login.svelte && git commit -m "feat: rewrite Login with split layout, ambient panel"
```

---

### Task 13: Settings.svelte rewrite (standalone drawer)

**Files:**
- Modify: `web/src/lib/components/Settings.svelte`

- [ ] **Step 1: Rewrite as standalone drawer component**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import type { DynamicConfig } from '../types';

  let { open, onClose, refreshRate, onRefreshRateChange, onLogout }: {
    open: boolean;
    onClose: () => void;
    refreshRate: number;
    onRefreshRateChange: (r: number) => void;
    onLogout: () => void;
  } = $props();

  let saving = $state(false);
  let config: DynamicConfig = $state({
    enable_process_list: false, alert_cpu: 90, alert_ram: 90,
    retention_days: 7, slack_webhook: null,
  });
  let webhookInput = $state('');
  let revoking = $state(false);

  const RATES  = [1, 2, 5, 10];
  const RETAIN = [7, 30, 90];

  async function load() {
    try {
      const r = await fetch('/api/settings');
      if (r.ok) { config = await r.json(); webhookInput = config.slack_webhook ?? ''; }
    } catch {/**/}
  }

  async function save(patch: Partial<DynamicConfig> & { slack_webhook?: string }) {
    saving = true;
    try {
      await fetch('/api/settings', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(patch),
      });
      await load();
    } finally { saving = false; }
  }

  async function revokeAll() {
    revoking = true;
    try {
      await fetch('/api/sessions/revoke', { method: 'POST' });
      onLogout();
    } finally { revoking = false; }
  }

  $effect(() => { if (open) load(); });

  function section(title: string) {
    return title;
  }
</script>

{#if open}
  <!-- Backdrop -->
  <div class="anim-backdrop" onclick={onClose} aria-label="Close settings"
       style="position:fixed;inset:0;z-index:40;background:rgba(0,0,0,0.5);cursor:default"></div>

  <!-- Drawer -->
  <div class="anim-drawer nice-scroll" role="dialog" aria-label="Settings"
       style="position:fixed;right:0;top:0;bottom:0;z-index:50;width:440px;max-width:100vw;
              background:var(--bg-1);border-left:1px solid var(--line);
              display:flex;flex-direction:column;overflow-y:auto">

    <!-- Header -->
    <div style="display:flex;align-items:center;justify-content:space-between;padding:20px 24px;border-bottom:1px solid var(--line);flex-shrink:0">
      <span style="font-size:15px;font-weight:600;color:var(--ink)">Settings</span>
      <button onclick={onClose} class="btn btn-ghost" style="padding:6px;border-radius:6px;border:1px solid var(--line)">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <!-- Body -->
    <div style="flex:1;padding:24px;display:flex;flex-direction:column;gap:28px">

      <!-- General -->
      <section>
        <div class="eyebrow" style="margin-bottom:14px">General</div>
        <div style="display:flex;flex-direction:column;gap:12px">
          <div>
            <div style="font-size:12px;color:var(--ink-2);margin-bottom:8px">Refresh rate</div>
            <div class="seg-control">
              {#each RATES as r}
                <button class="seg-btn {refreshRate === r ? 'active' : ''}"
                        onclick={() => onRefreshRateChange(r)}>{r}s</button>
              {/each}
            </div>
          </div>
          <div class="surface-2" style="padding:10px 14px;display:flex;align-items:center;justify-content:space-between;cursor:pointer"
               onclick={() => save({ enable_process_list: !config.enable_process_list })}>
            <span style="font-size:13px;color:var(--ink-2)">Process monitoring</span>
            <div style="width:36px;height:20px;border-radius:10px;background:{config.enable_process_list ? 'var(--accent-soft)' : 'var(--bg-2)'};border:1px solid {config.enable_process_list ? 'var(--accent-line)' : 'var(--line)'};position:relative;transition:all 0.2s">
              <div style="position:absolute;top:2px;width:14px;height:14px;border-radius:50%;background:{config.enable_process_list ? 'var(--accent)' : 'var(--ink-4)'};left:{config.enable_process_list ? '18px' : '2px'};transition:left 0.2s,background 0.2s"></div>
            </div>
          </div>
        </div>
      </section>

      <!-- Alerts -->
      <section>
        <div class="eyebrow" style="margin-bottom:14px">Alerts</div>
        <div style="display:flex;flex-direction:column;gap:12px">
          {#each [['CPU threshold', 'alert_cpu', config.alert_cpu], ['Memory threshold', 'alert_ram', config.alert_ram]] as [label, field, val]}
            <div>
              <div style="display:flex;justify-content:space-between;margin-bottom:6px">
                <span style="font-size:12px;color:var(--ink-2)">{label}</span>
                <span class="tnum font-mono" style="font-size:12px;color:var(--accent)">{(val as number).toFixed(0)}%</span>
              </div>
              <input type="range" min="50" max="100" step="1"
                     value={val}
                     oninput={e => { const v = parseFloat((e.target as HTMLInputElement).value); if (field === 'alert_cpu') config.alert_cpu = v; else config.alert_ram = v; }}
                     onchange={e => { const v = parseFloat((e.target as HTMLInputElement).value); save({ [field]: v }); }}
                     style="width:100%;accent-color:var(--accent)" />
            </div>
          {/each}
          <div>
            <div style="font-size:12px;color:var(--ink-2);margin-bottom:6px">Slack webhook URL</div>
            <div style="display:flex;gap:8px">
              <input type="url" bind:value={webhookInput} placeholder="https://hooks.slack.com/…"
                     class="font-mono focus-ring"
                     style="flex:1;background:var(--bg-2);border:1px solid var(--line);border-radius:8px;padding:8px 10px;font-size:11px;color:var(--ink);outline:none" />
              <button onclick={() => save({ slack_webhook: webhookInput })} disabled={saving}
                      class="btn" style="flex-shrink:0">Save</button>
            </div>
          </div>
        </div>
      </section>

      <!-- Data -->
      <section>
        <div class="eyebrow" style="margin-bottom:14px">Data</div>
        <div>
          <div style="font-size:12px;color:var(--ink-2);margin-bottom:8px">Retention period</div>
          <div class="seg-control">
            {#each RETAIN as d}
              <button class="seg-btn {config.retention_days === d ? 'active' : ''}"
                      onclick={() => save({ retention_days: d })}>{d}d</button>
            {/each}
          </div>
        </div>
        <a href="/api/history/export?window=all"
           download="astral-history.csv"
           style="display:inline-block;margin-top:12px;font-size:12px;color:var(--accent);text-decoration:none">
          Export CSV ↓
        </a>
      </section>

      <!-- Account -->
      <section>
        <div class="eyebrow" style="margin-bottom:14px">Account</div>
        <div style="display:flex;flex-direction:column;gap:8px">
          <button onclick={revokeAll} disabled={revoking}
                  style="padding:9px 14px;border-radius:8px;border:1px solid rgba(244,63,94,0.3);background:var(--crit-soft);color:var(--crit);font-size:12px;font-weight:500;cursor:pointer;text-align:left">
            {revoking ? 'Revoking…' : 'Revoke all sessions'}
          </button>
          <button onclick={() => { onClose(); onLogout(); }}
                  style="padding:9px 14px;border-radius:8px;border:1px solid rgba(244,63,94,0.2);background:transparent;color:var(--crit);font-size:12px;font-weight:500;cursor:pointer;text-align:left;opacity:0.8">
            Sign out
          </button>
        </div>
      </section>
    </div>

    <!-- Footer -->
    <div style="padding:16px 24px;border-top:1px solid var(--line);display:flex;align-items:center;justify-content:space-between;flex-shrink:0">
      <span class="font-mono" style="font-size:11px;color:var(--ink-4)">Astral v1.1.0</span>
      <button onclick={onClose} class="btn btn-primary">Done</button>
    </div>
  </div>
{/if}
```

- [ ] **Step 2: Build + commit**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
git add web/src/lib/components/Settings.svelte && git commit -m "feat: rewrite Settings as standalone drawer"
```

---

### Task 14: AlertHistoryDrawer.svelte restyle

**Files:**
- Modify: `web/src/lib/components/AlertHistoryDrawer.svelte`

- [ ] **Step 1: Replace glassmorphism classes with token system (logic unchanged)**

Replace all class strings in the drawer. Key replacements:
- `bg-slate-950` → `background:var(--bg-1)`
- `border-white/[0.08]` → `border:1px solid var(--line)`
- `text-white` → `color:var(--ink)`
- `text-slate-500` → `color:var(--ink-3)`
- `bg-white/[0.05]` → `background:var(--bg-2)`
- Add `anim-backdrop` and `anim-drawer` CSS animation classes to backdrop/panel
- Change `animate-fade-in` to `anim-fade-up`
- Alert item border: `border:1px solid var(--warm-line)` (CPU) or `border:1px solid rgba(244,63,94,0.25)` (RAM)

Full rewrite of the template:

```svelte
{#if open}
  <div class="anim-backdrop" onclick={onClose}
       style="position:fixed;inset:0;z-index:40;background:rgba(0,0,0,0.5);cursor:default"></div>

  <div class="anim-drawer nice-scroll"
       style="position:fixed;right:0;top:0;bottom:0;z-index:50;width:380px;max-width:100vw;
              background:var(--bg-1);border-left:1px solid var(--line);
              display:flex;flex-direction:column;overflow-y:auto">
    <!-- Header -->
    <div style="display:flex;justify-content:space-between;align-items:center;padding:20px 24px;border-bottom:1px solid var(--line);flex-shrink:0">
      <div>
        <div style="font-size:14px;font-weight:600;color:var(--ink)">Alert History</div>
        <div style="font-size:11px;color:var(--ink-4);margin-top:2px">Last 50 alerts · newest first</div>
      </div>
      <button onclick={onClose} class="btn btn-ghost" style="padding:6px;border:1px solid var(--line);border-radius:6px">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <!-- List -->
    <div style="flex:1;padding:16px;display:flex;flex-direction:column;gap:8px">
      {#if loading}
        <div style="text-align:center;padding:40px 0;color:var(--ink-4);font-size:12px">Loading…</div>
      {:else if history.length === 0}
        <div style="text-align:center;padding:60px 0;color:var(--ink-4);font-size:12px">No alerts recorded yet</div>
      {:else}
        {#each history as alert}
          <div style="border-radius:8px;padding:12px;border:1px solid {alert.kind === 'cpu' ? 'var(--warm-line)' : 'rgba(244,63,94,0.25)'};background:{alert.kind === 'cpu' ? 'var(--warm-soft)' : 'var(--crit-soft)'}">
            <div style="display:flex;justify-content:space-between;align-items:flex-start;gap:8px;margin-bottom:4px">
              <span style="font-size:10px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:{alert.kind === 'cpu' ? 'var(--warm)' : 'var(--crit)'}">
                {alert.kind === 'cpu' ? 'CPU' : 'Memory'}
              </span>
              <span class="tnum font-mono" style="font-size:10px;color:var(--ink-4)">{formatTime(alert.timestamp)}</span>
            </div>
            <div style="font-size:12px;color:var(--ink-2);margin-bottom:6px">{alert.message}</div>
            <div style="display:flex;gap:12px">
              <span style="font-size:10px;color:var(--ink-4)">Value: <span style="color:var(--ink-3)">{alert.value.toFixed(1)}%</span></span>
              <span style="font-size:10px;color:var(--ink-4)">Threshold: <span style="color:var(--ink-3)">{alert.threshold}%</span></span>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
{/if}
```

Keep the existing `<script>` block completely unchanged.

- [ ] **Step 2: Build + commit**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
git add web/src/lib/components/AlertHistoryDrawer.svelte && git commit -m "style: restyle AlertHistoryDrawer with token system"
```

---

### Task 15: Toast.svelte restyle

**Files:**
- Modify: `web/src/lib/components/Toast.svelte`

- [ ] **Step 1: Restyle (logic unchanged)**

```svelte
<script lang="ts">
  import type { AlertEvent } from '../types';
  let { alerts, onDismiss }: { alerts: AlertEvent[], onDismiss: (i: number) => void } = $props();
</script>

{#if alerts.length > 0}
  <div style="position:fixed;top:16px;right:16px;z-index:60;display:flex;flex-direction:column;gap:10px;max-width:340px">
    {#each alerts as alert, i}
      <div class="surface anim-toast"
           style="padding:14px 16px;border-color:{alert.kind === 'cpu' ? 'var(--warm-line)' : 'rgba(244,63,94,0.3)'}">
        <div style="display:flex;align-items:flex-start;gap:12px">
          <div style="width:28px;height:28px;border-radius:6px;flex-shrink:0;display:flex;align-items:center;justify-content:center;background:{alert.kind === 'cpu' ? 'var(--warm-soft)' : 'var(--crit-soft)'}">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
                 stroke="{alert.kind === 'cpu' ? 'var(--warm)' : 'var(--crit)'}" stroke-width="2">
              <path d="M12 9v4m0 4h.01M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
            </svg>
          </div>
          <div style="flex:1;min-width:0">
            <div style="font-size:11px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:{alert.kind === 'cpu' ? 'var(--warm)' : 'var(--crit)'};margin-bottom:2px">
              {alert.kind === 'cpu' ? 'CPU Alert' : 'Memory Alert'}
            </div>
            <div style="font-size:12px;color:var(--ink-2)">{alert.message}</div>
          </div>
          <button onclick={() => onDismiss(i)} style="color:var(--ink-4);background:none;border:none;cursor:pointer;padding:0;flex-shrink:0">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
      </div>
    {/each}
  </div>
{/if}
```

- [ ] **Step 2: Build + commit**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
git add web/src/lib/components/Toast.svelte && git commit -m "style: restyle Toast with token system"
```

---

### Task 16: App.svelte rewrite

**Files:**
- Modify: `web/src/App.svelte`

- [ ] **Step 1: Full rewrite — new layout, showSettings state, Settings import, footer**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import type { SystemMetrics, AlertEvent } from './lib/types';
  import TopBar    from './lib/components/TopBar.svelte';
  import CpuCard   from './lib/components/CpuCard.svelte';
  import MemoryCard  from './lib/components/MemoryCard.svelte';
  import NetworkCard from './lib/components/NetworkCard.svelte';
  import DiskCard    from './lib/components/DiskCard.svelte';
  import HistoryChart from './lib/components/HistoryChart.svelte';
  import ProcessList  from './lib/components/ProcessList.svelte';
  import Toast        from './lib/components/Toast.svelte';
  import Login        from './lib/components/Login.svelte';
  import AlertHistoryDrawer from './lib/components/AlertHistoryDrawer.svelte';
  import Settings    from './lib/components/Settings.svelte';

  let metrics: SystemMetrics | null = $state(null);
  let cpuHistory: number[] = $state([]);
  let txHistory:  number[] = $state([]);
  let rxHistory:  number[] = $state([]);
  let alerts: AlertEvent[] = $state([]);
  let authenticated = $state(false);
  let refreshRate   = $state(1);
  let lastUpdate    = $state(0);
  let showAlertHistory = $state(false);
  let showSettings     = $state(false);
  let lastRefreshTs    = $state(0);

  let eventSource: { close: () => void } | null = null;
  let alertSource: { close: () => void } | null = null;

  function handleLogin() { authenticated = true; connectStreams(); }

  async function handleLogout() {
    await fetch('/api/logout', { method: 'POST' }).catch(() => {});
    authenticated = false; metrics = null;
    cpuHistory = []; txHistory = []; rxHistory = []; alerts = [];
    showSettings = false; showAlertHistory = false;
    if (eventSource) eventSource.close();
    if (alertSource) alertSource.close();
  }

  function handleRefreshRateChange(rate: number) { refreshRate = rate; }

  function dismissAlert(i: number) { alerts = alerts.filter((_, idx) => idx !== i); }

  function connectStreams() { startMetricsStream(); startAlertStream(); }

  function startMetricsStream() {
    const ac = new AbortController();
    fetch('/api/stream', { signal: ac.signal })
      .then(res => {
        if (!res.ok) { if (res.status === 401) handleLogout(); return; }
        const reader = res.body!.getReader();
        const dec = new TextDecoder();
        let buf = '';
        function pump(): Promise<void> {
          return reader.read().then(({ done, value }) => {
            if (done) return;
            buf += dec.decode(value, { stream: true });
            const lines = buf.split('\n');
            buf = lines.pop() ?? '';
            for (const line of lines) {
              if (!line.startsWith('data: ')) continue;
              const now = Date.now();
              if (now - lastUpdate < refreshRate * 1000 - 100) continue;
              lastUpdate = now; lastRefreshTs = Math.floor(now / 1000);
              try {
                const d: SystemMetrics = JSON.parse(line.slice(6));
                metrics = d;
                cpuHistory = [...cpuHistory, d.cpu_usage].slice(-60);
                txHistory  = [...txHistory,  d.network_tx].slice(-30);
                rxHistory  = [...rxHistory,  d.network_rx].slice(-30);
              } catch { /**/ }
            }
            return pump();
          });
        }
        pump();
      })
      .catch(e => { if (e.name !== 'AbortError' && authenticated) setTimeout(startMetricsStream, 3000); });
    eventSource = { close: () => ac.abort() };
  }

  function startAlertStream() {
    const ac = new AbortController();
    fetch('/api/alerts', { signal: ac.signal })
      .then(res => {
        if (!res.ok) return;
        const reader = res.body!.getReader();
        const dec = new TextDecoder();
        let buf = '', nextIsData = false;
        function pump(): Promise<void> {
          return reader.read().then(({ done, value }) => {
            if (done) return;
            buf += dec.decode(value, { stream: true });
            const lines = buf.split('\n');
            buf = lines.pop() ?? '';
            for (const line of lines) {
              if (line === 'event: alert') { nextIsData = true; continue; }
              if (nextIsData && line.startsWith('data: ')) {
                try {
                  const a: AlertEvent = JSON.parse(line.slice(6));
                  alerts = [...alerts, a].slice(-5);
                  setTimeout(() => { if (alerts.length) alerts = alerts.slice(1); }, 10000);
                } catch { /**/ }
                nextIsData = false;
              }
            }
            return pump();
          });
        }
        pump();
      })
      .catch(e => { if (e.name !== 'AbortError' && authenticated) setTimeout(startAlertStream, 5000); });
    alertSource = { close: () => ac.abort() };
  }

  function fmtTs(ts: number): string {
    if (!ts) return '—';
    return new Date(ts * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }

  onMount(() => {
    fetch('/api/auth/check').then(r => { if (r.ok) { authenticated = true; connectStreams(); } }).catch(() => {});
    return () => { eventSource?.close(); alertSource?.close(); };
  });
</script>

{#if !authenticated}
  <Login onLogin={handleLogin} />
{:else}
  <Toast {alerts} onDismiss={dismissAlert} />
  <AlertHistoryDrawer open={showAlertHistory} onClose={() => showAlertHistory = false} />
  <Settings
    open={showSettings}
    onClose={() => showSettings = false}
    {refreshRate}
    onRefreshRateChange={handleRefreshRateChange}
    onLogout={handleLogout}
  />

  <div style="max-width:1440px;margin:0 auto;padding:26px 28px">
    <TopBar
      {metrics}
      {refreshRate}
      {alerts}
      onShowSettings={() => showSettings = true}
      onLogout={handleLogout}
      onShowAlertHistory={() => showAlertHistory = true}
    />

    {#if metrics}
      <!-- Metric cards -->
      <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:20px" class="anim-fade-up stagger-1">
        <div class="anim-fade-up stagger-1"><CpuCard usage={metrics.cpu_usage} cores={metrics.cpu_cores} history={cpuHistory} load={metrics.cpu_load} /></div>
        <div class="anim-fade-up stagger-2"><MemoryCard used={metrics.used_memory} total={metrics.total_memory} swap_used={metrics.used_swap} swap_total={metrics.total_swap} /></div>
        <div class="anim-fade-up stagger-3"><NetworkCard tx={metrics.network_tx} rx={metrics.network_rx} {txHistory} {rxHistory} /></div>
        <div class="anim-fade-up stagger-4"><DiskCard disks={metrics.disks} /></div>
      </div>

      <!-- Bottom row -->
      <div style="display:grid;grid-template-columns:2fr 1fr;gap:20px;margin-top:20px;height:400px" class="anim-fade-up stagger-5">
        <HistoryChart totalMemory={metrics.total_memory} />
        <ProcessList processes={metrics.processes} totalMemory={metrics.total_memory} />
      </div>

      <!-- Footer -->
      <div style="margin-top:16px;padding:10px 0;border-top:1px solid var(--line);display:flex;gap:16px;align-items:center">
        <span class="font-mono" style="font-size:10px;color:var(--ink-4)">{metrics.hostname}</span>
        <span style="color:var(--line-2)">·</span>
        <span style="font-size:10px;color:var(--ink-4)">Updated {fmtTs(lastRefreshTs)}</span>
        <span style="color:var(--line-2)">·</span>
        <span style="font-size:10px;color:var(--ink-4)">Rate {refreshRate}s</span>
        <span style="color:var(--line-2)">·</span>
        <span style="font-size:10px;color:var(--ink-4)">Retention {metrics ? '—' : '—'}</span>
      </div>

    {:else}
      <!-- Skeleton -->
      <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:20px">
        {#each [0,1,2,3] as i}
          <div class="surface skeleton" style="height:140px;animation-delay:{i*60}ms"></div>
        {/each}
      </div>
      <div style="display:grid;grid-template-columns:2fr 1fr;gap:20px;margin-top:20px">
        <div class="surface skeleton" style="height:400px"></div>
        <div class="surface skeleton" style="height:400px"></div>
      </div>
    {/if}
  </div>
{/if}
```

- [ ] **Step 2: Build + commit**

```bash
cd web && npm run build 2>&1 | grep -E "error|Error" | head -10
git add web/src/App.svelte && git commit -m "feat: rewrite App.svelte with new layout, settings drawer, footer"
```

---

### Task 17: Cleanup — remove uPlot, final build

**Files:**
- Modify: `web/package.json` — remove `uplot`

- [ ] **Step 1: Remove uPlot dependency**

```bash
cd web && npm uninstall uplot
```

- [ ] **Step 2: Verify no remaining uplot imports**

```bash
grep -r "uplot" web/src/ 2>/dev/null
```

Expected: No matches.

- [ ] **Step 3: Full frontend build**

```bash
cd web && npm run build
```

Expected: Build succeeds with no errors.

- [ ] **Step 4: Full backend build**

```bash
cargo build --release
```

Expected: Compiles cleanly.

- [ ] **Step 5: Smoke test** — start the binary and verify all pages load:

```bash
cargo run -- --auth admin:test
# In another terminal:
curl -s -c jar.txt -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" -d '{"username":"admin","password":"test"}' | head -5
curl -s -b jar.txt http://localhost:8080/api/settings | head -5
curl -s -b jar.txt -X POST http://localhost:8080/api/sessions/revoke -v 2>&1 | grep "< HTTP"
```

Expected: login returns `{"username":"admin"}`, settings returns JSON, revoke returns `204`.

- [ ] **Step 6: Final commit**

```bash
git add web/package.json web/package-lock.json
git commit -m "chore: remove uplot dependency, complete redesign"
```
