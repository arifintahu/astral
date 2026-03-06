<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import uPlot from 'uplot';
  import 'uplot/dist/uPlot.min.css';

  let window = $state('6h');
  let chartContainer: HTMLDivElement;
  let chart: uPlot;
  let interval: number;

  const windows = [
      { id: '6h', label: '6H' },
      { id: '24h', label: '24H' },
      { id: '7d', label: '7D' },
      { id: 'all', label: 'ALL' }
  ];

  async function fetchData() {
    try {
        const res = await fetch(`/api/history?window=${window}`);
        if (!res.ok) return;
        const data = await res.json();
        
        if (!data || data.length === 0) {
            if (chart) chart.setData([[], [], []]);
            return;
        }

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
          height: 300,
          series: [
              {
                value: (u, v) => v == null ? "-" : new Date(v * 1000).toLocaleTimeString(),
              },
              {
                  label: "CPU",
                  stroke: "#a855f7", // purple-500
                  width: 2,
                  scale: "%",
                  value: (u, v) => v == null ? "-" : v.toFixed(1) + "%",
              },
              {
                  label: "Memory",
                  stroke: "#06b6d4", // cyan-500
                  width: 2,
                  scale: "bytes",
                  value: (u, v) => v == null ? "-" : (v / 1024 / 1024 / 1024).toFixed(2) + " GB",
              }
          ],
          axes: [
              {
                  stroke: "#94a3b8", // slate-400
                  grid: { stroke: "rgba(255,255,255,0.05)", width: 1 },
                  space: 80,
                  values: (u, vals) => vals.map(v => new Date(v * 1000).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})),
              },
              {
                  scale: "%",
                  stroke: "#a855f7", // purple-500
                  values: (u, vals, space) => vals.map(v => +v.toFixed(0) + "%"),
                  grid: { stroke: "rgba(255,255,255,0.05)", width: 1 },
              },
              {
                  scale: "bytes",
                  stroke: "#06b6d4", // cyan-500
                  values: (u, vals, space) => vals.map(v => (v / 1024 / 1024 / 1024).toFixed(1) + "G"),
                  side: 1,
                  grid: { show: false },
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
                  height: 300
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

<div class="glass-panel p-6 mt-6">
    <div class="mb-4 flex flex-col md:flex-row justify-between items-center gap-4">
        <h3 class="text-slate-400 text-xs font-bold uppercase tracking-widest">Historical Data</h3>
        
        <div class="flex bg-slate-900/50 rounded-lg p-1 border border-white/5">
            {#each windows as w}
                <button 
                    class="px-3 py-1 text-xs font-bold rounded-md transition-all duration-200 {window === w.id ? 'bg-cyan-500/20 text-cyan-300 shadow-[0_0_10px_rgba(6,182,212,0.2)]' : 'text-slate-500 hover:text-slate-300'}"
                    onclick={() => setWindow(w.id)}
                >
                    {w.label}
                </button>
            {/each}
        </div>
    </div>
    
    <div class="relative min-h-[300px]">
        {#if !chart}
             <div class="absolute inset-0 flex items-center justify-center text-slate-500 font-mono text-sm">
                 Loading or No Data Available...
             </div>
        {/if}
        <div bind:this={chartContainer} class="w-full text-slate-300"></div>
    </div>
</div>
