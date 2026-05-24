<script lang="ts">
  import MiniChart from './MiniChart.svelte';

  let { usage, cores, history, load }: {
    usage: number;
    cores: number;
    history: number[];
    load: [number, number, number];
  } = $props();

  function sevColor(v: number): string {
    if (v >= 90) return 'var(--crit)';
    if (v >= 70) return 'var(--warm)';
    return 'var(--accent)';
  }

  const R = 26;
  const CIRC = 2 * Math.PI * R;
  let dashOffset = $derived(CIRC - (usage / 100) * CIRC);
  let accentColor = $derived(sevColor(usage));
</script>

<div class="surface card-hover h-full flex flex-col relative overflow-hidden" style="padding:18px 20px">
  <!-- Top accent hairline -->
  <div class="absolute top-0 left-0 right-0 rounded-t-[14px]"
       style="height:2px;background:linear-gradient(90deg,{accentColor}80,{accentColor}00)"></div>

  <!-- Header -->
  <div class="flex items-center justify-between mb-4">
    <span class="eyebrow">CPU</span>
    <span class="tnum font-mono" style="font-size:10px;padding:3px 8px;border-radius:6px;background:var(--bg-2);color:var(--ink-4);border:1px solid var(--line)">{cores} cores</span>
  </div>

  <!-- Body -->
  <div class="flex items-center gap-4 flex-1 min-h-0">
    <!-- Ring gauge -->
    <div class="relative flex-shrink-0" style="width:64px;height:64px">
      <svg width="64" height="64" viewBox="0 0 64 64" style="transform:rotate(-90deg);display:block">
        <circle cx="32" cy="32" r={R} fill="none" stroke="var(--line-2)" stroke-width="5" />
        <circle cx="32" cy="32" r={R} fill="none"
                stroke={accentColor}
                stroke-width="5"
                stroke-linecap="round"
                stroke-dasharray={CIRC}
                stroke-dashoffset={dashOffset}
                style="transition:stroke-dashoffset 0.6s ease,stroke 0.4s ease" />
      </svg>
      <div class="absolute inset-0 flex flex-col items-center justify-center">
        <span class="tnum font-mono font-semibold" style="font-size:16px;color:var(--ink);line-height:1">{usage.toFixed(0)}</span>
        <span style="font-size:9px;color:var(--ink-4)">%</span>
      </div>
    </div>

    <!-- Right: sparkline + load avg -->
    <div class="flex-1 flex flex-col gap-2 min-w-0" style="min-height:0">
      <div style="flex:1;min-height:36px">
        <MiniChart data={history} max={100} color={accentColor} w={140} h={44} />
      </div>
      {#if load[0] > 0}
        <div class="flex gap-3">
          {#each [['1m', load[0]], ['5m', load[1]], ['15m', load[2]]] as [label, val]}
            <div class="flex flex-col items-center">
              <span class="tnum font-mono" style="font-size:11px;color:var(--ink-2)">{(val as number).toFixed(2)}</span>
              <span style="font-size:9px;color:var(--ink-4)">{label}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>
