<script lang="ts">
  import type { SystemMetrics } from '../types';

  let { metrics }: { metrics: SystemMetrics | null } = $props();

  function formatUptime(uptime: number): string {
    const days = Math.floor(uptime / 86400);
    const hours = Math.floor((uptime % 86400) / 3600);
    const minutes = Math.floor((uptime % 3600) / 60);
    return `${days}d ${hours}h ${minutes}m`;
  }
</script>

<div class="flex justify-between items-center p-4 bg-zinc-800 rounded-lg shadow-md mb-4 border border-zinc-700">
  <div>
    <h1 class="text-2xl font-bold text-white tracking-tight">Astral</h1>
    {#if metrics}
      <div class="text-sm text-zinc-400 flex items-center gap-2">
        <span class="font-semibold text-zinc-300">{metrics.hostname}</span>
        <span class="w-1 h-1 bg-zinc-600 rounded-full"></span>
        <span>{metrics.os_name} {metrics.os_version}</span>
      </div>
    {/if}
  </div>
  
  <div class="text-right">
    {#if metrics}
      <div class="text-xl font-mono text-emerald-400 font-bold">
        {formatUptime(metrics.uptime)}
      </div>
      <div class="text-xs text-zinc-500 uppercase tracking-wider font-semibold">System Uptime</div>
    {:else}
      <div class="animate-pulse text-zinc-500">Connecting...</div>
    {/if}
  </div>
</div>
