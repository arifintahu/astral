<script lang="ts">
  import type { DiskInfo } from '../types';

  let { disks }: { disks: DiskInfo[] } = $props();

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
</script>

<div class="glass-card p-6">
  <div class="flex justify-between items-center mb-4">
    <h3 class="text-slate-400 text-xs font-bold uppercase tracking-widest">Storage</h3>
    <span class="text-xs font-mono text-cyan-300 bg-cyan-900/30 px-2 py-1 rounded border border-cyan-800/50 shadow-[0_0_10px_rgba(6,182,212,0.1)]">{disks.length} Mounts</span>
  </div>
  
  <div class="flex flex-col gap-6 overflow-y-auto max-h-52 scrollbar-thin scrollbar-thumb-white/20 scrollbar-track-transparent pr-2">
    {#each disks as disk}
      {@const used = disk.total_space - disk.available_space}
      {@const percent = (used / disk.total_space) * 100}
      <div>
        <div class="flex justify-between text-xs text-slate-300 mb-2">
          <span class="font-mono font-bold truncate max-w-[60%] text-cyan-100" title={disk.mount_point}>{disk.mount_point}</span>
          <span class="text-slate-500 font-mono whitespace-nowrap">{formatBytes(used)} / {formatBytes(disk.total_space)}</span>
        </div>
        <div class="w-full bg-slate-900/50 rounded-full h-2 overflow-hidden border border-white/5">
          <div 
            class="h-full rounded-full transition-all duration-500 ease-out bg-gradient-to-r from-purple-500 to-cyan-500 shadow-[0_0_8px_rgba(168,85,247,0.5)]" 
            style="width: {percent}%"
          ></div>
        </div>
      </div>
    {/each}
  </div>
</div>
