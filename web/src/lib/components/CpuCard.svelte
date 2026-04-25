<script lang="ts">
  let { usage, cores, history }: { usage: number, cores: number, history: number[] } = $props();

  let statusColor = $derived(
    usage >= 90 ? 'text-rose-400' : usage >= 70 ? 'text-amber-400' : 'text-emerald-400'
  );
  let fillColor = $derived(
    usage >= 90 ? 'stroke-rose-400' : usage >= 70 ? 'stroke-amber-400' : 'stroke-cyan-400'
  );
  let sparkFill = $derived(
    usage >= 90 ? 'rgba(244,63,94,0.08)' : usage >= 70 ? 'rgba(251,191,36,0.08)' : 'rgba(6,182,212,0.08)'
  );
  let sparkStroke = $derived(
    usage >= 90 ? 'rgba(244,63,94,0.6)' : usage >= 70 ? 'rgba(251,191,36,0.6)' : 'rgba(6,182,212,0.5)'
  );

  function getSparklinePath(data: number[], width: number, height: number): string {
    if (data.length < 2) return '';
    const step = width / (data.length - 1);
    const max = 100;
    const points = data.map((val, i) => {
      const x = i * step;
      const y = height - (val / max) * height;
      return `${x},${y}`;
    });
    return `M ${points.join(' ')}`;
  }

  function getSparklineArea(data: number[], width: number, height: number): string {
    if (data.length < 2) return '';
    const step = width / (data.length - 1);
    const max = 100;
    const points = data.map((val, i) => {
      const x = i * step;
      const y = height - (val / max) * height;
      return `${x},${y}`;
    });
    return `M 0,${height} L ${points.join(' ')} L ${width},${height} Z`;
  }

  const radius = 40;
  const circumference = 2 * Math.PI * radius;
  let dashOffset = $derived(circumference - (usage / 100) * circumference);
</script>

<!-- T-14: border for card separation; T-05: brighter label with cyan top accent -->
<div class="glass-card p-6 h-full flex flex-col justify-between border border-white/[0.08] border-t-cyan-500/30">
  <div class="flex justify-between items-center mb-5">
    <h3 class="text-[11px] font-bold text-slate-300 uppercase tracking-[0.15em]">Processor</h3>
    <span class="metric-badge badge-cyan">{cores} cores</span>
  </div>

  <div class="flex items-center gap-5">
    <!-- T-04: circular gauge with visible background track -->
    <div class="relative flex-shrink-0">
      <svg width="96" height="96" viewBox="0 0 96 96" class="transform -rotate-90">
        <!-- Background track — always visible at 15% white so the gauge reads at 0% -->
        <circle cx="48" cy="48" r={radius} fill="none" stroke="rgba(255,255,255,0.12)" stroke-width="6" />
        <!-- Filled arc -->
        <circle
          cx="48" cy="48" r={radius}
          fill="none"
          class={fillColor}
          stroke-width="6"
          stroke-linecap="round"
          stroke-dasharray={circumference}
          stroke-dashoffset={dashOffset}
          style="transition: stroke-dashoffset 0.7s ease-out, stroke 0.5s ease;"
        />
      </svg>
      <div class="absolute inset-0 flex flex-col items-center justify-center">
        <span class="text-2xl font-bold text-white tabular-nums tracking-tight">{usage.toFixed(0)}</span>
        <span class="text-[10px] text-slate-500 font-medium -mt-0.5">%</span>
      </div>
    </div>

    <!-- Sparkline -->
    <div class="flex-1 h-16 min-w-0">
      <svg width="100%" height="100%" viewBox="0 0 140 48" preserveAspectRatio="none" class="overflow-visible">
        <path d={getSparklineArea(history, 140, 48)} fill={sparkFill} />
        <path d={getSparklinePath(history, 140, 48)} fill="none" stroke={sparkStroke} stroke-width="1.5" vector-effect="non-scaling-stroke" />
      </svg>
    </div>
  </div>
</div>
