<script lang="ts">
  import type { Snippet } from 'svelte';

  let { open, title, subtitle, width = 440, onClose, children, footer }: {
    open: boolean;
    title: string;
    subtitle?: string;
    width?: number;
    onClose: () => void;
    children: Snippet;
    footer?: Snippet;
  } = $props();
</script>

<svelte:window onkeydown={(e) => { if (open && e.key === 'Escape') onClose(); }} />

{#if open}
  <button type="button" aria-label="Close panel" tabindex="-1"
          class="anim-backdrop fixed inset-0 z-40 m-0 w-full cursor-default border-0 p-0"
          style="background:rgba(0,0,0,0.62)" onclick={onClose}></button>

  <div role="dialog" aria-modal="true" aria-label={title}
         class="anim-drawer fixed inset-y-0 right-0 z-50 flex max-w-full flex-col border-l"
         style="width:{width}px;background:var(--bg-1);border-color:var(--line-2)">
    <div class="flex flex-none items-center justify-between gap-3 border-b py-5 pl-6 pr-3" style="border-color:var(--line)">
      <div class="flex flex-col gap-[3px]">
        <span class="text-[20px] font-medium tracking-[-0.02em]">{title}</span>
        {#if subtitle}<span class="text-[13px]" style="color:var(--ink-3)">{subtitle}</span>{/if}
      </div>
      <button type="button" class="icon-btn" aria-label="Close {title.toLowerCase()}" onclick={onClose}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true">
          <line x1="6" y1="6" x2="18" y2="18" /><line x1="18" y1="6" x2="6" y2="18" />
        </svg>
      </button>
    </div>

    <div class="nice-scroll min-h-0 flex-1 overflow-y-auto">{@render children()}</div>

    {#if footer}
      <div class="flex-none border-t" style="border-color:var(--line)">{@render footer()}</div>
    {/if}
  </div>
{/if}
