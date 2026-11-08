<script lang="ts">
  import type { DynamicConfig } from '../types';
  import Drawer from './Drawer.svelte';

  let { open, onClose, refreshRate, onRefreshRateChange, onLogout, onConfigChange }: {
    open: boolean;
    onClose: () => void;
    refreshRate: number;
    onRefreshRateChange: (r: number) => void;
    onLogout: () => void;
    onConfigChange: (c: DynamicConfig) => void;
  } = $props();

  let saving   = $state(false);
  let revoking = $state(false);
  let config: DynamicConfig = $state({
    enable_process_list: false,
    alert_cpu: 90,
    alert_ram: 90,
    retention_days: 7,
    slack_webhook: null,
  });
  let webhookInput = $state('');

  const RATES  = [1, 2, 5, 10];
  const RETAIN = [7, 30, 90];

  async function loadConfig() {
    try {
      const r = await fetch('/api/settings');
      if (r.ok) { config = await r.json(); webhookInput = config.slack_webhook ?? ''; onConfigChange(config); }
    } catch {/**/}
  }

  async function save(patch: Partial<DynamicConfig> & { slack_webhook?: string | null }) {
    saving = true;
    try {
      await fetch('/api/settings', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(patch),
      });
      await loadConfig();
    } finally { saving = false; }
  }

  async function revokeAll() {
    revoking = true;
    try {
      await fetch('/api/sessions/revoke', { method: 'POST' });
      onLogout();
    } finally { revoking = false; }
  }

  $effect(() => { if (open) loadConfig(); });
</script>

<Drawer {open} {onClose} title="Settings" width={460}>
  {#snippet children()}
    <div class="flex flex-col px-6 pb-6 pt-2">

      <!-- General -->
      <section class="flex flex-col gap-3.5 border-b pb-6 pt-5" style="border-color:var(--line)">
        <span class="eyebrow">General</span>
        <div class="flex flex-wrap items-center justify-between gap-3">
          <span class="text-[14px]" style="color:var(--ink-2)">Refresh rate</span>
          <div class="seg2" role="group" aria-label="Refresh rate">
            {#each RATES as r}
              <button type="button" aria-pressed={refreshRate === r} onclick={() => onRefreshRateChange(r)}><span>{r}s</span></button>
            {/each}
          </div>
        </div>
        <button type="button" role="switch" aria-checked={config.enable_process_list}
                class="flex min-h-[60px] w-full cursor-pointer items-center justify-between gap-4 rounded-[14px] border px-4 py-2 text-left"
                style="border-color:var(--line);background:var(--bg-2);color:var(--ink);font-family:inherit"
                onclick={() => save({ enable_process_list: !config.enable_process_list })}>
          <span class="flex flex-col gap-0.5">
            <span class="text-[14px]">Process monitoring</span>
            <span class="text-[12px]" style="color:var(--ink-3)">Lists the busiest processes on the dashboard</span>
          </span>
          <span class="switch"></span>
        </button>
      </section>

      <!-- Alerts -->
      <section class="flex flex-col gap-3 border-b py-6" style="border-color:var(--line)">
        <span class="eyebrow">Alerts</span>

        <div class="flex flex-col">
          <div class="flex items-baseline justify-between">
            <label for="alert-cpu" class="text-[14px]" style="color:var(--ink-2)">CPU threshold</label>
            <span class="font-mono tnum text-[13px]" style="color:var(--accent)">{config.alert_cpu.toFixed(0)}%</span>
          </div>
          <input id="alert-cpu" type="range" min="50" max="100" step="1" value={config.alert_cpu}
                 class="m-0 h-11 w-full" style="accent-color:var(--accent)"
                 oninput={e => config.alert_cpu = parseFloat((e.target as HTMLInputElement).value)}
                 onchange={e => save({ alert_cpu: parseFloat((e.target as HTMLInputElement).value) })} />
        </div>

        <div class="flex flex-col">
          <div class="flex items-baseline justify-between">
            <label for="alert-ram" class="text-[14px]" style="color:var(--ink-2)">Memory threshold</label>
            <span class="font-mono tnum text-[13px]" style="color:var(--accent)">{config.alert_ram.toFixed(0)}%</span>
          </div>
          <input id="alert-ram" type="range" min="50" max="100" step="1" value={config.alert_ram}
                 class="m-0 h-11 w-full" style="accent-color:var(--accent)"
                 oninput={e => config.alert_ram = parseFloat((e.target as HTMLInputElement).value)}
                 onchange={e => save({ alert_ram: parseFloat((e.target as HTMLInputElement).value) })} />
        </div>

        <div class="flex flex-col gap-2">
          <label for="slack-hook" class="text-[14px]" style="color:var(--ink-2)">Slack webhook URL</label>
          <div class="flex gap-2">
            <input id="slack-hook" type="url" bind:value={webhookInput} placeholder="https://hooks.slack.com/…"
                   class="field font-mono h-11 min-w-0 flex-1 px-3.5 text-[12px]" style="border-radius:12px" />
            <button type="button" class="btn-outline" disabled={saving}
                    onclick={() => save({ slack_webhook: webhookInput || null })}>Save</button>
          </div>
          {#if config.slack_webhook}
            <span class="text-[12px]" style="color:var(--accent)">Webhook active</span>
          {/if}
        </div>
      </section>

      <!-- Data -->
      <section class="flex flex-col gap-3.5 border-b py-6" style="border-color:var(--line)">
        <span class="eyebrow">Data</span>
        <div class="flex flex-wrap items-center justify-between gap-3">
          <span class="text-[14px]" style="color:var(--ink-2)">Retention period</span>
          <div class="seg2" role="group" aria-label="Retention period">
            {#each RETAIN as d}
              <button type="button" aria-pressed={config.retention_days === d} onclick={() => save({ retention_days: d })}><span>{d}d</span></button>
            {/each}
          </div>
        </div>
        <a href="/api/history/export?window=all" download="astral-history.csv"
           class="inline-flex min-h-11 items-center gap-2 text-[14px] font-medium no-underline" style="color:var(--accent)">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" />
          </svg>
          Export history as CSV
        </a>
      </section>

      <!-- Account -->
      <section class="flex flex-col gap-2.5 pt-6">
        <span class="eyebrow">Account</span>
        <button type="button" class="btn-danger" disabled={revoking} onclick={revokeAll}>
          {revoking ? 'Revoking…' : 'Revoke all sessions'}
        </button>
        <button type="button" class="btn-outline justify-start" style="height:auto;min-height:44px;padding:0 16px"
                onclick={() => { onClose(); onLogout(); }}>Sign out</button>
      </section>
    </div>
  {/snippet}

  {#snippet footer()}
    <div class="flex items-center justify-between gap-3 px-6 py-3.5">
      <span class="text-[12px]" style="color:var(--ink-3)">Changes save automatically</span>
      <button type="button" class="h-11 cursor-pointer rounded-xl border-0 px-[26px] text-[14px] font-semibold"
              style="background:var(--accent);color:#09090b;font-family:inherit" onclick={onClose}>Done</button>
    </div>
  {/snippet}
</Drawer>
