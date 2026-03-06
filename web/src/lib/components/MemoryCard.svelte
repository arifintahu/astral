<script lang="ts">
  let { used, total, swap_used, swap_total }: { used: number, total: number, swap_used: number, swap_total: number } = $props();

  let percent = $derived(total > 0 ? (used / total) * 100 : 0);
  let statusColor = $derived(
    percent >= 90 ? 'text-rose-400' : percent >= 75 ? 'text-amber-400' : 'text-emerald-400'
  );
  let barColor = $derived(
    percent >= 90 ? 'bg-rose-500/80' : percent >= 75 ? 'bg-amber-500/80' : 'bg-cyan-500/60'
  );

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }
</script>

<div class="glass-card p-6 h-full flex flex-col justify-between">
  <div class="flex justify-between items-center mb-5">
    <h3 class="metric-label">Memory</h3>
    <span class="metric-badge badge-cyan">
      Swap {formatBytes(swap_used)}
    </span>
  </div>

  <div class="flex items-end justify-between gap-3 mb-4">
    <div>
      <span class="metric-value">{percent.toFixed(0)}</span>
      <span class="metric-unit">%</span>
    </div>
    <div class="text-right pb-1">
      <div class="text-sm font-mono text-slate-300 font-medium">{formatBytes(used)}</div>
      <div class="text-[11px] text-slate-600 font-mono">of {formatBytes(total)}</div>
    </div>
  </div>

  <!-- Progress bar -->
  <div class="progress-track h-2">
    <div
      class="progress-fill {barColor}"
      style="width: {percent}%"
    ></div>
  </div>
</div>
