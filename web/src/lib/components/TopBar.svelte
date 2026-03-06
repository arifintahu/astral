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

<div class="flex justify-between items-center p-6 glass-panel mb-6">
  <div>
    <h1 class="text-3xl font-extrabold tracking-tight text-white mb-1">
      <span class="bg-clip-text text-transparent bg-gradient-to-r from-cyan-400 via-blue-500 to-purple-600 drop-shadow-lg">Astral</span>
    </h1>
    {#if metrics}
      <div class="text-sm text-slate-400 flex items-center gap-2 font-medium">
        <span class="text-slate-200">{metrics.hostname}</span>
        <span class="w-1.5 h-1.5 bg-cyan-500 rounded-full shadow-[0_0_8px_rgba(6,182,212,0.8)]"></span>
        <span class="text-slate-400">{metrics.os_name} {metrics.os_version}</span>
      </div>
    {/if}
  </div>
  
  <div class="text-right">
    {#if metrics}
      <div class="text-2xl font-mono text-cyan-300 font-bold drop-shadow-md">
        {formatUptime(metrics.uptime)}
      </div>
      <div class="text-xs text-slate-500 uppercase tracking-widest font-bold mt-1">System Uptime</div>
    {:else}
      <div class="animate-pulse text-slate-500 font-mono">Connecting...</div>
    {/if}
  </div>
</div>
