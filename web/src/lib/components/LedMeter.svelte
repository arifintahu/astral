<script lang="ts">
  const N = 28;

  let { pct, zones = true, color = 'var(--accent)' }: {
    pct: number;
    /** Colour lit ticks by position (lime → amber → red) instead of a single colour. */
    zones?: boolean;
    color?: string;
  } = $props();

  let lit = $derived(
    pct > 0 ? Math.max(1, Math.round((Math.min(Math.max(pct, 0), 100) / 100) * N)) : 0
  );

  function tickColor(j: number): string {
    if (j >= lit) return 'rgba(255,255,255,0.09)';
    if (!zones) return color;
    const pos = (j + 1) / N;
    if (pos > 0.9) return 'var(--crit)';
    if (pos > 0.7) return 'var(--warm)';
    return 'var(--accent)';
  }
</script>

<div class="flex h-3.5 gap-[3px]" aria-hidden="true">
  {#each Array(N) as _, j}
    <span class="flex-1 rounded-[1.5px] transition-colors duration-300" style="background:{tickColor(j)}"></span>
  {/each}
</div>
