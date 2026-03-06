<script lang="ts">
  let { usage, cores, history }: { usage: number, cores: number, history: number[] } = $props();

  function getSparklinePath(data: number[], width: number, height: number): string {
    if (data.length < 2) return '';
    const step = width / (data.length - 1);
    const max = 100;
    const points = data.map((val, i) => {
      const x = i * step;
      const y = height - (val / max) * height;
      return `${x},${y}`;
    }).join(' ');
    return `M ${points}`;
  }
</script>

<div class="glass-card p-6">
  <div class="flex justify-between items-center mb-4">
    <h3 class="text-slate-400 text-xs font-bold uppercase tracking-widest">CPU Usage</h3>
    <span class="text-xs font-mono text-cyan-300 bg-cyan-900/30 px-2 py-1 rounded border border-cyan-800/50 shadow-[0_0_10px_rgba(6,182,212,0.1)]">{cores} Cores</span>
  </div>
  
  <div class="flex items-end justify-between gap-4">
    <div class="text-5xl font-bold text-white tracking-tighter tabular-nums drop-shadow-xl">
      {usage.toFixed(1)}<span class="text-xl text-slate-500 ml-1 font-light">%</span>
    </div>
    
    <div class="h-12 w-32 filter drop-shadow-[0_0_5px_rgba(168,85,247,0.5)]">
        <svg width="100%" height="100%" viewBox="0 0 120 40" preserveAspectRatio="none" class="stroke-purple-400 fill-none stroke-2 overflow-visible">
            <path d={getSparklinePath(history, 120, 40)} vector-effect="non-scaling-stroke" />
        </svg>
    </div>
  </div>
</div>
