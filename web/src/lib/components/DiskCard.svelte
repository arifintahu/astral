<script lang="ts">
  import type { DiskInfo } from '../types';

  let { disks }: { disks: DiskInfo[] } = $props();

  function sevColor(v: number): string {
    if (v >= 90) return 'var(--crit)';
    if (v >= 70) return 'var(--warm)';
    return 'var(--accent)';
  }

  function fmt(b: number): string {
    if (b >= 1099511627776) return (b / 1099511627776).toFixed(1) + ' TB';
    if (b >= 1073741824)    return (b / 1073741824).toFixed(1) + ' GB';
    if (b >= 1048576)       return (b / 1048576).toFixed(0) + ' MB';
    return (b / 1024).toFixed(0) + ' KB';
  }

  function fmtRate(b: number): string {
    if (b >= 1048576) return (b / 1048576).toFixed(1) + ' MB/s';
    if (b >= 1024)    return (b / 1024).toFixed(0) + ' KB/s';
    return b + ' B/s';
  }

  let primary   = $derived(disks[0] ?? null);
  let totalUsed = $derived(primary ? primary.total_space - primary.available_space : 0);
  let usedPct   = $derived(primary && primary.total_space > 0
    ? (totalUsed / primary.total_space) * 100 : 0);
</script>

<div class="surface card-hover h-full flex flex-col relative overflow-hidden" style="padding:18px 20px">
  {#if primary}
    {@const barColor = sevColor(usedPct)}
    <div class="absolute top-0 left-0 right-0 rounded-t-[14px]"
         style="height:2px;background:linear-gradient(90deg,{barColor}80,{barColor}00)"></div>

    <div class="flex items-center justify-between mb-4">
      <span class="eyebrow">Storage</span>
      <span class="tnum font-mono" style="font-size:10px;padding:3px 8px;border-radius:6px;background:var(--bg-2);color:var(--ink-4);border:1px solid var(--line)">{disks.length} mount{disks.length !== 1 ? 's' : ''}</span>
    </div>

    <!-- Large % + used/total -->
    <div class="flex items-end justify-between mb-4">
      <div class="flex items-baseline gap-1">
        <span class="tnum font-mono font-semibold" style="font-size:34px;color:var(--ink);line-height:1">{usedPct.toFixed(0)}</span>
        <span style="font-size:14px;color:var(--ink-3)">%</span>
      </div>
      <div class="text-right">
        <div class="tnum font-mono" style="font-size:11px;color:var(--ink-2)">{fmt(totalUsed)}</div>
        <div class="tnum font-mono" style="font-size:11px;color:var(--ink-4)">of {fmt(primary.total_space)}</div>
      </div>
    </div>

    <!-- Per-mount bar -->
    <div class="overflow-hidden mb-4" style="height:6px;background:var(--bg-2);border:1px solid var(--line);border-radius:3px;display:flex">
      {#each disks as disk, i}
        {@const pct = primary.total_space > 0 ? ((disk.total_space - disk.available_space) / primary.total_space) * 100 : 0}
        <div class="h-full transition-all duration-700"
             style="width:{pct}%;background:{i === 0 ? sevColor(usedPct) : 'var(--ink-3)'};opacity:{i === 0 ? 1 : 0.4}"></div>
      {/each}
    </div>

    <!-- I/O rates + mount path -->
    <div class="flex items-center gap-4 mt-auto">
      <div class="flex items-center gap-1">
        <span style="font-size:10px;color:var(--accent)">↓</span>
        <span class="tnum font-mono" style="font-size:11px;color:var(--ink-2)">{fmtRate(primary.read_bytes)}</span>
      </div>
      <div class="flex items-center gap-1">
        <span style="font-size:10px;color:var(--warm)">↑</span>
        <span class="tnum font-mono" style="font-size:11px;color:var(--ink-2)">{fmtRate(primary.written_bytes)}</span>
      </div>
      <span class="font-mono truncate ml-auto" style="font-size:10px;color:var(--ink-4)">{primary.mount_point}</span>
    </div>
  {:else}
    <div class="flex items-center justify-center flex-1" style="color:var(--ink-4);font-size:12px">No disks detected</div>
  {/if}
</div>
