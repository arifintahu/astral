<script lang="ts">
  import MiniChart from './MiniChart.svelte';

  let { tx, rx, txHistory, rxHistory }: {
    tx: number; rx: number;
    txHistory: number[]; rxHistory: number[];
  } = $props();

  function fmtRate(bps: number): string {
    if (bps >= 1048576) return (bps / 1048576).toFixed(1) + ' MB/s';
    if (bps >= 1024)    return (bps / 1024).toFixed(0) + ' KB/s';
    return bps.toFixed(0) + ' B/s';
  }

  let txMax = $derived(Math.max(...txHistory, 1));
  let rxMax = $derived(Math.max(...rxHistory, 1));
</script>

<div class="surface card-hover h-full flex flex-col" style="padding:18px 20px">
  <div class="flex items-center justify-between mb-4">
    <span class="eyebrow">Network</span>
  </div>

  <div class="flex flex-col gap-3 flex-1">
    {#each [
      { label: 'Out ↑', value: tx, history: txHistory, max: txMax, color: 'var(--accent)' },
      { label: 'In ↓',  value: rx, history: rxHistory, max: rxMax, color: 'var(--warm)' },
    ] as row}
      <div class="surface-2 flex items-center gap-3" style="padding:10px 12px;flex:1;min-height:0">
        <div style="min-width:56px;flex-shrink:0">
          <div class="eyebrow" style="margin-bottom:3px">{row.label}</div>
          <div class="tnum font-mono font-medium" style="font-size:13px;color:var(--ink)">{fmtRate(row.value)}</div>
        </div>
        <div class="flex-1" style="min-height:36px">
          <MiniChart data={row.history} max={row.max} color={row.color} w={120} h={40} />
        </div>
      </div>
    {/each}
  </div>
</div>
