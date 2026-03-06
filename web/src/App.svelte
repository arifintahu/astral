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

<div class="min-h-screen p-4 md:p-6 lg:p-8 max-w-screen-2xl mx-auto">
  <TopBar metrics={metrics} />

  {#if metrics}
    <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4 md:gap-5">
      <div class="animate-fade-in-up stagger-1">
        <CpuCard usage={metrics.cpu_usage} cores={metrics.cpu_cores} history={cpuHistory} />
      </div>
      <div class="animate-fade-in-up stagger-2">
        <MemoryCard
          used={metrics.used_memory}
          total={metrics.total_memory}
          swap_used={metrics.used_swap}
          swap_total={metrics.total_swap}
        />
      </div>
      <div class="animate-fade-in-up stagger-3">
        <NetworkCard tx={metrics.network_tx} rx={metrics.network_rx} />
      </div>
      <div class="animate-fade-in-up stagger-4">
        <DiskCard disks={metrics.disks} />
      </div>
    </div>

    <div class="mt-5 animate-fade-in-up stagger-5">
        <HistoryChart />
    </div>
  {:else}
    <!-- Skeleton loading state -->
    <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4 md:gap-5">
      {#each [1, 2, 3, 4] as i}
        <div class="glass-card p-6 animate-fade-in" style="animation-delay: {i * 0.05}s">
          <div class="flex justify-between items-center mb-5">
            <div class="skeleton h-3 w-20"></div>
            <div class="skeleton h-5 w-16"></div>
          </div>
          <div class="flex items-end gap-4">
            <div class="skeleton h-12 w-24"></div>
            <div class="flex-1 skeleton h-8"></div>
          </div>
        </div>
      {/each}
    </div>
    <div class="mt-5 glass-panel p-6 animate-fade-in" style="animation-delay: 0.25s">
      <div class="flex justify-between items-center mb-5">
        <div class="skeleton h-3 w-16"></div>
        <div class="skeleton h-7 w-40"></div>
      </div>
      <div class="skeleton h-64 w-full"></div>
    </div>
  {/if}
</div>
