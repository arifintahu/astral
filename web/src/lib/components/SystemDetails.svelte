<script lang="ts">
  import type { DiskInfo } from '../types';
  import { fmtBytes, zoneColor } from '../format';

  let { load, cores, swapUsed, swapTotal, disks }: {
    load: [number, number, number];
    cores: number;
    swapUsed: number;
    swapTotal: number;
    disks: DiskInfo[];
  } = $props();

  const GB = 1073741824;

  let loads = $derived([
    { k: '1 min',  v: load[0].toFixed(2) },
    { k: '5 min',  v: load[1].toFixed(2) },
    { k: '15 min', v: load[2].toFixed(2) },
  ]);

  let swapPct = $derived(swapTotal > 0 ? (swapUsed / swapTotal) * 100 : 0);

  let mounts = $derived(
    [...disks]
      .sort((a, b) => b.total_space - a.total_space)
      .slice(0, 3)
      .map(d => {
        const used = d.total_space - d.available_space;
        const pct = d.total_space > 0 ? (used / d.total_space) * 100 : 0;
        return {
          path: d.mount_point,
          text: `${(used / GB).toFixed(1)} / ${(d.total_space / GB).toFixed(1)} GB`,
          pct,
        };
      })
  );

  let more = $derived(Math.max(0, disks.length - 3));
</script>

<div class="panel flex min-w-0 flex-1 flex-col gap-[26px] p-[clamp(16px,2.4vw,28px)]">
  <!-- Load average -->
  <div class="flex flex-col gap-3.5">
    <span class="panel-title">Load average</span>
    <div class="flex gap-7">
      {#each loads as l (l.k)}
        <div class="flex flex-col gap-1">
          <span class="tnum text-[28px] font-light tracking-[-0.04em]">{l.v}</span>
          <span class="eyebrow">{l.k}</span>
        </div>
      {/each}
    </div>
    <span class="font-mono text-[12px]" style="color:var(--ink-3)">across {cores} cores</span>
  </div>

  <!-- Swap -->
  <div class="flex flex-col gap-2.5 border-t pt-[22px]" style="border-color:var(--line)">
    <div class="flex items-baseline justify-between">
      <span class="panel-title">Swap</span>
      <span class="font-mono tnum text-[13px]" style="color:var(--ink-2)">{fmtBytes(swapUsed)} / {fmtBytes(swapTotal)}</span>
    </div>
    <span class="block h-1 overflow-hidden rounded-sm" style="background:rgba(255,255,255,0.08)">
      <span class="block h-full rounded-sm" style="width:{Math.max(1.5, swapPct)}%;background:{zoneColor(swapPct)}"></span>
    </span>
  </div>

  <!-- Mounts -->
  <div class="flex flex-col gap-3.5 border-t pt-[22px]" style="border-color:var(--line)">
    <span class="panel-title">Mounts</span>
    {#each mounts as m (m.path)}
      <div class="flex flex-col gap-2">
        <div class="flex items-baseline justify-between gap-3">
          <span class="font-mono truncate text-[13px]" title={m.path}>{m.path}</span>
          <span class="font-mono tnum whitespace-nowrap text-[12px]" style="color:var(--ink-3)">{m.text}</span>
        </div>
        <span class="block h-1 overflow-hidden rounded-sm" style="background:rgba(255,255,255,0.08)">
          <span class="block h-full rounded-sm" style="width:{Math.max(1.5, m.pct)}%;background:{zoneColor(m.pct)}"></span>
        </span>
      </div>
    {/each}
    {#if more > 0}
      <span class="font-mono text-[12px]" style="color:var(--ink-3)">+{more} more mount{more === 1 ? '' : 's'}</span>
    {/if}
  </div>
</div>
