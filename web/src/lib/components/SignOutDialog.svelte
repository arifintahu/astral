<script lang="ts">
  let { open, onCancel, onConfirm }: {
    open: boolean;
    onCancel: () => void;
    onConfirm: () => void;
  } = $props();
</script>

<svelte:window onkeydown={(e) => { if (open && e.key === 'Escape') onCancel(); }} />

{#if open}
  <button type="button" aria-label="Cancel sign out" tabindex="-1"
          class="anim-backdrop fixed inset-0 z-40 m-0 w-full cursor-default border-0 p-0"
          style="background:rgba(0,0,0,0.62)" onclick={onCancel}></button>

  <div class="pointer-events-none fixed inset-0 z-50 flex items-center justify-center p-4">
    <div role="alertdialog" aria-modal="true" aria-label="Sign out"
         class="anim-pop pointer-events-auto flex w-[min(420px,100%)] flex-col gap-5 rounded-[20px] border p-7"
         style="background:var(--bg-1);border-color:var(--line-2);box-shadow:0 30px 80px rgba(0,0,0,0.6)">
      <span class="flex size-12 items-center justify-center rounded-full" style="background:var(--bg-2)">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4" /><polyline points="16 17 21 12 16 7" /><line x1="21" y1="12" x2="9" y2="12" />
        </svg>
      </span>
      <div class="flex flex-col gap-2">
        <span class="text-[22px] font-medium tracking-[-0.02em]">Sign out of Astral?</span>
        <span class="text-pretty text-[15px] leading-normal" style="color:var(--ink-2)">
          This ends your session on this device. You'll need your username and password to get back in.
        </span>
      </div>
      <div class="flex gap-2.5">
        <button type="button" class="btn-outline flex-1" style="height:48px" onclick={onCancel}>Cancel</button>
        <button type="button" class="flex h-12 flex-1 cursor-pointer items-center justify-center rounded-xl border-0 text-[14px] font-semibold"
                style="background:var(--crit);color:#09090b;font-family:inherit" onclick={onConfirm}>Sign out</button>
      </div>
    </div>
  </div>
{/if}
