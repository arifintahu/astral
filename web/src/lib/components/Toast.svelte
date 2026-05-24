<script lang="ts">
  import type { AlertEvent } from '../types';
  let { alerts, onDismiss }: { alerts: AlertEvent[], onDismiss: (i: number) => void } = $props();
</script>

{#if alerts.length > 0}
  <div style="position:fixed;top:16px;right:16px;z-index:60;display:flex;flex-direction:column;gap:10px;max-width:340px">
    {#each alerts as alert, i}
      {@const isCpu = alert.kind === 'cpu'}
      <div class="surface anim-toast" style="padding:14px 16px;border-color:{isCpu ? 'var(--warm-line)' : 'rgba(244,63,94,0.3)'}">
        <div style="display:flex;align-items:flex-start;gap:12px">
          <!-- Icon -->
          <div style="width:28px;height:28px;border-radius:6px;flex-shrink:0;display:flex;align-items:center;justify-content:center;background:{isCpu ? 'var(--warm-soft)' : 'var(--crit-soft)'}">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
                 stroke="{isCpu ? 'var(--warm)' : 'var(--crit)'}" stroke-width="2" stroke-linecap="round">
              <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
              <line x1="12" y1="9" x2="12" y2="13"/>
              <line x1="12" y1="17" x2="12.01" y2="17"/>
            </svg>
          </div>
          <div style="flex:1;min-width:0">
            <div style="font-size:11px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:{isCpu ? 'var(--warm)' : 'var(--crit)'};margin-bottom:2px">
              {isCpu ? 'CPU Alert' : 'Memory Alert'}
            </div>
            <div style="font-size:12px;color:var(--ink-2)">{alert.message}</div>
          </div>
          <button onclick={() => onDismiss(i)} aria-label="Dismiss"
                  style="color:var(--ink-4);background:none;border:none;cursor:pointer;padding:0;flex-shrink:0;display:flex">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
      </div>
    {/each}
  </div>
{/if}
