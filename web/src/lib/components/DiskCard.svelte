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

<div class="bg-zinc-800 rounded-lg p-4 shadow-md border border-zinc-700 overflow-y-auto max-h-64 scrollbar-thin scrollbar-thumb-zinc-600 scrollbar-track-zinc-800">
  <div class="flex justify-between items-center mb-4 sticky top-0 bg-zinc-800 pb-2 border-b border-zinc-700 z-10">
    <h3 class="text-zinc-400 text-sm font-semibold uppercase tracking-wider">Storage</h3>
    <span class="text-xs text-zinc-500 font-mono bg-zinc-900 px-2 py-1 rounded">{disks.length} Mounts</span>
  </div>
  
  <div class="flex flex-col gap-4">
    {#each disks as disk}
      {@const used = disk.total_space - disk.available_space}
      {@const percent = (used / disk.total_space) * 100}
      <div>
        <div class="flex justify-between text-xs text-zinc-300 mb-1">
          <span class="font-mono font-bold truncate max-w-[50%]">{disk.mount_point}</span>
          <span class="text-zinc-500 font-mono">{formatBytes(used)} / {formatBytes(disk.total_space)}</span>
        </div>
        <div class="w-full bg-zinc-900 rounded-full h-2 overflow-hidden border border-zinc-700/50">
          <div 
            class="bg-purple-500 h-2 rounded-full transition-all duration-500 ease-out" 
            style="width: {percent}%"
          ></div>
        </div>
      </div>
    {/each}
  </div>
</div>
