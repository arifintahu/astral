<script lang="ts">
  let idSeq = 0;

  let { data, max, color, w = 140, h = 48 }: {
    data: number[];
    max: number;
    color: string;
    w?: number;
    h?: number;
  } = $props();

  const uid = `mc${idSeq++}`;

  let paths = $derived.by(() => {
    if (data.length < 2) {
      return { line: '', area: '', dot: null as { x: number; y: number } | null };
    }
    const cap = max > 0 ? max : 1;
    const step = w / (data.length - 1);
    const pts = data.map((v, i) => ({
      x: i * step,
      y: 2 + (1 - Math.min(v, cap) / cap) * (h - 4),
    }));
    const coords = pts.map(p => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' L ');
    return {
      line: `M ${coords}`,
      area: `M ${coords} L ${w},${h} L 0,${h} Z`,
      dot: pts[pts.length - 1],
    };
  });
</script>

<svg width={w} height={h} viewBox="0 0 {w} {h}" preserveAspectRatio="none" style="overflow:visible;display:block">
  <defs>
    <linearGradient id={uid} x1="0" x2="0" y1="0" y2="1">
      <stop offset="0%"   stop-color={color} stop-opacity="0.28" />
      <stop offset="100%" stop-color={color} stop-opacity="0.02" />
    </linearGradient>
  </defs>
  {#if paths.area}
    <path d={paths.area} fill="url(#{uid})" />
  {/if}
  {#if paths.line}
    <path d={paths.line} fill="none" stroke={color} stroke-width="1.5"
          vector-effect="non-scaling-stroke" />
  {/if}
  {#if paths.dot}
    <circle cx={paths.dot.x} cy={paths.dot.y} r="2.5" fill={color}
            style="animation: pulseDot 2s ease-in-out infinite" />
  {/if}
</svg>
