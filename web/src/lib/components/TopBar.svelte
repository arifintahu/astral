<script lang="ts">
  import type { SystemMetrics, AlertEvent } from '../types';
  import Settings from './Settings.svelte';

  let { metrics, refreshRate, alerts, onRefreshRateChange, onLogout, onShowAlertHistory }: {
    metrics: SystemMetrics | null,
    refreshRate: number,
    alerts: AlertEvent[],
    onRefreshRateChange: (rate: number) => void,
    onLogout: () => void,
    onShowAlertHistory: () => void,
  } = $props();

  let activeAlertCount = $derived(alerts.length);

  function formatUptime(uptime: number): string {
    const days = Math.floor(uptime / 86400);
    const hours = Math.floor((uptime % 86400) / 3600);
    const minutes = Math.floor((uptime % 3600) / 60);
    const parts: string[] = [];
    if (days > 0) parts.push(`${days}d`);
    parts.push(`${hours}h`);
    parts.push(`${minutes}m`);
    return parts.join(' ');
  }
</script>

<header class="glass-panel mb-6 animate-fade-in-up relative z-30">
  <div class="flex justify-between items-center px-6 py-5">
    <div class="flex items-center gap-4">
      <!-- Logo -->
      <div class="relative flex items-center justify-center w-9 h-9 rounded-xl bg-gradient-to-br from-purple-500/20 to-cyan-500/20 border border-white/[0.08]">
        <svg class="w-5 h-5 text-cyan-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="3" />
          <path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4M4.22 19.78l2.83-2.83M16.95 7.05l2.83-2.83" stroke-linecap="round"/>
        </svg>
      </div>

      <div>
        <h1 class="text-xl font-bold tracking-tight text-white">
          <span class="bg-clip-text text-transparent bg-gradient-to-r from-cyan-300 via-blue-400 to-purple-500">Astral</span>
        </h1>
        {#if metrics}
          <div class="flex items-center gap-2 mt-0.5">
            <span class="text-[13px] text-slate-300 font-medium">{metrics.hostname}</span>
            <span class="text-slate-600">&#183;</span>
            <span class="text-[13px] text-slate-500">{metrics.os_name} {metrics.os_version}</span>
          </div>
        {/if}
      </div>
    </div>

    <div class="flex items-center gap-4">
      {#if metrics}
        <!-- T-13: status indicator changes to red when alerts are active -->
        <div class="flex items-center gap-2">
          {#if activeAlertCount > 0}
            <button
              onclick={onShowAlertHistory}
              class="flex items-center gap-2 cursor-pointer group"
              title="View alert history"
            >
              <div class="w-2 h-2 rounded-full bg-rose-400 animate-pulse"></div>
              <span class="text-[11px] text-rose-400/90 font-medium uppercase tracking-wider">Alert</span>
              <span class="flex items-center justify-center min-w-[18px] h-[18px] px-1 text-[10px] font-bold bg-rose-500/20 border border-rose-500/30 text-rose-300 rounded-full">
                {activeAlertCount}
              </span>
            </button>
          {:else}
            <div class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse-glow"></div>
            <span class="text-[11px] text-emerald-400/80 font-medium uppercase tracking-wider">Live</span>
          {/if}
        </div>

        <div class="h-6 w-px bg-white/[0.08]"></div>

        <!-- Uptime -->
        <div class="text-right hidden sm:block">
          <div class="text-lg font-mono text-slate-200 font-semibold tabular-nums tracking-tight">
            {formatUptime(metrics.uptime)}
          </div>
          <div class="text-[10px] text-slate-500 uppercase tracking-[0.15em] font-semibold mt-0.5">Uptime</div>
        </div>

        <div class="h-6 w-px bg-white/[0.08] hidden sm:block"></div>
      {:else}
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full bg-amber-400 animate-pulse"></div>
          <span class="text-[11px] text-amber-400/80 font-medium uppercase tracking-wider font-mono">Connecting</span>
        </div>
      {/if}

      <Settings {refreshRate} {onRefreshRateChange} {onLogout} />
    </div>
  </div>
</header>
