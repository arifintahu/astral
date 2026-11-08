<script lang="ts">
  import type { TileData } from '../types';
  import LedMeter from './LedMeter.svelte';

  let { tile, active, onSelect }: {
    tile: TileData;
    active: boolean;
    onSelect: () => void;
  } = $props();
</script>

<button
  type="button"
  class="tile flex min-h-[clamp(172px,16vw,204px)] flex-col gap-3.5 p-[clamp(16px,2.2vw,24px)]"
  aria-pressed={active}
  onclick={onSelect}
>
  <span class="tile-bar absolute inset-x-0 top-0 h-0.5"></span>

  <span class="flex items-center justify-between gap-2">
    <span class="eyebrow">{tile.label}</span>
    <span class="font-mono text-right text-[12px]" style="color:var(--ink-3)">{tile.meta}</span>
  </span>

  <span class="flex items-baseline gap-1.5">
    <span class="big">{tile.value}</span>
    <span class="text-[clamp(14px,1.6vw,18px)]" style="color:var(--ink-3)">{tile.unit}</span>
  </span>

  <span class="font-mono text-[13px]" style="color:var(--ink-2)">{tile.sub}</span>

  <span class="mt-auto block">
    <LedMeter pct={tile.pct} zones={tile.zones} color={tile.meterColor} />
  </span>
</button>
