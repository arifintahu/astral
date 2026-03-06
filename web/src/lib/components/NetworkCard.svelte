<script lang="ts">
  let { tx, rx }: { tx: number, rx: number } = $props();

  function formatSpeed(bytes: number): { value: string, unit: string } {
    if (bytes === 0) return { value: '0', unit: 'B/s' };
    const k = 1024;
    const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s', 'TB/s'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return {
      value: parseFloat((bytes / Math.pow(k, i)).toFixed(1)).toString(),
      unit: sizes[i]
    };
  }

  let txFormatted = $derived(formatSpeed(tx));
  let rxFormatted = $derived(formatSpeed(rx));
</script>

<div class="glass-card p-6">
  <div class="flex justify-between items-center mb-5">
    <h3 class="metric-label">Network</h3>
  </div>

  <div class="flex flex-col gap-3.5">
    <!-- Upload -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2.5">
        <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-purple-500/[0.08] border border-purple-500/[0.12]">
          <svg class="w-3.5 h-3.5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"></path>
          </svg>
        </div>
        <span class="text-[11px] text-slate-500 font-semibold uppercase tracking-wider">Upload</span>
      </div>
      <div class="text-right">
        <span class="text-xl font-mono text-purple-300 font-semibold tabular-nums">{txFormatted.value}</span>
        <span class="text-[11px] text-purple-400/50 font-mono ml-1">{txFormatted.unit}</span>
      </div>
    </div>

    <div class="h-px bg-white/[0.04]"></div>

    <!-- Download -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2.5">
        <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-cyan-500/[0.08] border border-cyan-500/[0.12]">
          <svg class="w-3.5 h-3.5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3"></path>
          </svg>
        </div>
        <span class="text-[11px] text-slate-500 font-semibold uppercase tracking-wider">Download</span>
      </div>
      <div class="text-right">
        <span class="text-xl font-mono text-cyan-300 font-semibold tabular-nums">{rxFormatted.value}</span>
        <span class="text-[11px] text-cyan-400/50 font-mono ml-1">{rxFormatted.unit}</span>
      </div>
    </div>
  </div>
</div>
