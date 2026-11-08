<script lang="ts">
  import type { SystemMetrics, AlertEvent } from '../types';

  let { metrics, refreshRate, alerts, onShowSettings, onLogout, onShowAlertHistory }: {
    metrics: SystemMetrics | null;
    refreshRate: number;
    alerts: AlertEvent[];
    onShowSettings: () => void;
    onLogout: () => void;
    onShowAlertHistory: () => void;
  } = $props();

  let hasAlerts = $derived(alerts.length > 0);
</script>

<header class="flex min-h-14 flex-wrap items-center justify-between gap-3">
  <!-- Brand -->
  <div class="flex items-center gap-3">
    <svg width="28" height="28" viewBox="0 0 28 28" fill="none" aria-hidden="true">
      <circle cx="14" cy="14" r="3.2" fill="var(--accent)" />
      <circle cx="14" cy="14" r="8.5" stroke="var(--accent)" stroke-opacity=".45" stroke-width="1.2" />
      <g class="orbit"><circle cx="22.5" cy="14" r="2" fill="var(--accent)" /></g>
    </svg>
    <span class="text-[18px] font-semibold tracking-[-0.02em]">Astral</span>
    {#if metrics}
      <span class="font-mono rounded-full border px-2.5 py-[5px] text-[12px]"
            style="color:var(--ink-2);border-color:var(--line)">{metrics.hostname}</span>
    {/if}
  </div>

  <!-- Status + actions -->
  <div class="flex items-center gap-1">
    {#if metrics}
      <div class="mr-2 flex h-9 items-center gap-2 rounded-full border px-3.5" style="border-color:var(--line)">
        <span class="relative inline-block size-2" aria-hidden="true">
          <span class="ping absolute inset-0 rounded-full" style="background:var(--accent)"></span>
          <span class="absolute inset-0 rounded-full" style="background:var(--accent)"></span>
        </span>
        <span class="font-mono text-[12px] tracking-[0.06em]">LIVE · {refreshRate}s</span>
      </div>
    {:else}
      <div class="mr-2 flex h-9 items-center gap-2 rounded-full border px-3.5" style="border-color:var(--line)">
        <span class="size-2 rounded-full" style="background:var(--warm);animation:pulseDot 1s infinite"></span>
        <span class="font-mono text-[12px] tracking-[0.06em]" style="color:var(--warm)">CONNECTING</span>
      </div>
    {/if}

    <button type="button" class="icon-btn relative" aria-label="Alert history" onclick={onShowAlertHistory}>
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9" /><path d="M13.7 21a2 2 0 01-3.4 0" />
      </svg>
      {#if hasAlerts}
        <span class="absolute right-[9px] top-[9px] size-[9px] rounded-full border-2"
              style="background:var(--crit);border-color:var(--bg)"></span>
      {/if}
    </button>

    <button type="button" class="icon-btn" aria-label="Settings" onclick={onShowSettings}>
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <line x1="4" y1="7" x2="20" y2="7" /><line x1="4" y1="17" x2="20" y2="17" />
        <circle cx="9" cy="7" r="2.4" fill="var(--bg)" /><circle cx="15" cy="17" r="2.4" fill="var(--bg)" />
      </svg>
    </button>

    <button type="button" class="icon-btn" aria-label="Sign out" onclick={onLogout}>
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4" /><polyline points="16 17 21 12 16 7" /><line x1="21" y1="12" x2="9" y2="12" />
      </svg>
    </button>
  </div>
</header>
