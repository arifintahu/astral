<script lang="ts">
  import type { DiskInfo } from '../types';

  let { disks }: { disks: DiskInfo[] } = $props();
  let sortedDisks = $derived([...disks].sort((a, b) => b.total_space - a.total_space));

  let rootDisk = $derived(disks.find(d => d.mount_point === '/') || disks[0]);
  let totalSpace = $derived(rootDisk ? rootDisk.total_space : 0);
  let totalUsed = $derived(rootDisk ? (rootDisk.total_space - rootDisk.available_space) : 0);
  let utilization = $derived(totalSpace > 0 ? (totalUsed / totalSpace) * 100 : 0);

  let barColor = $derived(
    utilization >= 90 ? 'bg-rose-500/80' : utilization >= 75 ? 'bg-amber-500/80' : 'bg-gradient-to-r from-purple-500/70 to-cyan-500/70'
  );

  function diskBarColor(percent: number): string {
    if (percent >= 90) return 'bg-rose-500/60';
    if (percent >= 75) return 'bg-amber-500/50';
    return 'bg-slate-400/20';
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }
</script>

<div class="glass-card p-6">
  <div class="flex justify-between items-center mb-5">
    <h3 class="metric-label">Storage</h3>
    <span class="metric-badge badge-cyan">{disks.length} mounts</span>
  </div>

  <!-- Primary disk -->
  <div class="mb-5">
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
      <div
        class="progress-fill {barColor}"
        style="width: {utilization}%"
      ></div>
    </div>
  </div>

  <!-- Secondary disks -->
  {#if sortedDisks.length > 1}
    <div class="flex flex-col gap-3 overflow-y-auto max-h-32 custom-scrollbar pr-1">
      {#each sortedDisks as disk}
        {@const used = disk.total_space - disk.available_space}
        {@const percent = disk.total_space > 0 ? (used / disk.total_space) * 100 : 0}
        <div>
          <div class="flex justify-between text-[11px] mb-1">
            <span class="font-mono font-medium text-slate-400 truncate max-w-[55%]" title={disk.mount_point}>{disk.mount_point}</span>
            <span class="text-slate-600 font-mono tabular-nums">{formatBytes(used)} / {formatBytes(disk.total_space)}</span>
          </div>
          <div class="progress-track h-1">
            <div
              class="progress-fill {diskBarColor(percent)}"
              style="width: {percent}%"
            ></div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
