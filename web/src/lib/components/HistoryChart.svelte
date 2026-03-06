<script lang="ts">
  import { onMount } from 'svelte';
  import uPlot from 'uplot';
  import 'uplot/dist/uPlot.min.css';

  let { window = '6h' } = $props();
  let chartContainer: HTMLDivElement;
  let chart: uPlot;

  async function fetchData() {
    try {
        const res = await fetch(`/api/history?window=${window}`);
        if (!res.ok) return;
        const data = await res.json();
        
        if (!data || data.length === 0) return;

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
  
  function initChart(data: any) {
      if (!chartContainer) return;

      const opts: uPlot.Options = {
          title: "System Metrics History",
          width: chartContainer.clientWidth,
          height: 300,
          series: [
              {},
              {
                  label: "CPU",
                  stroke: "#10b981", // emerald-500
                  width: 2,
                  scale: "%",
                  value: (u, v) => v == null ? "-" : v.toFixed(1) + "%",
              },
              {
                  label: "Memory",
                  stroke: "#3b82f6", // blue-500
                  width: 2,
                  scale: "bytes",
                  value: (u, v) => v == null ? "-" : (v / 1024 / 1024 / 1024).toFixed(2) + " GB",
              }
          ],
          axes: [
              {
                  stroke: "#71717a", // zinc-500
                  grid: { stroke: "#3f3f46", width: 1 }, // zinc-700
              },
              {
                  scale: "%",
                  stroke: "#71717a",
                  values: (u, vals, space) => vals.map(v => +v.toFixed(0) + "%"),
                  grid: { stroke: "#3f3f46", width: 1 },
              },
              {
                  scale: "bytes",
                  stroke: "#71717a",
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
      const interval = setInterval(fetchData, 60000);
      
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

<div class="bg-zinc-800 rounded-lg p-4 shadow-md border border-zinc-700 mt-6">
    <div class="mb-4 flex justify-between items-center">
        <h3 class="text-zinc-400 text-sm font-semibold uppercase tracking-wider">Historical Data ({window})</h3>
        <!-- Window selector could go here -->
    </div>
    <div bind:this={chartContainer} class="w-full text-zinc-300"></div>
</div>
