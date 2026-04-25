<script lang="ts">
  let { tx, rx, txHistory, rxHistory }: { tx: number, rx: number, txHistory: number[], rxHistory: number[] } = $props();

  // T-11: normalize TX and RX to the same unit by picking the larger value's scale.
  function formatSpeedRaw(bytes: number): { value: number, unitIndex: number } {
    if (bytes <= 0) return { value: 0, unitIndex: 0 };
    const k = 1024;
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return { value: bytes / Math.pow(k, i), unitIndex: i };
  }

  const units = ['B/s', 'KB/s', 'MB/s', 'GB/s', 'TB/s'];

  let txRaw = $derived(formatSpeedRaw(tx));
  let rxRaw = $derived(formatSpeedRaw(rx));

  // Use the higher unit scale for both so they're visually comparable.
  let sharedUnitIndex = $derived(Math.max(txRaw.unitIndex, rxRaw.unitIndex));
  let divisor = $derived(Math.pow(1024, sharedUnitIndex));
  let unitLabel = $derived(units[sharedUnitIndex] ?? 'B/s');

  let txValue = $derived(tx > 0 ? (tx / divisor).toFixed(1) : '0');
  let rxValue = $derived(rx > 0 ? (rx / divisor).toFixed(1) : '0');

  function getSparklinePath(data: number[], width: number, height: number): string {
    if (data.length < 2) return '';
    const max = Math.max(...data, 1);
    const step = width / (data.length - 1);
    const points = data.map((val, i) => {
      const x = i * step;
      const y = height - (val / max) * height * 0.9;
      return `${x},${y}`;
    });
    return `M ${points.join(' ')}`;
  }

  function getSparklineArea(data: number[], width: number, height: number): string {
    if (data.length < 2) return '';
    const max = Math.max(...data, 1);
    const step = width / (data.length - 1);
    const points = data.map((val, i) => {
      const x = i * step;
      const y = height - (val / max) * height * 0.9;
      return `${x},${y}`;
    });
    return `M 0,${height} L ${points.join(' ')} L ${width},${height} Z`;
  }
</script>

<!-- T-14: border; T-05: brighter label with amber top accent -->
<div class="glass-card p-6 h-full flex flex-col justify-between border border-white/[0.08] border-t-amber-500/30">
  <div class="flex justify-between items-center mb-5">
    <h3 class="text-[11px] font-bold text-slate-300 uppercase tracking-[0.15em]">Network</h3>
    <span class="text-[10px] text-slate-500 font-mono">{unitLabel}</span>
  </div>

  <div class="flex flex-col gap-3">
    <!-- Upload (TX) -->
    <div class="flex items-center gap-3">
      <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-purple-500/[0.08] border border-purple-500/[0.12] flex-shrink-0">
        <svg class="w-3.5 h-3.5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"></path>
        </svg>
      </div>
      <div class="flex-1 h-7 min-w-0">
        <svg width="100%" height="100%" viewBox="0 0 100 28" preserveAspectRatio="none" class="overflow-visible">
          <path d={getSparklineArea(txHistory, 100, 28)} fill="rgba(168,85,247,0.06)" />
          <path d={getSparklinePath(txHistory, 100, 28)} fill="none" stroke="rgba(168,85,247,0.5)" stroke-width="1.5" vector-effect="non-scaling-stroke" />
        </svg>
      </div>
      <div class="text-right flex-shrink-0 min-w-[60px]">
        <span class="text-lg font-mono text-purple-300 font-semibold tabular-nums">{txValue}</span>
      </div>
    </div>

    <div class="h-px bg-white/[0.04]"></div>

    <!-- Download (RX) -->
    <div class="flex items-center gap-3">
      <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-cyan-500/[0.08] border border-cyan-500/[0.12] flex-shrink-0">
        <svg class="w-3.5 h-3.5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3"></path>
        </svg>
      </div>
      <div class="flex-1 h-7 min-w-0">
        <svg width="100%" height="100%" viewBox="0 0 100 28" preserveAspectRatio="none" class="overflow-visible">
          <path d={getSparklineArea(rxHistory, 100, 28)} fill="rgba(6,182,212,0.06)" />
          <path d={getSparklinePath(rxHistory, 100, 28)} fill="none" stroke="rgba(6,182,212,0.5)" stroke-width="1.5" vector-effect="non-scaling-stroke" />
        </svg>
      </div>
      <div class="text-right flex-shrink-0 min-w-[60px]">
        <span class="text-lg font-mono text-cyan-300 font-semibold tabular-nums">{rxValue}</span>
      </div>
    </div>
  </div>
</div>
