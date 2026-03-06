<script lang="ts">
  import { onMount } from 'svelte';
  import type { SystemMetrics } from './lib/types';
  import TopBar from './lib/components/TopBar.svelte';
  import CpuCard from './lib/components/CpuCard.svelte';
  import MemoryCard from './lib/components/MemoryCard.svelte';
  import NetworkCard from './lib/components/NetworkCard.svelte';
  import DiskCard from './lib/components/DiskCard.svelte';
  import HistoryChart from './lib/components/HistoryChart.svelte';

  let metrics: SystemMetrics | null = $state(null);
  let cpuHistory: number[] = $state([]);

  onMount(() => {
    const eventSource = new EventSource('/api/stream');

    eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        metrics = data;
        
        // Update history buffer
        if (metrics) {
            cpuHistory = [...cpuHistory, metrics.cpu_usage];
            if (cpuHistory.length > 60) {
                cpuHistory = cpuHistory.slice(1);
            }
        }
      } catch (e) {
        console.error('Error parsing SSE data:', e);
      }
    };

    eventSource.onerror = (e) => {
        console.error('SSE Error:', e);
    };

    return () => {
      eventSource.close();
    };
  });
</script>

<div class="min-h-screen bg-zinc-900 text-zinc-100 p-4 md:p-8 font-sans">
  <TopBar metrics={metrics} />

  {#if metrics}
    <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-6">
      <CpuCard usage={metrics.cpu_usage} cores={metrics.cpu_cores} history={cpuHistory} />
      <MemoryCard 
        used={metrics.used_memory} 
        total={metrics.total_memory} 
        swap_used={metrics.used_swap} 
        swap_total={metrics.total_swap} 
      />
      <NetworkCard tx={metrics.network_tx} rx={metrics.network_rx} />
      <DiskCard disks={metrics.disks} />
    </div>
    
    <div class="mt-6">
        <HistoryChart window="6h" />
    </div>
  {:else}
    <div class="flex flex-col justify-center items-center h-96 gap-4">
      <div class="animate-spin rounded-full h-16 w-16 border-4 border-zinc-700 border-t-emerald-500"></div>
      <p class="text-zinc-500 font-mono animate-pulse">Establishing Uplink...</p>
    </div>
  {/if}
</div>
