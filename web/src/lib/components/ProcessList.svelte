<script lang="ts">
  import type { ProcessInfo } from '../types';

  let { processes, totalMemory }: { processes: ProcessInfo[], totalMemory: number } = $props();

  let sortBy = $state<'cpu' | 'mem'>('cpu');

  let hasProcesses = $derived(processes && processes.length > 0);

  let sorted = $derived(
    [...processes].sort((a, b) =>
      sortBy === 'cpu' ? b.cpu_usage - a.cpu_usage : b.memory - a.memory
    ).slice(0, 10)
  );

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function cpuColor(pct: number): string {
    if (pct > 80) return 'text-rose-400';
    if (pct > 50) return 'text-amber-400';
    return 'text-slate-400';
  }
</script>

<div class="glass-panel p-6 h-full flex flex-col">
  <div class="flex justify-between items-center mb-4 flex-shrink-0">
    <h3 class="text-[11px] font-bold text-slate-300 uppercase tracking-[0.15em]">Top Processes</h3>
    {#if hasProcesses}
      <div class="flex bg-white/[0.03] rounded-xl p-1 border border-white/[0.05]">
        <button
          class="px-3 py-1 text-[11px] font-semibold rounded-lg transition-all duration-200 cursor-pointer
                 {sortBy === 'cpu' ? 'bg-white/[0.08] text-white shadow-sm' : 'text-slate-500 hover:text-slate-300'}"
          onclick={() => sortBy = 'cpu'}
        >CPU</button>
        <button
          class="px-3 py-1 text-[11px] font-semibold rounded-lg transition-all duration-200 cursor-pointer
                 {sortBy === 'mem' ? 'bg-white/[0.08] text-white shadow-sm' : 'text-slate-500 hover:text-slate-300'}"
          onclick={() => sortBy = 'mem'}
        >Memory</button>
      </div>
    {/if}
  </div>

  <div class="flex-1 min-h-0 flex flex-col">
    {#if hasProcesses}
      <!-- Header -->
      <div class="grid grid-cols-[1fr_80px_80px] gap-2 px-3 py-2 text-[10px] text-slate-600 uppercase tracking-wider font-semibold border-b border-white/[0.04] flex-shrink-0">
        <span>Process</span>
        <span class="text-right">CPU</span>
        <span class="text-right">Memory</span>
      </div>

      <!-- Rows -->
      <div class="overflow-y-auto custom-scrollbar flex-1 min-h-0">
        {#each sorted as proc, i}
          <div class="grid grid-cols-[1fr_80px_80px] gap-2 px-3 py-2 items-center hover:bg-white/[0.02] transition-colors rounded-lg">
            <div class="flex items-center gap-2 min-w-0">
              <span class="text-[10px] text-slate-700 font-mono tabular-nums w-5 flex-shrink-0">{i + 1}</span>
              <span class="text-[13px] text-slate-300 truncate font-medium" title="{proc.name} (PID: {proc.pid})">{proc.name}</span>
            </div>
            <div class="text-right">
              <span class="text-[13px] font-mono tabular-nums font-medium {cpuColor(proc.cpu_usage)}">{proc.cpu_usage.toFixed(1)}%</span>
            </div>
            <div class="text-right">
              <span class="text-[13px] font-mono tabular-nums text-slate-400">{formatBytes(proc.memory)}</span>
            </div>
          </div>
        {/each}
      </div>

    {:else}
      <!-- T-01: empty state when process collection is disabled -->
      <div class="flex-1 flex flex-col items-center justify-center text-center px-4 gap-3">
        <div class="w-12 h-12 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center">
          <svg class="w-6 h-6 text-slate-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 3H5a2 2 0 00-2 2v4m6-6h10a2 2 0 012 2v4M9 3v18m0 0h10a2 2 0 002-2V9M9 21H5a2 2 0 01-2-2V9m0 0h18" />
          </svg>
        </div>
        <div>
          <p class="text-[13px] text-slate-400 font-medium mb-1">Process monitoring is off</p>
          <p class="text-[11px] text-slate-600 leading-relaxed">Enable it in <span class="text-slate-400">Settings → Processes</span> or start with <code class="text-cyan-600 bg-white/[0.04] px-1 rounded">--enable-process-list</code></p>
        </div>
      </div>
    {/if}
  </div>
</div>
