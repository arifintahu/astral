<script lang="ts">
  let { refreshRate, onRefreshRateChange, onLogout }: {
    refreshRate: number,
    onRefreshRateChange: (rate: number) => void,
    onLogout: () => void,
  } = $props();

  let open = $state(false);

  const rates = [
    { value: 1, label: '1s' },
    { value: 2, label: '2s' },
    { value: 3, label: '3s' },
    { value: 5, label: '5s' },
  ];
</script>

<div class="relative">
  <button
    onclick={() => open = !open}
    aria-label="Settings"
    class="flex items-center justify-center w-8 h-8 rounded-lg bg-white/[0.04] border border-white/[0.06] hover:bg-white/[0.08] hover:border-white/[0.1] transition-all cursor-pointer"
  >
    <svg class="w-4 h-4 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
    </svg>
  </button>

  {#if open}
    <!-- Backdrop -->
    <button class="fixed inset-0 z-40 cursor-default" onclick={() => open = false} tabindex="-1" aria-label="Close settings"></button>

    <!-- Dropdown -->
    <div class="absolute right-0 top-full mt-2 z-50 w-56 animate-fade-in">
      <div class="bg-slate-950 border border-white/[0.1] rounded-xl shadow-2xl p-4 space-y-4">
        <!-- Refresh Rate -->
        <div>
          <div class="text-[10px] text-slate-500 uppercase tracking-[0.15em] font-semibold mb-2">Refresh Rate</div>
          <div class="flex gap-1.5">
            {#each rates as rate}
              <button
                class="flex-1 px-2 py-1.5 text-[11px] font-semibold rounded-lg transition-all duration-200 cursor-pointer
                       {refreshRate === rate.value
                         ? 'bg-cyan-500/15 text-cyan-300 border border-cyan-500/20'
                         : 'text-slate-500 bg-white/[0.03] border border-white/[0.05] hover:text-slate-300 hover:bg-white/[0.06]'}"
                onclick={() => { onRefreshRateChange(rate.value); }}
              >
                {rate.label}
              </button>
            {/each}
          </div>
        </div>

        <div class="h-px bg-white/[0.06]"></div>

        <!-- Logout -->
        <button
          onclick={() => { open = false; onLogout(); }}
          class="w-full flex items-center gap-2 px-3 py-2 text-[12px] text-slate-400 hover:text-rose-400 hover:bg-rose-500/5 rounded-lg transition-colors cursor-pointer"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
          </svg>
          Sign out
        </button>
      </div>
    </div>
  {/if}
</div>
