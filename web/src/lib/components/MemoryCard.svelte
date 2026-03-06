<script lang="ts">
  let { used, total, swap_used, swap_total }: { used: number, total: number, swap_used: number, swap_total: number } = $props();

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
</script>

<div class="glass-card p-6">
  <div class="flex justify-between items-center mb-4">
    <h3 class="text-slate-400 text-xs font-bold uppercase tracking-widest">Memory</h3>
    <span class="text-xs font-mono text-cyan-300 bg-cyan-900/30 px-2 py-1 rounded border border-cyan-800/50 shadow-[0_0_10px_rgba(6,182,212,0.1)]">
        Swap: {formatBytes(swap_used)} / {formatBytes(swap_total)}
    </span>
  </div>
  
  <div class="flex items-end justify-between gap-4">
    <div class="text-5xl font-bold text-white tracking-tighter tabular-nums drop-shadow-xl">
      {((used / total) * 100).toFixed(1)}<span class="text-xl text-slate-500 ml-1 font-light">%</span>
    </div>
    
    <div class="text-right">
        <div class="text-sm font-mono text-cyan-200">{formatBytes(used)}</div>
        <div class="text-xs text-slate-500 font-mono">of {formatBytes(total)}</div>
    </div>
  </div>
</div>
