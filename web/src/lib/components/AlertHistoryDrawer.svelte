<script lang="ts">
  import type { AlertEvent } from '../types';

  let { open, onClose }: { open: boolean, onClose: () => void } = $props();

  let history: AlertEvent[] = $state([]);
  let loading = $state(false);

  async function fetchHistory() {
    loading = true;
    try {
      const res = await fetch('/api/alerts/history');
      if (res.ok) {
        const raw: AlertEvent[] = await res.json();
        history = raw.slice().reverse();
      }
    } catch { /**/ } finally {
      loading = false;
    }
  }

  $effect(() => { if (open) fetchHistory(); });

  function formatTime(ts: number): string {
    return new Date(ts * 1000).toLocaleString([], {
      month: 'short', day: 'numeric',
      hour: '2-digit', minute: '2-digit', second: '2-digit',
    });
  }
</script>

{#if open}
  <div class="anim-backdrop" onclick={onClose} role="presentation"
       style="position:fixed;inset:0;z-index:40;background:rgba(0,0,0,0.5);cursor:default"></div>

  <div class="anim-drawer nice-scroll"
       style="position:fixed;right:0;top:0;bottom:0;z-index:50;width:380px;max-width:100vw;
              background:var(--bg-1);border-left:1px solid var(--line);
              display:flex;flex-direction:column;overflow-y:auto">
    <!-- Header -->
    <div style="display:flex;justify-content:space-between;align-items:center;padding:20px 24px;border-bottom:1px solid var(--line);flex-shrink:0">
      <div>
        <div style="font-size:14px;font-weight:600;color:var(--ink)">Alert History</div>
        <div style="font-size:11px;color:var(--ink-4);margin-top:2px">Last 50 alerts · newest first</div>
      </div>
      <button onclick={onClose} class="btn" aria-label="Close" style="width:28px;height:28px;padding:0;border-radius:6px">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" stroke-width="2" stroke-linecap="round">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <!-- List -->
    <div style="flex:1;padding:16px;display:flex;flex-direction:column;gap:8px;overflow-y:auto" class="nice-scroll">
      {#if loading}
        <div style="text-align:center;padding:48px 0;color:var(--ink-4);font-size:12px">Loading…</div>
      {:else if history.length === 0}
        <div style="text-align:center;padding:64px 0;color:var(--ink-4);font-size:12px">No alerts recorded yet</div>
      {:else}
        {#each history as alert}
          {@const isCpu = alert.kind === 'cpu'}
          <div style="border-radius:8px;padding:12px;border:1px solid {isCpu ? 'var(--warm-line)' : 'rgba(244,63,94,0.25)'};background:{isCpu ? 'var(--warm-soft)' : 'var(--crit-soft)'}">
            <div style="display:flex;justify-content:space-between;align-items:flex-start;gap:8px;margin-bottom:4px">
              <span style="font-size:10px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:{isCpu ? 'var(--warm)' : 'var(--crit)'}">{isCpu ? 'CPU' : 'Memory'}</span>
              <span class="tnum font-mono" style="font-size:10px;color:var(--ink-4);flex-shrink:0">{formatTime(alert.timestamp)}</span>
            </div>
            <div style="font-size:12px;color:var(--ink-2);margin-bottom:6px">{alert.message}</div>
            <div style="display:flex;gap:12px">
              <span style="font-size:10px;color:var(--ink-4)">Value: <span style="color:var(--ink-3)">{alert.value.toFixed(1)}%</span></span>
              <span style="font-size:10px;color:var(--ink-4)">Threshold: <span style="color:var(--ink-3)">{alert.threshold}%</span></span>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
{/if}
