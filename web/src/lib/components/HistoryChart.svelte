<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import uPlot from 'uplot';
  import 'uplot/dist/uPlot.min.css';

  let window = $state('6h');
  let chartContainer: HTMLDivElement;
  let chart: uPlot;
  let interval: number;
  let hasData = $state(false);
  let isLoading = $state(true);

  const windows = [
      { id: '6h', label: '6H' },
      { id: '24h', label: '24H' },
      { id: '7d', label: '7D' },
      { id: 'all', label: 'All' }
  ];

  async function fetchData() {
    try {
        isLoading = true;
        const res = await fetch(`/api/history?window=${window}`);
        if (!res.ok) return;
        const data = await res.json();

        if (!data || data.length === 0) {
            hasData = false;
            if (chart) chart.setData([[], [], []]);
            return;
        }

        hasData = true;
        const timestamps = data.map((d: any) => d.timestamp);
        const cpu = data.map((d: any) => d.cpu_usage);
        const mem = data.map((d: any) => d.used_memory);

        const chartData = [timestamps, cpu, mem];

        if (chart) {
            chart.setData(chartData);
        } else {
            initChart(chartData);
        }
    } catch (e) {
        console.error("Failed to fetch history:", e);
    } finally {
        isLoading = false;
    }
  }

  function setWindow(w: string) {
      window = w;
      fetchData();
  }

  function initChart(data: any) {
      if (!chartContainer) return;

      const opts: uPlot.Options = {
          width: chartContainer.clientWidth,
          height: 280,
          padding: [16, 8, 0, 0],
          cursor: {
              show: true,
              x: true,
              y: false,
              points: { show: true, size: 6, fill: '#0f172a', stroke: '#06b6d4', width: 2 },
          },
          series: [
              {
                value: (u, v) => v == null ? "-" : new Date(v * 1000).toLocaleTimeString(),
              },
              {
                  label: "CPU",
                  stroke: "#a855f7",
                  width: 1.5,
                  scale: "%",
                  fill: "rgba(168,85,247,0.06)",
                  value: (u, v) => v == null ? "-" : v.toFixed(1) + "%",
              },
              {
                  label: "Memory",
                  stroke: "#06b6d4",
                  width: 1.5,
                  scale: "bytes",
                  fill: "rgba(6,182,212,0.06)",
                  value: (u, v) => v == null ? "-" : (v / 1024 / 1024 / 1024).toFixed(2) + " GB",
              }
          ],
          axes: [
              {
                  stroke: "rgba(148,163,184,0.4)",
                  grid: { stroke: "rgba(255,255,255,0.03)", width: 1 },
                  ticks: { stroke: "rgba(255,255,255,0.05)", width: 1 },
                  font: "11px 'Inter', sans-serif",
                  space: 80,
                  values: (u, vals) => vals.map(v => new Date(v * 1000).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})),
              },
              {
                  scale: "%",
                  stroke: "rgba(168,85,247,0.5)",
                  values: (u, vals, space) => vals.map(v => +v.toFixed(0) + "%"),
                  grid: { stroke: "rgba(255,255,255,0.03)", width: 1 },
                  ticks: { stroke: "rgba(255,255,255,0.05)", width: 1 },
                  font: "11px 'JetBrains Mono', monospace",
                  size: 48,
              },
              {
                  scale: "bytes",
                  stroke: "rgba(6,182,212,0.5)",
                  values: (u, vals, space) => vals.map(v => (v / 1024 / 1024 / 1024).toFixed(1) + "G"),
                  side: 1,
                  grid: { show: false },
                  ticks: { stroke: "rgba(255,255,255,0.05)", width: 1 },
                  font: "11px 'JetBrains Mono', monospace",
                  size: 48,
              }
          ],
          scales: {
              "%": { auto: true, range: [0, 100] },
              "bytes": { auto: true }
          },
          legend: {
              show: true,
          }
      };

      chart = new uPlot(opts, data, chartContainer);
  }

  onMount(() => {
      fetchData();
      interval = setInterval(fetchData, 60000);

      const resizeObserver = new ResizeObserver(() => {
          if (chart) {
              chart.setSize({
                  width: chartContainer.clientWidth,
                  height: 280
              });
          }
      });
      resizeObserver.observe(chartContainer);

      return () => {
          clearInterval(interval);
          resizeObserver.disconnect();
          if (chart) chart.destroy();
      };
  });
</script>

<div class="glass-panel p-6">
    <div class="mb-5 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
        <h3 class="metric-label">History</h3>

        <div class="flex bg-white/[0.03] rounded-xl p-1 border border-white/[0.05]">
            {#each windows as w}
                <button
                    class="px-3.5 py-1.5 text-[11px] font-semibold rounded-lg transition-all duration-200 cursor-pointer
                           {window === w.id
                             ? 'bg-white/[0.08] text-white shadow-sm'
                             : 'text-slate-500 hover:text-slate-300'}"
                    onclick={() => setWindow(w.id)}
                >
                    {w.label}
                </button>
            {/each}
        </div>
    </div>

    <div class="relative min-h-[280px]">
        {#if isLoading && !chart}
            <div class="absolute inset-0 flex items-center justify-center">
                <div class="flex items-center gap-3">
                    <div class="w-4 h-4 border-2 border-slate-700 border-t-cyan-500 rounded-full animate-spin"></div>
                    <span class="text-sm text-slate-500">Loading history...</span>
                </div>
            </div>
        {:else if !hasData && !isLoading}
            <div class="absolute inset-0 flex flex-col items-center justify-center gap-2">
                <svg class="w-8 h-8 text-slate-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
                <span class="text-sm text-slate-600">No historical data yet</span>
                <span class="text-[11px] text-slate-700">Data will appear after a few minutes</span>
            </div>
        {/if}
        <div bind:this={chartContainer} class="w-full text-slate-300"></div>
    </div>
</div>
