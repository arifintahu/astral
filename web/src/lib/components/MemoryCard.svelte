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

<div class="bg-zinc-800 rounded-lg p-4 shadow-md border border-zinc-700">
  <div class="flex justify-between items-center mb-4">
    <h3 class="text-zinc-400 text-sm font-semibold uppercase tracking-wider">Memory</h3>
    <span class="text-xs font-mono text-zinc-500 bg-zinc-900 px-2 py-1 rounded">
        Swap: {formatBytes(swap_used)} / {formatBytes(swap_total)}
    </span>
  </div>
  
  <div class="flex items-end justify-between gap-4">
    <div class="text-4xl font-bold text-white tracking-tighter tabular-nums">
      {((used / total) * 100).toFixed(1)}<span class="text-lg text-zinc-500 ml-1">%</span>
    </div>
    
    <div class="text-right">
        <div class="text-sm font-mono text-zinc-300">{formatBytes(used)}</div>
        <div class="text-xs text-zinc-500 font-mono">of {formatBytes(total)}</div>
    </div>
  </div>
</div>
