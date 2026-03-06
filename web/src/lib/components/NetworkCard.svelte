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

<div class="glass-card p-6">
  <div class="flex justify-between items-center mb-4">
    <h3 class="text-slate-400 text-xs font-bold uppercase tracking-widest">Network</h3>
  </div>
  
  <div class="flex flex-col gap-4">
    <div class="flex justify-between items-center border-b border-white/10 pb-3">
        <span class="text-xs text-slate-500 uppercase font-bold flex items-center gap-2">
            <div class="p-1.5 bg-purple-500/20 rounded-md">
                <svg class="w-4 h-4 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"></path></svg>
            </div>
            Tx
        </span>
        <span class="text-xl font-mono text-purple-300 font-bold tabular-nums drop-shadow">{formatSpeed(tx)}</span>
    </div>
    <div class="flex justify-between items-center pt-1">
        <span class="text-xs text-slate-500 uppercase font-bold flex items-center gap-2">
            <div class="p-1.5 bg-cyan-500/20 rounded-md">
                <svg class="w-4 h-4 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3"></path></svg>
            </div>
            Rx
        </span>
        <span class="text-xl font-mono text-cyan-300 font-bold tabular-nums drop-shadow">{formatSpeed(rx)}</span>
    </div>
  </div>
</div>
