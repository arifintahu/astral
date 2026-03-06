<script lang="ts">
  import type { ProcessInfo } from '../types';

  let { processes, totalMemory }: { processes: ProcessInfo[], totalMemory: number } = $props();

  let sortBy = $state<'cpu' | 'mem'>('cpu');

  let sorted = $derived(
    [...processes].sort((a, b) =>
      sortBy === 'cpu'
        ? b.cpu_usage - a.cpu_usage
        : b.memory - a.memory
    ).slice(0, 10)
  );

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }
</script>

<div class="glass-panel p-6 h-full flex flex-col">
  <div class="flex justify-between items-center mb-4 flex-shrink-0">
    <h3 class="metric-label">Top Processes</h3>
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
  </div>

  <div class="flex-1 min-h-0 flex flex-col">
    <!-- Header -->
    <div class="grid grid-cols-[1fr_80px_80px] gap-2 px-3 py-2 text-[10px] text-slate-600 uppercase tracking-wider font-semibold border-b border-white/[0.04] flex-shrink-0">
      <span>Process</span>
      <span class="text-right">CPU</span>
      <span class="text-right">Memory</span>
    </div>

    <!-- Rows -->
    <div class="overflow-y-auto custom-scrollbar flex-1 min-h-0">
      {#each sorted as proc, i}
        <div class="grid grid-cols-[1fr_80px_80px] gap-2 px-3 py-2 items-center hover:bg-white/[0.02] transition-colors rounded-lg group {i % 2 === 0 ? '' : ''}">
          <div class="flex items-center gap-2 min-w-0">
            <span class="text-[10px] text-slate-700 font-mono tabular-nums w-5 flex-shrink-0">{i + 1}</span>
            <span class="text-[13px] text-slate-300 truncate font-medium" title="{proc.name} (PID: {proc.pid})">{proc.name}</span>
          </div>
          <div class="text-right">
            <span class="text-[13px] font-mono tabular-nums font-medium {proc.cpu_usage > 50 ? 'text-amber-400' : proc.cpu_usage > 80 ? 'text-rose-400' : 'text-slate-400'}">{proc.cpu_usage.toFixed(1)}%</span>
          </div>
          <div class="text-right">
            <span class="text-[13px] font-mono tabular-nums text-slate-400">{formatBytes(proc.memory)}</span>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>
