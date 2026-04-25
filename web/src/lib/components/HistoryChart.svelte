<script lang="ts">
  import { onMount } from 'svelte';
  import uPlot from 'uplot';
  import 'uplot/dist/uPlot.min.css';
  import type { MetricPoint } from '../types';

  let window = $state('6h');
  let metricView = $state<'perf' | 'network' | 'disk'>('perf');
  let chartContainer: HTMLDivElement;
  let chart: uPlot | null = $state(null);
  let interval: ReturnType<typeof setInterval>;
  let hasData = $state(false);
  let isLoading = $state(true);

  const windows = [
    { id: '6h', label: '6H' },
    { id: '24h', label: '24H' },
    { id: '7d', label: '7D' },
    { id: 'all', label: 'All' },
  ];

  const views = [
    { id: 'perf', label: 'CPU & Mem' },
    { id: 'network', label: 'Network' },
    { id: 'disk', label: 'Disk I/O' },
  ];

  // Legend config per view — T-06: always-visible static legend.
  const legendConfig = {
    perf: [
      { color: '#a855f7', label: 'CPU %' },
      { color: '#06b6d4', label: 'Memory' },
    ],
    network: [
      { color: '#a855f7', label: 'TX' },
      { color: '#06b6d4', label: 'RX' },
    ],
    disk: [
      { color: '#06b6d4', label: 'Read' },
      { color: '#f59e0b', label: 'Write' },
    ],
  };

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function buildChartOptions(view: string): uPlot.Options {
    const baseAxes: uPlot.Axis[] = [
      {
        stroke: 'rgba(148,163,184,0.4)',
        grid: { stroke: 'rgba(255,255,255,0.03)', width: 1 },
        ticks: { stroke: 'rgba(255,255,255,0.05)', width: 1 },
        font: "11px 'Inter', sans-serif",
        space: 80,
        values: (u, vals) =>
          vals.map(v => new Date(v * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })),
      },
    ];

    if (view === 'perf') {
      return {
        width: chartContainer?.clientWidth ?? 600,
        height: chartContainer?.clientHeight ?? 300,
        padding: [16, 8, 0, 0],
        cursor: { show: true, x: true, y: false },
        legend: { show: false },
        series: [
          { value: (u, v) => (v == null ? '-' : new Date(v * 1000).toLocaleTimeString()) },
          {
            label: 'CPU',
            stroke: '#a855f7',
            width: 1.5,
            scale: '%',
            fill: 'rgba(168,85,247,0.06)',
            value: (u, v) => (v == null ? '-' : v.toFixed(1) + '%'),
          },
          {
            label: 'Memory',
            stroke: '#06b6d4',
            width: 1.5,
            scale: 'bytes',
            fill: 'rgba(6,182,212,0.06)',
            value: (u, v) => (v == null ? '-' : (v / 1073741824).toFixed(2) + ' GB'),
          },
        ],
        axes: [
          ...baseAxes,
          {
            scale: '%',
            stroke: 'rgba(168,85,247,0.5)',
            values: (u, vals) => vals.map(v => +v.toFixed(0) + '%'),
            grid: { stroke: 'rgba(255,255,255,0.03)', width: 1 },
            ticks: { stroke: 'rgba(255,255,255,0.05)', width: 1 },
            font: "11px 'JetBrains Mono', monospace",
            size: 48,
          },
          {
            scale: 'bytes',
            stroke: 'rgba(6,182,212,0.5)',
            values: (u, vals) => vals.map(v => (v / 1073741824).toFixed(1) + 'G'),
            side: 1,
            grid: { show: false },
            ticks: { stroke: 'rgba(255,255,255,0.05)', width: 1 },
            font: "11px 'JetBrains Mono', monospace",
            size: 48,
          },
        ],
        scales: { '%': { auto: true, range: [0, 100] }, bytes: { auto: true } },
      };
    }

    // network and disk share a single bytes/s scale
    const [s1Color, s2Color] = view === 'network' ? ['#a855f7', '#06b6d4'] : ['#06b6d4', '#f59e0b'];
    const [s1Label, s2Label] = view === 'network' ? ['TX', 'RX'] : ['Read', 'Write'];

    return {
      width: chartContainer?.clientWidth ?? 600,
      height: chartContainer?.clientHeight ?? 300,
      padding: [16, 8, 0, 0],
      cursor: { show: true, x: true, y: false },
      legend: { show: false },
      series: [
        { value: (u, v) => (v == null ? '-' : new Date(v * 1000).toLocaleTimeString()) },
        {
          label: s1Label,
          stroke: s1Color,
          width: 1.5,
          scale: 'bps',
          fill: s1Color + '10',
          value: (u, v) => (v == null ? '-' : formatBytes(v) + '/s'),
        },
        {
          label: s2Label,
          stroke: s2Color,
          width: 1.5,
          scale: 'bps',
          fill: s2Color + '10',
          value: (u, v) => (v == null ? '-' : formatBytes(v) + '/s'),
        },
      ],
      axes: [
        ...baseAxes,
        {
          scale: 'bps',
          stroke: 'rgba(148,163,184,0.4)',
          values: (u, vals) => vals.map(v => formatBytes(v) + '/s'),
          grid: { stroke: 'rgba(255,255,255,0.03)', width: 1 },
          ticks: { stroke: 'rgba(255,255,255,0.05)', width: 1 },
          font: "11px 'JetBrains Mono', monospace",
          size: 64,
        },
      ],
      scales: { bps: { auto: true } },
    };
  }

  function buildChartData(data: MetricPoint[], view: string): uPlot.AlignedData {
    const ts = data.map(d => d.timestamp);
    if (view === 'perf') {
      return [ts, data.map(d => d.cpu_usage), data.map(d => d.used_memory)];
    }
    if (view === 'network') {
      return [ts, data.map(d => d.network_tx), data.map(d => d.network_rx)];
    }
    // disk
    return [ts, data.map(d => d.disk_read_rate), data.map(d => d.disk_write_rate)];
  }

  function destroyChart() {
    if (chart) {
      chart.destroy();
      chart = null;
    }
  }

  async function fetchData() {
    try {
      isLoading = true;
      const res = await fetch(`/api/history?window=${window}`);
      if (!res.ok) return;
      const raw: MetricPoint[] = await res.json();

      if (!raw || raw.length === 0) {
        hasData = false;
        destroyChart();
        return;
      }

      hasData = true;
      const chartData = buildChartData(raw, metricView);

      if (chart) {
        chart.setData(chartData);
      } else {
        destroyChart();
        const opts = buildChartOptions(metricView);
        chart = new uPlot(opts, chartData, chartContainer);
      }
    } catch (e) {
      console.error('Failed to fetch history:', e);
    } finally {
      isLoading = false;
    }
  }

  function setWindow(w: string) {
    window = w;
    fetchData();
  }

  function setView(v: typeof metricView) {
    metricView = v;
    destroyChart();
    fetchData();
  }

  // T-16: CSV export
  function exportCSV() {
    const url = `/api/history/export?window=${window}`;
    const a = document.createElement('a');
    a.href = url;
    a.download = `astral-history-${window}.csv`;
    a.click();
  }

  onMount(() => {
    fetchData();
    interval = setInterval(fetchData, 60000);

    const resizeObserver = new ResizeObserver(() => {
      if (chart && chartContainer) {
        chart.setSize({
          width: chartContainer.clientWidth,
          height: chartContainer.clientHeight,
        });
      }
    });
    if (chartContainer) resizeObserver.observe(chartContainer);

    return () => {
      clearInterval(interval);
      resizeObserver.disconnect();
      destroyChart();
    };
  });
