<script lang="ts">
  import { onMount } from 'svelte';
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
        history = raw.slice().reverse(); // newest first
      }
    } catch {
      // silently fail
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (open) fetchHistory();
  });

  function formatTime(ts: number): string {
    return new Date(ts * 1000).toLocaleString([], {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  }
</script>

{#if open}
  <!-- Backdrop -->
  <button
    class="fixed inset-0 z-40 bg-black/40 backdrop-blur-sm cursor-default"
    onclick={onClose}
    tabindex="-1"
    aria-label="Close alert history"
  ></button>

  <!-- Drawer -->
  <div class="fixed right-0 top-0 bottom-0 z-50 w-full max-w-sm flex flex-col animate-fade-in">
    <div class="flex-1 bg-slate-950 border-l border-white/[0.08] flex flex-col">
      <!-- Header -->
      <div class="flex justify-between items-center px-5 py-4 border-b border-white/[0.06] flex-shrink-0">
        <div>
          <h2 class="text-sm font-semibold text-white">Alert History</h2>
          <p class="text-[11px] text-slate-500 mt-0.5">Last 50 alerts (newest first)</p>
        </div>
        <button
          onclick={onClose}
          class="w-8 h-8 flex items-center justify-center rounded-lg text-slate-500 hover:text-slate-300 hover:bg-white/[0.05] transition-colors cursor-pointer"
          aria-label="Close"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- List -->
      <div class="flex-1 overflow-y-auto custom-scrollbar p-4 space-y-2">
        {#if loading}
          <div class="flex items-center justify-center py-12 text-slate-500 text-sm">Loading…</div>
        {:else if history.length === 0}
          <div class="flex flex-col items-center justify-center py-12 gap-3 text-center">
            <div class="w-10 h-10 rounded-xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center">
              <svg class="w-5 h-5 text-slate-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <p class="text-[13px] text-slate-500">No alerts recorded yet</p>
          </div>
        {:else}
          {#each history as alert}
            <div class="rounded-xl p-3 border {alert.kind === 'cpu' ? 'border-amber-500/20 bg-amber-500/[0.04]' : 'border-rose-500/20 bg-rose-500/[0.04]'}">
              <div class="flex justify-between items-start gap-2 mb-1">
                <span class="text-[11px] font-semibold uppercase tracking-wider {alert.kind === 'cpu' ? 'text-amber-400' : 'text-rose-400'}">
                  {alert.kind === 'cpu' ? 'CPU' : 'Memory'}
                </span>
                <span class="text-[10px] text-slate-600 font-mono flex-shrink-0">{formatTime(alert.timestamp)}</span>
              </div>
              <p class="text-[12px] text-slate-300">{alert.message}</p>
              <div class="flex items-center gap-3 mt-1.5">
                <span class="text-[10px] text-slate-600">Value: <span class="text-slate-400">{alert.value.toFixed(1)}%</span></span>
                <span class="text-[10px] text-slate-600">Threshold: <span class="text-slate-400">{alert.threshold}%</span></span>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}
