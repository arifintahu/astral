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
        // M-1: Session cookie is sent automatically by the browser for same-origin requests.
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
          height: chartContainer.clientHeight,
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
          if (chart && chartContainer) {
              chart.setSize({
                  width: chartContainer.clientWidth,
                  height: chartContainer.clientHeight
              });
          }
      });
      if (chartContainer) {
          resizeObserver.observe(chartContainer);
      }

      return () => {
          clearInterval(interval);
          resizeObserver.disconnect();
          if (chart) chart.destroy();
      };
  });
</script>

<div class="glass-panel p-6 h-full flex flex-col">
  <div class="flex justify-between items-center mb-6 flex-shrink-0">
    <h3 class="metric-label">History</h3>
    <div class="flex bg-white/[0.03] rounded-xl p-1 border border-white/[0.05]">
      {#each windows as w}
        <button
          class="px-3 py-1 text-[11px] font-semibold rounded-lg transition-all duration-200 cursor-pointer
                 {window === w.id ? 'bg-white/[0.08] text-white shadow-sm' : 'text-slate-500 hover:text-slate-300'}"
          onclick={() => setWindow(w.id)}
        >{w.label}</button>
      {/each}
    </div>
  </div>

  <div class="flex-1 min-h-0 relative">
    <div bind:this={chartContainer} class="absolute inset-0"></div>
    {#if !hasData && !isLoading}
      <div class="absolute inset-0 flex items-center justify-center text-slate-500 text-sm pointer-events-none">
        No data available for this period
      </div>
    {/if}
    {#if isLoading && !chart}
      <div class="absolute inset-0 flex items-center justify-center text-slate-500 text-sm pointer-events-none">
        Loading...
      </div>
    {/if}
  </div>
</div>
