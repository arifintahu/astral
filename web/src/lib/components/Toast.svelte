<script lang="ts">
  import type { AlertEvent } from '../types';

  let { alerts, onDismiss }: { alerts: AlertEvent[], onDismiss: (index: number) => void } = $props();
</script>

{#if alerts.length > 0}
  <div class="fixed top-4 right-4 z-50 flex flex-col gap-3 max-w-sm">
    {#each alerts as alert, i}
      <div
        class="animate-fade-in-up bg-slate-900/95 backdrop-blur-xl border rounded-xl p-4 shadow-2xl
               {alert.kind === 'cpu' ? 'border-amber-500/30 shadow-amber-500/10' : 'border-rose-500/30 shadow-rose-500/10'}"
      >
        <div class="flex items-start gap-3">
          <!-- Icon -->
          <div class="flex-shrink-0 mt-0.5">
            {#if alert.kind === 'cpu'}
              <div class="w-8 h-8 rounded-lg bg-amber-500/10 border border-amber-500/20 flex items-center justify-center">
                <svg class="w-4 h-4 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                </svg>
              </div>
            {:else}
              <div class="w-8 h-8 rounded-lg bg-rose-500/10 border border-rose-500/20 flex items-center justify-center">
                <svg class="w-4 h-4 text-rose-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                </svg>
              </div>
            {/if}
          </div>

          <div class="flex-1 min-w-0">
            <div class="text-[11px] font-semibold uppercase tracking-wider {alert.kind === 'cpu' ? 'text-amber-400' : 'text-rose-400'} mb-0.5">
              {alert.kind === 'cpu' ? 'CPU Alert' : 'Memory Alert'}
            </div>
            <div class="text-sm text-slate-300">{alert.message}</div>
          </div>

          <button
            onclick={() => onDismiss(i)}
            aria-label="Dismiss alert"
            class="flex-shrink-0 text-slate-600 hover:text-slate-400 transition-colors cursor-pointer"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    {/each}
  </div>
{/if}
