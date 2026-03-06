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

<div class="bg-zinc-800 rounded-lg p-4 shadow-md border border-zinc-700">
  <div class="flex justify-between items-center mb-4">
    <h3 class="text-zinc-400 text-sm font-semibold uppercase tracking-wider">CPU Usage</h3>
    <span class="text-xs font-mono text-zinc-500 bg-zinc-900 px-2 py-1 rounded">{cores} Cores</span>
  </div>
  
  <div class="flex items-end justify-between gap-4">
    <div class="text-4xl font-bold text-white tracking-tighter tabular-nums">
      {usage.toFixed(1)}<span class="text-lg text-zinc-500 ml-1">%</span>
    </div>
    
    <div class="h-10 w-32">
        <svg width="100%" height="100%" viewBox="0 0 120 40" preserveAspectRatio="none" class="stroke-emerald-500 fill-none stroke-2 overflow-visible">
            <path d={getSparklinePath(history, 120, 40)} vector-effect="non-scaling-stroke" />
        </svg>
    </div>
  </div>
</div>
