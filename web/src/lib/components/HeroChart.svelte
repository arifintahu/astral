<script lang="ts">
  import type { LiveSeries, MetricKey, MetricPoint, WindowKey } from '../types';
  import { fmtPct, fmtRate, fmtRateShort, niceMaxPct, niceMaxRate, smoothPath, thin } from '../format';

  let { live, totalMemory, metric }: {
    live: LiveSeries;
    totalMemory: number;
    metric: MetricKey;
  } = $props();

  interface Series { a: number[]; b: number[] | null; ts: number[] | null }

  const METRICS: Record<MetricKey, { title: string; pct: boolean; names: string[] }> = {
    cpu:  { title: 'CPU usage',       pct: true,  names: ['CPU'] },
    mem:  { title: 'Memory used',     pct: true,  names: ['Memory'] },
    net:  { title: 'Network traffic', pct: false, names: ['Down', 'Up'] },
    disk: { title: 'Disk I/O',        pct: false, names: ['Read', 'Write'] },
  };

  const WINDOWS: { id: WindowKey; label: string }[] = [
    { id: 'live', label: 'Live' },
    { id: '6h',   label: '6H'   },
    { id: '24h',  label: '24H'  },
    { id: '7d',   label: '7D'   },
    { id: 'all',  label: 'All'  },
  ];

  let win = $state<WindowKey>('live');
  let history: MetricPoint[] = $state([]);
  let loading = $state(false);
  let hover = $state<number | null>(null);
  let plotEl: HTMLDivElement | undefined = $state();

  /* ── history (everything except Live) ─────────────────── */
  async function fetchHistory(w: WindowKey) {
    loading = true;
    try {
      const res = await fetch(`/api/history?window=${w}`);
      if (res.ok) {
        const rows: MetricPoint[] = await res.json();
        if (win === w) history = rows;
      }
    } catch { /**/ } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (win === 'live') return;
    const w = win;
    history = [];
    fetchHistory(w);
    const id = setInterval(() => fetchHistory(w), 60_000);
    return () => clearInterval(id);
  });

  // Moving to another metric or window drops the hover state.
  $effect(() => { void metric; void win; hover = null; });

  /* ── series ───────────────────────────────────────────── */
  let series = $derived.by<Series>(() => {
    if (win === 'live') {
      switch (metric) {
        case 'cpu':  return { a: live.cpu, b: null, ts: null };
        case 'mem':  return { a: live.mem, b: null, ts: null };
        case 'net':  return { a: live.rx,  b: live.tx, ts: null };
        case 'disk': return { a: live.rd,  b: live.wr, ts: null };
      }
    }
    const d = thin(history);
    const ts = d.map(p => p.timestamp);
    switch (metric) {
      case 'cpu':  return { a: d.map(p => p.cpu_usage), b: null, ts };
      case 'mem':  return { a: d.map(p => (totalMemory > 0 ? (p.used_memory / totalMemory) * 100 : 0)), b: null, ts };
      case 'net':  return { a: d.map(p => p.network_rx), b: d.map(p => p.network_tx), ts };
      case 'disk': return { a: d.map(p => p.disk_read_rate), b: d.map(p => p.disk_write_rate), ts };
    }
  });

  let cfg   = $derived(METRICS[metric]);
  let n     = $derived(series.a.length);
  let ready = $derived(n >= 2);
  let peak  = $derived(Math.max(0, ...series.a, ...(series.b ?? [])));
  let max   = $derived(cfg.pct ? niceMaxPct(peak) : niceMaxRate(peak));
  let fmt   = $derived(cfg.pct ? fmtPct : fmtRate);

  function yOf(v: number): number {
    return 300 - Math.min(Math.max(v / max, 0), 1) * 300;
  }
  function toPoints(arr: number[]) {
    return arr.map((v, i) => ({ x: (i / (n - 1)) * 1000, y: yOf(v) }));
  }

  let lineA = $derived(ready ? smoothPath(toPoints(series.a)) : '');
  let lineB = $derived(ready && series.b ? smoothPath(toPoints(series.b)) : '');
  let areaA = $derived(lineA ? `${lineA} L1000,300 L0,300 Z` : '');

  let yTicks = $derived(
    Array.from({ length: 5 }, (_, i) => {
      const v = max * (1 - i / 4);
      return { top: i * 25, label: cfg.pct ? `${+v.toFixed(1)}%` : fmtRateShort(v) };
    })
  );

  /* ── hover / tooltip ──────────────────────────────────── */
  let h  = $derived(hover === null ? n - 1 : Math.min(hover, n - 1));
  let hx = $derived(ready ? (h / (n - 1)) * 100 : 100);
  let hyA = $derived(ready ? yOf(series.a[h]) / 3 : 100);
  let hyB = $derived(ready && series.b ? yOf(series.b[h]) / 3 : 0);

  function onMove(e: PointerEvent) {
    if (!plotEl || !ready) return;
    const r = plotEl.getBoundingClientRect();
    const x = (e.clientX - r.left) / r.width;
    hover = Math.min(n - 1, Math.max(0, Math.round(x * (n - 1))));
  }

  const DAY = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  const pad = (x: number) => String(x).padStart(2, '0');

  function stamp(ts: number, long: boolean): string {
    const d = new Date(ts * 1000);
    const hm = `${pad(d.getHours())}:${pad(d.getMinutes())}`;
    switch (win) {
      case '6h':  return hm;
      case '24h': return long ? `${DAY[d.getDay()]} ${hm}` : hm;
      case '7d':  return long ? `${DAY[d.getDay()]} ${hm}` : DAY[d.getDay()];
      default:    return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
    }
  }

  function labelAt(i: number, long: boolean): string {
    if (win === 'live') return i === n - 1 ? 'now' : `−${n - 1 - i}s`;
    return series.ts ? stamp(series.ts[i], long) : '';
  }

  let xLabels = $derived(
    ready ? Array.from({ length: 5 }, (_, i) => labelAt(Math.round((i * (n - 1)) / 4), false)) : []
  );

  /* ── header stats ─────────────────────────────────────── */
  let stats = $derived.by(() => {
    if (!ready) return [] as { k: string; v: string; dot?: string }[];
    const last = n - 1;
    if (series.b) {
      return [
        { k: cfg.names[0], v: fmt(series.a[last]), dot: 'var(--accent)' },
        { k: cfg.names[1], v: fmt(series.b[last]), dot: 'var(--violet)' },
        { k: 'Peak', v: fmt(peak) },
      ];
    }
    const avg = series.a.reduce((s, v) => s + v, 0) / n;
    return [
      { k: 'Now',  v: fmt(series.a[last]) },
      { k: 'Avg',  v: fmt(avg) },
      { k: 'Peak', v: fmt(peak) },
    ];
  });
