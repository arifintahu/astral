<script lang="ts">
  import type { ProcessInfo } from '../types';

  let { processes, totalMemory }: { processes: ProcessInfo[]; totalMemory: number } = $props();

  let sortBy = $state<'cpu' | 'mem'>('cpu');

  let sorted = $derived(
    [...processes]
      .sort((a, b) => sortBy === 'cpu' ? b.cpu_usage - a.cpu_usage : b.memory - a.memory)
      .slice(0, 8)
  );

  let maxCpu = $derived(Math.max(...sorted.map(p => p.cpu_usage), 0.001));
  let maxMem = $derived(Math.max(...sorted.map(p => p.memory), 1));

  function fmtMem(b: number): string {
    if (b >= 1073741824) return (b / 1073741824).toFixed(1) + 'G';
    if (b >= 1048576)    return (b / 1048576).toFixed(0) + 'M';
    return (b / 1024).toFixed(0) + 'K';
  }

  function cpuColor(v: number): string {
    if (v >= 50) return 'var(--crit)';
    if (v >= 20) return 'var(--warm)';
    return 'var(--ink-2)';
  }
</script>

<div class="surface h-full flex flex-col" style="padding:18px 20px">
  <!-- Header -->
  <div class="flex items-center justify-between mb-4 flex-shrink-0">
    <span class="eyebrow">Top Processes</span>
    {#if processes.length > 0}
      <div class="seg-control">
        <button class="seg-btn {sortBy === 'cpu' ? 'active' : ''}" onclick={() => sortBy = 'cpu'}>CPU</button>
        <button class="seg-btn {sortBy === 'mem' ? 'active' : ''}" onclick={() => sortBy = 'mem'}>Mem</button>
      </div>
    {/if}
  </div>

  {#if processes.length === 0}
    <div class="flex-1 flex flex-col items-center justify-center gap-3 text-center" style="padding:0 12px">
      <div style="width:40px;height:40px;border-radius:10px;background:var(--bg-2);border:1px solid var(--line);display:flex;align-items:center;justify-content:center">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--ink-4)" stroke-width="1.5">
          <rect x="2" y="3" width="20" height="14" rx="2"/>
          <line x1="8" y1="21" x2="16" y2="21"/>
          <line x1="12" y1="17" x2="12" y2="21"/>
        </svg>
      </div>
      <div>
        <div style="font-size:12px;color:var(--ink-3);font-weight:500;margin-bottom:4px">Process monitoring is off</div>
        <div style="font-size:11px;color:var(--ink-4)">Enable in Settings → Processes</div>
      </div>
    </div>
  {:else}
    <!-- Column headers -->
    <div style="display:grid;grid-template-columns:1fr 60px 80px 70px;gap:8px;padding:2px 6px 6px;flex-shrink:0">
      <span class="eyebrow" style="font-size:9px">Process</span>
      <span class="eyebrow" style="font-size:9px;text-align:right">PID</span>
      <span class="eyebrow" style="font-size:9px;text-align:right">CPU</span>
      <span class="eyebrow" style="font-size:9px;text-align:right">Memory</span>
    </div>

    <!-- Rows -->
    <div class="flex-1 min-h-0 overflow-y-auto nice-scroll">
      {#each sorted as proc, i}
        <div style="display:grid;grid-template-columns:1fr 60px 80px 70px;gap:8px;padding:5px 6px;border-radius:6px;align-items:center;cursor:default"
             class="card-hover">
          <!-- Name + rank -->
          <div class="flex items-center gap-2 min-w-0">
            <span class="tnum font-mono flex-shrink-0" style="font-size:10px;color:var(--ink-4);width:14px">{i+1}</span>
            <span style="font-size:12px;color:var(--ink-2);overflow:hidden;text-overflow:ellipsis;white-space:nowrap" title="{proc.name}">{proc.name}</span>
          </div>
          <!-- PID -->
          <div class="tnum font-mono text-right" style="font-size:11px;color:var(--ink-4)">{proc.pid}</div>
          <!-- CPU -->
          <div class="flex flex-col items-end gap-0.5">
            <span class="tnum font-mono" style="font-size:11px;color:{cpuColor(proc.cpu_usage)}">{proc.cpu_usage.toFixed(1)}%</span>
            <div style="width:100%;height:2px;background:var(--bg-2);border-radius:1px;overflow:hidden">
              <div style="width:{(proc.cpu_usage / maxCpu) * 100}%;height:100%;background:var(--accent);border-radius:1px;transition:width 0.4s"></div>
            </div>
          </div>
          <!-- Memory -->
          <div class="flex flex-col items-end gap-0.5">
            <span class="tnum font-mono" style="font-size:11px;color:var(--ink-2)">{fmtMem(proc.memory)}</span>
            <div style="width:100%;height:2px;background:var(--bg-2);border-radius:1px;overflow:hidden">
              <div style="width:{(proc.memory / maxMem) * 100}%;height:100%;background:var(--warm);border-radius:1px;transition:width 0.4s"></div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
