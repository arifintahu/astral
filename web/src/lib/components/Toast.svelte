<script lang="ts">
  import type { AlertEvent } from '../types';

  let { alerts, onDismiss }: { alerts: AlertEvent[]; onDismiss: (i: number) => void } = $props();

  const BOLT = 'M13 2L4 14h7l-1 8 9-12h-7l1-8z';
  const CHIP = 'M4 7h16v10H4z M8 7V4 M12 7V4 M16 7V4 M8 20v-3 M12 20v-3 M16 20v-3';
</script>

{#if alerts.length > 0}
  <div class="fixed right-[clamp(16px,4vw,32px)] top-[72px] z-[60] flex w-[min(380px,calc(100%-32px))] flex-col gap-3"
       aria-live="assertive">
    {#each alerts as a, i (a.timestamp + a.kind)}
      {@const isCpu = a.kind === 'cpu'}
      <div role="alert" class="anim-toast flex items-start gap-3.5 rounded-[14px] border py-4 pl-4 pr-2"
           style="background:var(--bg-2);border-color:color-mix(in srgb, var(--crit) 45%, transparent);box-shadow:0 18px 50px rgba(0,0,0,0.55)">
        <span class="flex size-9 flex-none items-center justify-center rounded-full"
              style="background:color-mix(in srgb, var(--crit) 18%, transparent);color:var(--crit)">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d={isCpu ? BOLT : CHIP} />
          </svg>
        </span>
        <div class="flex min-w-0 flex-1 flex-col gap-[3px] pt-0.5">
          <span class="text-[15px] font-medium">{isCpu ? 'CPU' : 'Memory'} above threshold</span>
          <span class="font-mono text-[13px]" style="color:var(--ink-2)">{a.message}</span>
        </div>
        <button type="button" class="icon-btn flex-none" style="width:44px;height:44px;border-radius:10px;color:var(--ink-3)"
                aria-label="Dismiss alert" onclick={() => onDismiss(i)}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <line x1="6" y1="6" x2="18" y2="18" /><line x1="18" y1="6" x2="6" y2="18" />
          </svg>
        </button>
      </div>
    {/each}
  </div>
{/if}