</script>

<section class="panel anim-fade-up stagger-3 flex flex-col gap-[22px] p-[clamp(16px,2.4vw,28px)]">
  <!-- Header -->
  <div class="flex flex-wrap items-end justify-between gap-x-6 gap-y-4">
    <div class="flex min-w-0 flex-col gap-3">
      <div class="flex flex-wrap items-center gap-3">
        <span class="panel-title">{cfg.title}</span>
        {#if win === 'live'}
          <span class="font-mono inline-flex items-center gap-[7px] rounded-full border px-2.5 py-1 text-[11px] tracking-[0.08em]"
                style="color:var(--accent);border-color:var(--accent-line)">
            <span class="size-1.5 rounded-full" style="background:var(--accent)"></span>STREAMING
          </span>
        {/if}
      </div>
      <div class="flex flex-wrap gap-x-8 gap-y-2">
        {#each stats as s (s.k)}
          <div class="flex flex-col gap-1">
            <span class="eyebrow flex items-center gap-[7px]">
              {#if s.dot}<span class="size-2 rounded-full" style="background:{s.dot}"></span>{/if}{s.k}
            </span>
            <span class="tnum text-[22px] font-medium tracking-[-0.02em]">{s.v}</span>
          </div>
        {/each}
      </div>
    </div>

    <div class="seg2" role="group" aria-label="Time window">
      {#each WINDOWS as w (w.id)}
        <button type="button" aria-pressed={win === w.id} onclick={() => (win = w.id)}><span>{w.label}</span></button>
      {/each}
    </div>
  </div>

  <!-- Plot -->
  <div class="flex h-[clamp(220px,32vw,300px)] gap-3">
    <div class="relative w-12 flex-none" aria-hidden="true">
      {#each yTicks as y (y.top)}
        <span class="font-mono tnum absolute right-0 -translate-y-1/2 whitespace-nowrap text-[11px]"
              style="top:{y.top}%;color:var(--ink-3)">{y.label}</span>
      {/each}
    </div>

    <div class="relative min-w-0 flex-1">
      {#each yTicks as y (y.top)}
        <div class="absolute inset-x-0 h-px" style="top:{y.top}%;background:var(--line)"></div>
      {/each}

      {#if ready}
        <svg viewBox="0 0 1000 300" preserveAspectRatio="none" role="img"
             aria-label="{cfg.title}, {win === 'live' ? 'last 60 seconds' : `window ${win}`}"
             class="absolute inset-0 h-full w-full overflow-visible">
          <defs>
            <linearGradient id="hero-fill" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0" style="stop-color:var(--accent);stop-opacity:0.30" />
              <stop offset="1" style="stop-color:var(--accent);stop-opacity:0" />
            </linearGradient>
          </defs>
          <path d={areaA} fill="url(#hero-fill)" />
          {#if lineB}
            <path d={lineB} fill="none" stroke="var(--violet)" stroke-width="2" stroke-linecap="round"
                  stroke-linejoin="round" vector-effect="non-scaling-stroke" style="opacity:0.9" />
          {/if}
          <path d={lineA} fill="none" stroke="var(--accent)" stroke-width="2.5" stroke-linecap="round"
                stroke-linejoin="round" vector-effect="non-scaling-stroke"
                style="filter:drop-shadow(0 0 7px color-mix(in srgb, var(--accent) 55%, transparent))" />
        </svg>

        <!-- crosshair + points (HTML so they stay round under the stretched viewBox) -->
        <div class="pointer-events-none absolute inset-y-0 w-px" style="left:{hx}%;background:var(--line-2)"></div>
        {#if series.b}
          <span class="pointer-events-none absolute size-[9px] rounded-full"
                style="left:{hx}%;top:{hyB}%;margin:-4.5px 0 0 -4.5px;background:var(--violet)"></span>
        {/if}
        <span class="pointer-events-none absolute size-3" style="left:{hx}%;top:{hyA}%;margin:-6px 0 0 -6px">
          <span class="ping absolute inset-0 rounded-full" style="background:var(--accent)"></span>
          <span class="absolute inset-0 box-border rounded-full border-2"
                style="background:var(--accent);border-color:var(--bg-1)"></span>
        </span>

        {#if hover !== null}
          <div class="pointer-events-none absolute top-1.5 z-10 flex min-w-[120px] flex-col gap-1.5 rounded-[10px] border px-3 py-2.5"
               style="left:{hx}%;transform:translateX({hx > 60 ? 'calc(-100% - 14px)' : '14px'});background:var(--bg-2);border-color:var(--line-2)">
            <span class="font-mono text-[11px]" style="color:var(--ink-3)">{labelAt(h, true)}</span>
            <span class="flex items-center gap-2 text-[13px]">
              <span class="size-2 rounded-full" style="background:var(--accent)"></span>
              <span style="color:var(--ink-2)">{cfg.names[0]}</span>
              <span class="tnum ml-auto font-medium">{fmt(series.a[h])}</span>
            </span>
            {#if series.b}
              <span class="flex items-center gap-2 text-[13px]">
                <span class="size-2 rounded-full" style="background:var(--violet)"></span>
                <span style="color:var(--ink-2)">{cfg.names[1]}</span>
                <span class="tnum ml-auto font-medium">{fmt(series.b[h])}</span>
              </span>
            {/if}
          </div>
        {/if}

        <div bind:this={plotEl} role="presentation" class="absolute inset-0 touch-pan-y"
             onpointermove={onMove} onpointerleave={() => (hover = null)}></div>
      {:else}
        <div class="absolute inset-0 flex flex-col items-center justify-center gap-3 text-center text-[13px]" style="color:var(--ink-3)">
          {#if loading}
            <span>Loading…</span>
          {:else}
            <span>Not enough history for this window yet.</span>
            <button type="button" class="btn" onclick={() => (win = 'live')}>Switch to Live</button>
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <!-- X axis -->
  <div class="ml-[60px] flex justify-between" aria-hidden="true">
    {#each xLabels as label}
      <span class="font-mono text-[11px]" style="color:var(--ink-3)">{label}</span>
    {/each}
  </div>
</section>
