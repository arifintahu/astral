<script lang="ts">
  import type { AlertEvent } from '../types';
  import Drawer from './Drawer.svelte';

  let { open, onClose, onAdjust }: {
    open: boolean;
    onClose: () => void;
    /** Jump to Settings so the thresholds can be changed. */
    onAdjust: () => void;
  } = $props();

  const BOLT = 'M13 2L4 14h7l-1 8 9-12h-7l1-8z';
  const CHIP = 'M4 7h16v10H4z M8 7V4 M12 7V4 M16 7V4 M8 20v-3 M12 20v-3 M16 20v-3';

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

<Drawer {open} {onClose} title="Alert history" subtitle="Last 50 alerts · newest first" width={430}>
  {#snippet children()}
    <div class="px-6">
      {#if loading}
        <div class="py-16 text-center text-[13px]" style="color:var(--ink-3)">Loading…</div>
      {:else if history.length === 0}
        <div class="flex flex-col items-center gap-3 py-20 text-center">
          <span class="flex size-12 items-center justify-center rounded-full" style="background:var(--bg-2);color:var(--accent)">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><polyline points="20 6 9 17 4 12" /></svg>
          </span>
          <span class="text-[15px] font-medium">No alerts recorded yet</span>
          <span class="text-[13px]" style="color:var(--ink-3)">Everything has stayed inside its limits.</span>
        </div>
      {:else}
        {#each history as a (a.timestamp + a.kind)}
          {@const isCpu = a.kind === 'cpu'}
          <div class="flex gap-3.5 border-b py-5" style="border-color:var(--line)">
            <span class="flex size-10 flex-none items-center justify-center rounded-full"
                  style="background:color-mix(in srgb, var(--crit) 15%, transparent);color:var(--crit)">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d={isCpu ? BOLT : CHIP} />
              </svg>
            </span>
            <div class="flex min-w-0 flex-1 flex-col gap-2">
              <div class="flex items-baseline justify-between gap-3">
                <span class="text-[15px] font-medium">{isCpu ? 'CPU' : 'Memory'} above threshold</span>
                <span class="font-mono whitespace-nowrap text-[12px]" style="color:var(--ink-3)">{formatTime(a.timestamp)}</span>
              </div>
              <span class="text-pretty text-[14px] leading-[1.45]" style="color:var(--ink-2)">{a.message}</span>
              <div class="flex flex-wrap gap-2">
                <span class="font-mono tnum rounded-full border px-2.5 py-1 text-[12px]" style="border-color:var(--line)">Peak {a.value.toFixed(1)}%</span>
                <span class="font-mono tnum rounded-full border px-2.5 py-1 text-[12px]" style="border-color:var(--line);color:var(--ink-3)">Limit {a.threshold}%</span>
              </div>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  {/snippet}

  {#snippet footer()}
    <div class="flex items-center justify-end px-3 py-3">
      <button type="button" class="h-11 cursor-pointer rounded-xl border-0 bg-transparent px-4 text-[14px] font-medium"
              style="color:var(--accent);font-family:inherit" onclick={onAdjust}>Adjust limits</button>
    </div>
  {/snippet}
</Drawer>
