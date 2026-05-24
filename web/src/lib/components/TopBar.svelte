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

  let alertCount = $derived(alerts.length);

  function fmtUptime(s: number): string {
    const d = Math.floor(s / 86400);
    const h = Math.floor((s % 86400) / 3600);
    const m = Math.floor((s % 3600) / 60);
    const parts: string[] = [];
    if (d > 0) parts.push(`${d}d`);
    parts.push(`${h}h`);
    parts.push(`${m}m`);
    return parts.join(' ');
  }
</script>

<header class="surface anim-fade-up mb-6" style="border-radius:14px">
  <div class="flex items-center justify-between" style="padding:14px 20px">

    <!-- Left: Sigil + name + host info -->
    <div class="flex items-center gap-3">
      <!-- Sigil -->
      <div class="flex items-center justify-center flex-shrink-0"
           style="width:36px;height:36px;border-radius:10px;background:var(--bg-2);border:1px solid var(--line-2)">
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
          <circle cx="10" cy="10" r="2.5"  stroke="var(--accent)" stroke-width="1.5"/>
          <circle cx="10" cy="10" r="6"    stroke="var(--accent)" stroke-width="1"    stroke-opacity="0.5"/>
          <circle cx="10" cy="10" r="9"    stroke="var(--accent)" stroke-width="0.75" stroke-opacity="0.25"/>
        </svg>
      </div>

      <div>
        <div class="flex items-center gap-2">
          <span style="font-size:15px;font-weight:600;color:var(--ink)">Astral</span>
          <span class="tnum font-mono" style="font-size:10px;color:var(--ink-4)">v1.1.0</span>
        </div>
        {#if metrics}
          <div class="flex items-center gap-1.5" style="margin-top:2px">
            <span class="font-mono" style="font-size:12px;color:var(--ink-2)">{metrics.hostname}</span>
            <span style="color:var(--ink-4)">·</span>
            <span style="font-size:12px;color:var(--ink-3)">{metrics.os_name} {metrics.os_version}</span>
          </div>
        {/if}
      </div>
    </div>

    <!-- Right: pills + icon buttons -->
    <div class="flex items-center gap-2">
      {#if metrics}
        <!-- Live / Alert pill -->
        {#if alertCount > 0}
          <button onclick={onShowAlertHistory} class="flex items-center gap-1.5 cursor-pointer"
                  style="padding:6px 12px;border-radius:20px;background:var(--crit-soft);border:1px solid rgba(244,63,94,0.25)">
            <div style="width:6px;height:6px;border-radius:50%;background:var(--crit);animation:pulseDot 1.5s infinite"></div>
            <span style="font-size:11px;font-weight:500;color:var(--crit)">{alertCount} alert{alertCount !== 1 ? 's' : ''}</span>
          </button>
        {:else}
          <div class="flex items-center gap-1.5"
               style="padding:6px 12px;border-radius:20px;background:var(--bg-2);border:1px solid var(--line)">
            <div style="width:6px;height:6px;border-radius:50%;background:var(--ok);animation:pulseDot 2s infinite"></div>
            <span style="font-size:11px;font-weight:500;color:var(--ok)">Live · {refreshRate}s</span>
          </div>
        {/if}

        <!-- Uptime pill -->
        <div class="flex items-center gap-1.5"
             style="padding:6px 12px;border-radius:20px;background:var(--bg-2);border:1px solid var(--line)">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" stroke-width="2" stroke-linecap="round">
            <circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>
          </svg>
          <span class="tnum font-mono" style="font-size:11px;color:var(--ink-2)">{fmtUptime(metrics.uptime)}</span>
        </div>
      {:else}
        <div class="flex items-center gap-1.5"
             style="padding:6px 12px;border-radius:20px;background:var(--bg-2);border:1px solid var(--line)">
          <div style="width:6px;height:6px;border-radius:50%;background:var(--warm);animation:pulseDot 1s infinite"></div>
          <span style="font-size:11px;font-weight:500;color:var(--warm)">Connecting</span>
        </div>
      {/if}

      <!-- Bell -->
      <button onclick={onShowAlertHistory} aria-label="Alert history" class="btn"
              style="width:32px;height:32px;padding:0;border-radius:8px">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" stroke-width="1.5" stroke-linecap="round">
          <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/>
          <path d="M13.73 21a2 2 0 01-3.46 0"/>
        </svg>
      </button>

      <!-- Gear -->
      <button onclick={onShowSettings} aria-label="Settings" class="btn"
              style="width:32px;height:32px;padding:0;border-radius:8px">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" stroke-width="1.5" stroke-linecap="round">
          <circle cx="12" cy="12" r="3"/>
          <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/>
        </svg>
      </button>

      <!-- Logout -->
      <button onclick={onLogout} aria-label="Logout" class="btn"
              style="width:32px;height:32px;padding:0;border-radius:8px">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" stroke-width="1.5" stroke-linecap="round">
          <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4"/>
          <polyline points="16 17 21 12 16 7"/>
          <line x1="21" y1="12" x2="9" y2="12"/>
        </svg>
      </button>
    </div>
  </div>
</header>
