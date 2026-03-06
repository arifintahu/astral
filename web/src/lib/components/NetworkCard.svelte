<script lang="ts">
  let { tx, rx }: { tx: number, rx: number } = $props();

  function formatSpeed(bytes: number): string {
    if (bytes === 0) return '0 B/s';
    const k = 1024;
    const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s', 'TB/s'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
</script>

<div class="bg-zinc-800 rounded-lg p-4 shadow-md border border-zinc-700">
  <div class="flex justify-between items-center mb-4">
    <h3 class="text-zinc-400 text-sm font-semibold uppercase tracking-wider">Network</h3>
  </div>
  
  <div class="flex flex-col gap-2">
    <div class="flex justify-between items-center border-b border-zinc-700 pb-2">
        <span class="text-xs text-zinc-500 uppercase font-bold flex items-center gap-1">
            <svg class="w-3 h-3 text-emerald-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"></path></svg>
            Tx
        </span>
        <span class="text-lg font-mono text-emerald-400 font-bold tabular-nums">{formatSpeed(tx)}</span>
    </div>
    <div class="flex justify-between items-center pt-2">
        <span class="text-xs text-zinc-500 uppercase font-bold flex items-center gap-1">
            <svg class="w-3 h-3 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3"></path></svg>
            Rx
        </span>
        <span class="text-lg font-mono text-blue-400 font-bold tabular-nums">{formatSpeed(rx)}</span>
    </div>
  </div>
</div>
