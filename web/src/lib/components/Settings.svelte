<script lang="ts">
  import { onMount } from 'svelte';
  import type { DynamicConfig } from '../types';

  let { refreshRate, onRefreshRateChange, onLogout }: {
    refreshRate: number,
    onRefreshRateChange: (rate: number) => void,
    onLogout: () => void,
  } = $props();

  let open = $state(false);
  let saving = $state(false);
  let saveError = $state('');

  // Local editable copies of server config
  let config: DynamicConfig = $state({
    enable_process_list: false,
    alert_cpu: 90,
    alert_ram: 90,
    retention_days: 7,
    slack_webhook: null,
  });
  let slackWebhookInput = $state('');

  const rates = [
    { value: 1, label: '1s' },
    { value: 2, label: '2s' },
    { value: 3, label: '3s' },
    { value: 5, label: '5s' },
  ];

  const retentionOptions = [
    { value: 7, label: '7d' },
    { value: 30, label: '30d' },
    { value: 90, label: '90d' },
  ];

  async function fetchConfig() {
    try {
      const res = await fetch('/api/settings');
      if (res.ok) {
        const data: DynamicConfig = await res.json();
        config = data;
        slackWebhookInput = data.slack_webhook ?? '';
      }
    } catch { /* ignore */ }
  }

  async function saveConfig(patch: Partial<DynamicConfig> & { slack_webhook?: string }) {
    saving = true;
    saveError = '';
    try {
      const res = await fetch('/api/settings', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(patch),
      });
      if (!res.ok) saveError = 'Save failed';
      else await fetchConfig();
    } catch {
      saveError = 'Network error';
    } finally {
      saving = false;
    }
  }

  $effect(() => {
    if (open) fetchConfig();
  });
</script>

