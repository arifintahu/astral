<script lang="ts">
  import type { DiskInfo } from '../types';

  let { disks }: { disks: DiskInfo[] } = $props();

  let rootDisk = $derived(disks.find(d => d.mount_point === '/') || disks[0]);
  let totalSpace = $derived(rootDisk ? rootDisk.total_space : 0);
  let totalUsed = $derived(rootDisk ? (rootDisk.total_space - rootDisk.available_space) : 0);
  let utilization = $derived(totalSpace > 0 ? (totalUsed / totalSpace) * 100 : 0);

  // T-12: aggregate I/O rates — already bytes/s from sysinfo delta.
  let totalRead = $derived(disks.reduce((sum, d) => sum + d.read_bytes, 0));
  let totalWritten = $derived(disks.reduce((sum, d) => sum + d.written_bytes, 0));

  // T-10: per-disk breakdown visibility
  let showBreakdown = $state(false);

  let barColor = $derived(
    utilization >= 90
      ? 'bg-rose-500/80'
      : utilization >= 75
      ? 'bg-amber-500/80'
      : 'bg-gradient-to-r from-purple-500/70 to-cyan-500/70'
  );

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  // T-12: show B/s rate (already a delta value from sysinfo).
  function formatRate(bytes: number): string {
    if (bytes === 0) return '0 B/s';
    const k = 1024;
    const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function diskPercent(d: DiskInfo): number {
    if (d.total_space === 0) return 0;
    return ((d.total_space - d.available_space) / d.total_space) * 100;
  }
</script>

<!-- T-14: border; T-05: brighter label with emerald top accent -->
<div class="glass-card p-6 h-full flex flex-col justify-between border border-white/[0.08] border-t-emerald-500/30">
  <div class="flex justify-between items-center mb-5">
    <h3 class="text-[11px] font-bold text-slate-300 uppercase tracking-[0.15em]">Storage</h3>
    <!-- T-10: toggle breakdown when >1 disk -->
    {#if disks.length > 1}
      <button
        onclick={() => showBreakdown = !showBreakdown}
        class="metric-badge badge-cyan cursor-pointer hover:bg-cyan-500/20 transition-colors"
        title={showBreakdown ? 'Hide per-disk breakdown' : 'Show per-disk breakdown'}
      >
        {disks.length} mounts {showBreakdown ? '▲' : '▼'}
      </button>
    {:else}
      <span class="metric-badge badge-cyan">{disks.length} mount</span>
    {/if}
  </div>

  <!-- Primary disk -->
  <div class="mb-3">
    <div class="flex justify-between items-end mb-3">
      <div>
        <span class="text-3xl font-extrabold text-white tracking-tight tabular-nums">{utilization.toFixed(0)}</span>
        <span class="text-sm text-slate-500 ml-0.5 font-normal">%</span>
      </div>
      <div class="text-right">
        <div class="text-sm font-mono text-slate-300 font-medium">{formatBytes(totalUsed)}</div>
        <div class="text-[11px] text-slate-600 font-mono">of {formatBytes(totalSpace)}</div>
      </div>
    </div>
    <div class="progress-track h-2.5">
      <div class="progress-fill {barColor}" style="width: {utilization}%"></div>
    </div>
  </div>

  <!-- T-10: per-disk breakdown -->
  {#if showBreakdown && disks.length > 1}
    <div class="mb-3 space-y-2 max-h-28 overflow-y-auto custom-scrollbar pr-1">
      {#each disks as disk}
        {@const pct = diskPercent(disk)}
        <div>
          <div class="flex justify-between items-center mb-0.5">
            <span class="text-[10px] text-slate-400 truncate max-w-[60%]" title={disk.mount_point}>{disk.mount_point}</span>
            <span class="text-[10px] font-mono text-slate-500">{pct.toFixed(0)}%</span>
          </div>
          <div class="progress-track h-1">
            <div
              class="progress-fill {pct >= 90 ? 'bg-rose-500/80' : pct >= 75 ? 'bg-amber-500/80' : 'bg-cyan-500/50'}"
              style="width: {pct}%"
            ></div>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  <!-- T-12: Disk I/O shown as B/s rate (not cumulative bytes) -->
  <div class="flex gap-4 py-2.5 px-3 bg-white/[0.02] rounded-lg border border-white/[0.03]">
    <div class="flex items-center gap-2">
      <svg class="w-3 h-3 text-cyan-500/60" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3"></path>
      </svg>
      <span class="text-[11px] text-slate-500">Read</span>
      <span class="text-[12px] font-mono text-slate-300 tabular-nums">{formatRate(totalRead)}</span>
    </div>
    <div class="h-4 w-px bg-white/[0.06]"></div>
    <div class="flex items-center gap-2">
      <svg class="w-3 h-3 text-purple-500/60" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"></path>
      </svg>
      <span class="text-[11px] text-slate-500">Write</span>
      <span class="text-[12px] font-mono text-slate-300 tabular-nums">{formatRate(totalWritten)}</span>
    </div>
  </div>
</div>
