<script lang="ts">
  import type { SystemMetrics } from '../types';
  import { deriveStatus, type Thresholds } from '../status';
  import { fmtUptime } from '../format';

  let { metrics, thresholds }: { metrics: SystemMetrics; thresholds: Thresholds } = $props();

  let status = $derived(deriveStatus(metrics, thresholds));
  let facts = $derived([
    { k: 'Host',   v: metrics.hostname },
    { k: 'OS',     v: `${metrics.os_name} ${metrics.os_version}`.trim() },
    { k: 'Cores',  v: String(metrics.cpu_cores) },
    { k: 'Uptime', v: fmtUptime(metrics.uptime) },
  ]);
</script>

<section class="anim-fade-up flex flex-col gap-3.5 pb-1 pt-[clamp(8px,3vw,28px)]">
  <div class="flex items-center gap-3.5">
    <span class="relative inline-block size-3 flex-none" aria-hidden="true">
      <span class="ping absolute inset-0 rounded-full" style="background:{status.color}"></span>
      <span class="absolute inset-0 rounded-full" style="background:{status.color}"></span>
    </span>
    <h1 class="m-0 text-balance text-[clamp(30px,5vw,54px)] font-medium leading-[1.05] tracking-[-0.04em]">
      {status.headline}
    </h1>
  </div>

  <p class="m-0 max-w-[60ch] text-pretty text-[clamp(15px,1.6vw,17px)] leading-normal" style="color:var(--ink-2)">
    {status.sub}
  </p>

  <div class="mt-1.5 flex flex-wrap gap-x-7 gap-y-2">
    {#each facts as f (f.k)}
      <div class="flex items-baseline gap-2">
        <span class="eyebrow">{f.k}</span>
        <span class="font-mono text-[13px]">{f.v}</span>
      </div>
    {/each}
  </div>
</section>