<div class="relative">
  <button
    onclick={() => open = !open}
    aria-label="Settings"
    class="flex items-center justify-center w-8 h-8 rounded-lg bg-white/[0.04] border border-white/[0.06]
           hover:bg-white/[0.08] hover:border-white/[0.1] transition-all cursor-pointer"
  >
    <svg class="w-4 h-4 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
    </svg>
  </button>

  {#if open}
    <button class="fixed inset-0 z-40 cursor-default" onclick={() => open = false} tabindex="-1" aria-label="Close settings"></button>

    <div class="absolute right-0 top-full mt-2 z-50 w-72 animate-fade-in">
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
                onclick={() => onRefreshRateChange(rate.value)}
              >{rate.label}</button>
            {/each}
          </div>
        </div>

        <div class="h-px bg-white/[0.06]"></div>

        <!-- T-02: Process list toggle -->
        <div>
          <div class="text-[10px] text-slate-500 uppercase tracking-[0.15em] font-semibold mb-2">Processes</div>
          <button
            onclick={() => saveConfig({ enable_process_list: !config.enable_process_list })}
            class="w-full flex items-center justify-between px-3 py-2 rounded-lg bg-white/[0.03] border border-white/[0.05]
                   hover:bg-white/[0.06] transition-colors cursor-pointer"
          >
            <span class="text-[12px] text-slate-300">Enable process list</span>
            <div class="w-8 h-4 rounded-full transition-colors relative flex-shrink-0 {config.enable_process_list ? 'bg-cyan-500/40' : 'bg-white/[0.08]'}">
              <div class="absolute top-0.5 w-3 h-3 rounded-full transition-all {config.enable_process_list ? 'left-4 bg-cyan-400' : 'left-0.5 bg-slate-500'}"></div>
            </div>
          </button>
        </div>

        <div class="h-px bg-white/[0.06]"></div>

        <!-- T-03: Alert thresholds -->
        <div>
          <div class="text-[10px] text-slate-500 uppercase tracking-[0.15em] font-semibold mb-2">Alert Thresholds</div>
          <div class="space-y-2">
            <div class="flex items-center gap-2">
              <span class="text-[11px] text-slate-400 w-14 flex-shrink-0">CPU %</span>
              <input
                type="number"
                min="1" max="100"
                bind:value={config.alert_cpu}
                class="flex-1 bg-white/[0.04] border border-white/[0.08] rounded-lg px-2 py-1 text-[12px] text-slate-200
                       font-mono focus:outline-none focus:border-cyan-500/40 focus:bg-white/[0.06]"
              />
              <button
                onclick={() => saveConfig({ alert_cpu: config.alert_cpu })}
                disabled={saving}
                class="px-2 py-1 text-[10px] font-semibold rounded-lg bg-cyan-500/10 border border-cyan-500/20
                       text-cyan-300 hover:bg-cyan-500/20 transition-colors cursor-pointer disabled:opacity-50"
              >Save</button>
            </div>
            <div class="flex items-center gap-2">
              <span class="text-[11px] text-slate-400 w-14 flex-shrink-0">RAM %</span>
              <input
                type="number"
                min="1" max="100"
                bind:value={config.alert_ram}
                class="flex-1 bg-white/[0.04] border border-white/[0.08] rounded-lg px-2 py-1 text-[12px] text-slate-200
                       font-mono focus:outline-none focus:border-cyan-500/40 focus:bg-white/[0.06]"
              />
              <button
                onclick={() => saveConfig({ alert_ram: config.alert_ram })}
                disabled={saving}
                class="px-2 py-1 text-[10px] font-semibold rounded-lg bg-cyan-500/10 border border-cyan-500/20
                       text-cyan-300 hover:bg-cyan-500/20 transition-colors cursor-pointer disabled:opacity-50"
              >Save</button>
            </div>
          </div>
        </div>

        <div class="h-px bg-white/[0.06]"></div>

        <!-- T-15: Data retention -->
        <div>
          <div class="text-[10px] text-slate-500 uppercase tracking-[0.15em] font-semibold mb-2">Data Retention</div>
          <div class="flex gap-1.5">
            {#each retentionOptions as opt}
              <button
                onclick={() => saveConfig({ retention_days: opt.value })}
                class="flex-1 px-2 py-1.5 text-[11px] font-semibold rounded-lg transition-all duration-200 cursor-pointer
                       {config.retention_days === opt.value
                         ? 'bg-cyan-500/15 text-cyan-300 border border-cyan-500/20'
                         : 'text-slate-500 bg-white/[0.03] border border-white/[0.05] hover:text-slate-300 hover:bg-white/[0.06]'}"
              >{opt.label}</button>
            {/each}
          </div>
        </div>

        <div class="h-px bg-white/[0.06]"></div>

        <!-- T-17: Slack webhook -->
        <div>
          <div class="text-[10px] text-slate-500 uppercase tracking-[0.15em] font-semibold mb-2">Slack Alerts</div>
          <div class="flex gap-1.5">
            <input
              type="url"
              placeholder="https://hooks.slack.com/…"
              bind:value={slackWebhookInput}
              class="flex-1 bg-white/[0.04] border border-white/[0.08] rounded-lg px-2 py-1.5 text-[11px] text-slate-300
                     font-mono focus:outline-none focus:border-cyan-500/40 focus:bg-white/[0.06] placeholder:text-slate-700"
            />
            <button
              onclick={() => saveConfig({ slack_webhook: slackWebhookInput })}
              disabled={saving}
              class="px-2 py-1.5 text-[10px] font-semibold rounded-lg bg-cyan-500/10 border border-cyan-500/20
                     text-cyan-300 hover:bg-cyan-500/20 transition-colors cursor-pointer disabled:opacity-50 flex-shrink-0"
            >Save</button>
          </div>
          {#if config.slack_webhook}
            <p class="text-[10px] text-emerald-600 mt-1.5">Webhook active</p>
          {/if}
          {#if saveError}
            <p class="text-[10px] text-rose-500 mt-1">{saveError}</p>
          {/if}
        </div>

        <div class="h-px bg-white/[0.06]"></div>

        <!-- Sign out -->
        <button
          onclick={() => { open = false; onLogout(); }}
          class="w-full flex items-center gap-2 px-3 py-2 text-[12px] text-slate-400 hover:text-rose-400
                 hover:bg-rose-500/5 rounded-lg transition-colors cursor-pointer"
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
