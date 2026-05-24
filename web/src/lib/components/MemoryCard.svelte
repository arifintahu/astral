<script lang="ts">
  let { used, total, swap_used, swap_total }: {
    used: number; total: number;
    swap_used: number; swap_total: number;
  } = $props();

  function sevColor(v: number): string {
    if (v >= 90) return 'var(--crit)';
    if (v >= 70) return 'var(--warm)';
    return 'var(--accent)';
  }

  function fmt(b: number): string {
    if (b >= 1073741824) return (b / 1073741824).toFixed(1) + ' GB';
    if (b >= 1048576) return (b / 1048576).toFixed(0) + ' MB';
    return (b / 1024).toFixed(0) + ' KB';
  }

  let pct      = $derived(total > 0 ? (used / total) * 100 : 0);
  let barColor = $derived(sevColor(pct));
  let usedW    = $derived(total > 0 ? (used / total) * 100 : 0);
  let swapW    = $derived(total > 0 ? (swap_used / total) * 100 : 0);
</script>

<div class="surface card-hover h-full flex flex-col relative overflow-hidden" style="padding:18px 20px">
  <div class="absolute top-0 left-0 right-0 rounded-t-[14px]"
       style="height:2px;background:linear-gradient(90deg,{barColor}80,{barColor}00)"></div>

  <div class="flex items-center justify-between mb-4">
    <span class="eyebrow">Memory</span>
    <span class="tnum font-mono" style="font-size:10px;padding:3px 8px;border-radius:6px;background:var(--bg-2);color:var(--ink-4);border:1px solid var(--line)">Swap {fmt(swap_used)}</span>
  </div>

  <!-- Large % -->
  <div class="flex items-end justify-between mb-4">
    <div class="flex items-baseline gap-1">
      <span class="tnum font-mono font-semibold" style="font-size:34px;color:var(--ink);line-height:1">{pct.toFixed(0)}</span>
      <span style="font-size:14px;color:var(--ink-3)">%</span>
    </div>
    <div class="text-right">
      <div class="tnum font-mono" style="font-size:11px;color:var(--ink-2)">{fmt(used)}</div>
      <div class="tnum font-mono" style="font-size:11px;color:var(--ink-4)">of {fmt(total)}</div>
    </div>
  </div>

  <!-- Stacked bar -->
  <div class="overflow-hidden mb-3" style="height:6px;background:var(--bg-2);border:1px solid var(--line);border-radius:3px">
    <div class="h-full flex">
      <div class="h-full transition-all duration-700"
           style="width:{usedW}%;background:{barColor};border-radius:3px 0 0 3px"></div>
      <div class="h-full transition-all duration-700"
           style="width:{swapW}%;background:var(--warm);opacity:0.6"></div>
    </div>
  </div>

  <!-- Legend -->
  <div class="flex gap-4">
    <div class="flex items-center gap-1.5">
      <div style="width:10px;height:6px;border-radius:2px;background:{barColor}"></div>
      <span style="font-size:10px;color:var(--ink-3)">Used</span>
    </div>
    <div class="flex items-center gap-1.5">
      <div style="width:10px;height:6px;border-radius:2px;background:var(--warm);opacity:0.7"></div>
      <span style="font-size:10px;color:var(--ink-3)">Swap</span>
    </div>
    <div class="flex items-center gap-1.5">
      <div style="width:10px;height:6px;border-radius:2px;background:var(--bg-2);border:1px solid var(--line-2)"></div>
      <span style="font-size:10px;color:var(--ink-3)">Free</span>
    </div>
  </div>
</div>
