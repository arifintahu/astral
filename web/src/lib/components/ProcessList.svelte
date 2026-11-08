<script lang="ts">
  import type { ProcessInfo } from '../types';
  import { fmtBytes } from '../format';

  let { processes }: { processes: ProcessInfo[] } = $props();

  let sortBy = $state<'cpu' | 'mem'>('cpu');

  let sorted = $derived(
    [...processes]
      .sort((a, b) => (sortBy === 'cpu' ? b.cpu_usage - a.cpu_usage : b.memory - a.memory))
      .slice(0, 6)
  );

  let maxCpu = $derived(Math.max(...sorted.map(p => p.cpu_usage), 0.001));
  let maxMem = $derived(Math.max(...sorted.map(p => p.memory), 1));

  function cpuColor(v: number): string {
    if (v >= 50) return 'var(--crit)';
    if (v >= 20) return 'var(--warm)';
    return 'var(--accent)';
  }
  function cpuText(v: number): string {
    return v >= 50 ? 'var(--crit)' : v >= 20 ? 'var(--warm)' : 'var(--ink-2)';
  }

  const COLS = 'grid-template-columns:minmax(0,1fr) 88px 88px';
</script>

<div class="panel flex min-w-0 flex-1 flex-col gap-1.5 p-[clamp(16px,2.4vw,28px)]">
  <div class="mb-2.5 flex items-center justify-between gap-3">
    <span class="panel-title">Top processes</span>
    {#if processes.length > 0}
      <div class="seg2" role="group" aria-label="Sort processes by">
        <button type="button" aria-pressed={sortBy === 'cpu'} onclick={() => (sortBy = 'cpu')}><span>CPU</span></button>
        <button type="button" aria-pressed={sortBy === 'mem'} onclick={() => (sortBy = 'mem')}><span>Memory</span></button>
      </div>
    {/if}
  </div>

  {#if processes.length === 0}
    <div class="flex flex-1 flex-col items-center justify-center gap-1 px-3 py-10 text-center">
      <span class="text-[13px] font-medium" style="color:var(--ink-2)">Process monitoring is off</span>
      <span class="text-[12px]" style="color:var(--ink-3)">Enable it in Settings → Processes</span>
    </div>
  {:else}
    <div class="grid gap-4 pb-2.5" style={COLS}>
      <span class="eyebrow">Process</span>
      <span class="eyebrow text-right">CPU</span>
      <span class="eyebrow text-right">Memory</span>
    </div>

    {#each sorted as p (p.pid)}
      <div class="row-hover -mx-2 grid items-center gap-4 rounded-lg border-t px-2 py-3" style="{COLS};border-color:var(--line)">
        <div class="flex min-w-0 items-baseline gap-2.5">
          <span class="truncate text-[14px]" title={p.name}>{p.name}</span>
          <span class="font-mono text-[12px]" style="color:var(--ink-3)">{p.pid}</span>
        </div>

        <div class="flex flex-col items-end gap-1.5">
          <span class="font-mono tnum text-[13px]" style="color:{cpuText(p.cpu_usage)}">{p.cpu_usage.toFixed(1)}%</span>
          <span class="block h-[3px] w-full overflow-hidden rounded-sm" style="background:rgba(255,255,255,0.08)">
            <span class="block h-full rounded-sm transition-[width] duration-500"
                  style="width:{Math.max(2, (p.cpu_usage / maxCpu) * 100)}%;background:{cpuColor(p.cpu_usage)}"></span>
          </span>
        </div>

        <div class="flex flex-col items-end gap-1.5">
          <span class="font-mono tnum text-[13px]" style="color:var(--ink-2)">{fmtBytes(p.memory)}</span>
          <span class="block h-[3px] w-full overflow-hidden rounded-sm" style="background:rgba(255,255,255,0.08)">
            <span class="block h-full rounded-sm transition-[width] duration-500"
                  style="width:{Math.max(2, (p.memory / maxMem) * 100)}%;background:var(--violet)"></span>
          </span>
        </div>
      </div>
    {/each}
  {/if}
</div>
