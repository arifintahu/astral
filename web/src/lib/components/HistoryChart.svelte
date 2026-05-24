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
  let containerH = $state(220);
  let fetchInterval: ReturnType<typeof setInterval>;

  const METRICS: Record<MetricKey, {
    label: string; color: string; unit: string;
    fixedMax: number | null; fmt: (v: number) => string;
  }> = {
    cpu:  { label: 'CPU',      color: 'var(--accent)', unit: '%',    fixedMax: 100,  fmt: v => v.toFixed(1) + '%' },
    mem:  { label: 'Memory',   color: 'var(--warm)',   unit: '%',    fixedMax: 100,  fmt: v => v.toFixed(1) + '%' },
    net:  { label: 'Network',  color: 'var(--accent)', unit: 'MB/s', fixedMax: null, fmt: v => v.toFixed(2) + ' MB/s' },
    disk: { label: 'Disk I/O', color: 'var(--warm)',   unit: 'MB/s', fixedMax: null, fmt: v => v.toFixed(2) + ' MB/s' },
  };

  const WINDOWS: { id: WindowKey; label: string }[] = [
    { id: '6h',  label: '6H'  },
    { id: '24h', label: '24H' },
    { id: '7d',  label: '7D'  },
    { id: 'all', label: 'All' },
  ];

  function extractValue(p: MetricPoint, key: MetricKey): number {
    switch (key) {
      case 'cpu':  return p.cpu_usage;
      case 'mem':  return totalMemory > 0 ? (p.used_memory / totalMemory) * 100 : 0;
      case 'net':  return (p.network_tx + p.network_rx) / 1e6;
      case 'disk': return (p.disk_read_rate + p.disk_write_rate) / 1e6;
    }
  }

  let values   = $derived(data.map(p => extractValue(p, metricKey)));
  let nowVal   = $derived(values.length ? values[values.length - 1] : 0);
  let avgVal   = $derived(values.length ? values.reduce((a, b) => a + b, 0) / values.length : 0);
  let peakVal  = $derived(values.length ? Math.max(...values) : 0);

  const PAD_L = 54, PAD_R = 12, PAD_T = 14, PAD_B = 24;

  let plotW = $derived(Math.max(containerW - PAD_L - PAD_R, 1));
  let plotH = $derived(Math.max(containerH - PAD_T - PAD_B, 1));

  let maxVal = $derived.by(() => {
    const cfg = METRICS[metricKey];
    if (cfg.fixedMax !== null) return cfg.fixedMax;
    const m = Math.max(...values, 0.001);
    const mag = Math.pow(10, Math.floor(Math.log10(m)));
    return Math.ceil(m / mag) * mag;
  });

  let gridLines = $derived(
    Array.from({ length: 5 }, (_, i) => ({
      y: PAD_T + (i / 4) * plotH,
      val: maxVal * (1 - i / 4),
    }))
  );

  let svgPath = $derived.by(() => {
    if (values.length < 2) return { line: '', area: '', dotX: 0, dotY: PAD_T };
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
    if (data.length < 2) return [] as { x: number; label: string }[];
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

  $effect(() => {
    // Re-fetch when window changes; metric change only affects derived values
    const wk = windowKey;
    void wk;
    fetchData();
  });

  onMount(() => {
    const ro = new ResizeObserver(entries => {
      for (const e of entries) {
        containerW = e.contentRect.width;
        containerH = e.contentRect.height;
      }
    });
    if (containerEl) ro.observe(containerEl);
    fetchInterval = setInterval(fetchData, 60000);
    return () => { clearInterval(fetchInterval); ro.disconnect(); };
  });

  function fmtStat(v: number): string {
    return METRICS[metricKey].fmt(v);
  }
</script>

<div class="surface h-full flex flex-col" style="padding:18px 20px">
  <!-- Header row -->
  <div class="flex flex-wrap items-center gap-3 mb-4 flex-shrink-0">
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
      <div class="flex items-center gap-5 ml-1">
        {#each [['now', nowVal], ['avg', avgVal], ['peak', peakVal]] as [lbl, val]}
          <div class="flex flex-col">
            <span class="eyebrow" style="font-size:9px;margin-bottom:1px">{lbl}</span>
            <span class="tnum font-mono" style="font-size:12px;color:var(--ink)">{fmtStat(val as number)}</span>
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

  <!-- Chart -->
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
          <linearGradient id="cgrad-{metricKey}" x1="0" x2="0" y1="0" y2="1">
            <stop offset="0%"   stop-color={METRICS[metricKey].color} stop-opacity="0.22"/>
            <stop offset="100%" stop-color={METRICS[metricKey].color} stop-opacity="0.02"/>
          </linearGradient>
          <pattern id="chart-dots" x={PAD_L} y={PAD_T} width="16" height="16" patternUnits="userSpaceOnUse">
            <circle cx="1" cy="1" r="0.8" fill="var(--line)"/>
          </pattern>
        </defs>

        <!-- Dot grid -->
        <rect x={PAD_L} y={PAD_T} width={plotW} height={plotH} fill="url(#chart-dots)" opacity="0.7"/>

        <!-- Gridlines + Y labels -->
        {#each gridLines as gl}
          <line x1={PAD_L} y1={gl.y} x2={PAD_L + plotW} y2={gl.y}
                stroke="var(--line)" stroke-width="1"/>
          <text x={PAD_L - 5} y={gl.y + 3.5} text-anchor="end"
                font-family="'Geist Mono', monospace" font-size="9"
                fill="var(--ink-4)">{METRICS[metricKey].fmt(gl.val)}</text>
        {/each}

        <!-- Area fill -->
        <path d={svgPath.area} fill="url(#cgrad-{metricKey})"/>

        <!-- Line -->
        <path d={svgPath.line} fill="none"
              stroke={METRICS[metricKey].color} stroke-width="1.5"
              stroke-linecap="round" stroke-linejoin="round"/>

        <!-- Animated leading dot -->
        <circle cx={svgPath.dotX} cy={svgPath.dotY} r="3"
                fill={METRICS[metricKey].color}
                style="animation:pulseDot 2s ease-in-out infinite"/>

        <!-- X-axis labels -->
        {#each xLabels as xl}
          <text x={xl.x} y={PAD_T + plotH + 17} text-anchor="middle"
                font-family="'Geist Mono', monospace" font-size="9"
                fill="var(--ink-4)">{xl.label}</text>
        {/each}
      </svg>
    {/if}
  </div>
</div>