</script>

<div class="glass-panel p-6 h-full flex flex-col">
  <!-- Header row -->
  <div class="flex flex-wrap justify-between items-center gap-3 mb-4 flex-shrink-0">
    <div class="flex items-center gap-3">
      <h3 class="text-[11px] font-bold text-slate-300 uppercase tracking-[0.15em]">History</h3>
      <!-- T-08/T-09: metric view selector -->
      <div class="flex bg-white/[0.03] rounded-xl p-1 border border-white/[0.05]">
        {#each views as v}
          <button
            class="px-2.5 py-1 text-[10px] font-semibold rounded-lg transition-all duration-200 cursor-pointer
                   {metricView === v.id ? 'bg-white/[0.08] text-white shadow-sm' : 'text-slate-500 hover:text-slate-300'}"
            onclick={() => setView(v.id as typeof metricView)}
          >{v.label}</button>
        {/each}
      </div>
    </div>

    <div class="flex items-center gap-2">
      <!-- Time window selector -->
      <div class="flex bg-white/[0.03] rounded-xl p-1 border border-white/[0.05]">
        {#each windows as w}
          <button
            class="px-3 py-1 text-[11px] font-semibold rounded-lg transition-all duration-200 cursor-pointer
                   {window === w.id ? 'bg-white/[0.08] text-white shadow-sm' : 'text-slate-500 hover:text-slate-300'}"
            onclick={() => setWindow(w.id)}
          >{w.label}</button>
        {/each}
      </div>

      <!-- T-16: export button -->
      <button
        onclick={exportCSV}
        title="Export CSV"
        class="flex items-center justify-center w-7 h-7 rounded-lg bg-white/[0.04] border border-white/[0.06]
               hover:bg-white/[0.08] hover:border-white/[0.1] transition-all cursor-pointer"
      >
        <svg class="w-3.5 h-3.5 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
        </svg>
      </button>
    </div>
  </div>

  <!-- T-06: static legend — always shows color swatches + labels, no dashes -->
  <div class="flex items-center gap-5 mb-3 flex-shrink-0">
    {#each legendConfig[metricView] as entry}
      <div class="flex items-center gap-1.5">
        <div class="w-4 h-0.5 rounded-full" style="background:{entry.color}"></div>
        <span class="text-[11px] text-slate-500">{entry.label}</span>
      </div>
    {/each}
  </div>

  <!-- Chart area -->
  <div class="flex-1 min-h-0 relative">
    <div bind:this={chartContainer} class="absolute inset-0"></div>
    {#if !hasData && !isLoading}
      <div class="absolute inset-0 flex items-center justify-center text-slate-500 text-sm pointer-events-none">
        No data available for this period
      </div>
    {/if}
    {#if isLoading && !chart}
      <div class="absolute inset-0 flex items-center justify-center text-slate-500 text-sm pointer-events-none">
        Loading…
      </div>
    {/if}
  </div>
</div>
