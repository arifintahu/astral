<script lang="ts">
  import type { DynamicConfig } from '../types';

  let { open, onClose, refreshRate, onRefreshRateChange, onLogout }: {
    open: boolean;
    onClose: () => void;
    refreshRate: number;
    onRefreshRateChange: (r: number) => void;
    onLogout: () => void;
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
      if (r.ok) { config = await r.json(); webhookInput = config.slack_webhook ?? ''; }
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

{#if open}
  <!-- Backdrop -->
  <div class="anim-backdrop" onclick={onClose} role="presentation"
       style="position:fixed;inset:0;z-index:40;background:rgba(0,0,0,0.5);cursor:default"></div>

  <!-- Drawer -->
  <div class="anim-drawer nice-scroll" role="dialog" aria-label="Settings"
       style="position:fixed;right:0;top:0;bottom:0;z-index:50;width:440px;max-width:100vw;
              background:var(--bg-1);border-left:1px solid var(--line);
              display:flex;flex-direction:column;overflow-y:auto">

    <!-- Header -->
    <div style="display:flex;align-items:center;justify-content:space-between;padding:20px 24px;border-bottom:1px solid var(--line);flex-shrink:0">
      <span style="font-size:15px;font-weight:600;color:var(--ink)">Settings</span>
      <button onclick={onClose} class="btn" aria-label="Close settings" style="width:28px;height:28px;padding:0;border-radius:6px">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="var(--ink-3)" stroke-width="2" stroke-linecap="round">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <!-- Body -->
    <div style="flex:1;padding:24px;display:flex;flex-direction:column;gap:28px;overflow-y:auto" class="nice-scroll">

      <!-- General -->
      <section>
        <div class="eyebrow" style="margin-bottom:14px">General</div>
        <div style="display:flex;flex-direction:column;gap:12px">
          <!-- Refresh rate -->
          <div>
            <div style="font-size:12px;color:var(--ink-2);margin-bottom:8px">Refresh rate</div>
            <div class="seg-control">
              {#each RATES as r}
                <button class="seg-btn {refreshRate === r ? 'active' : ''}"
                        onclick={() => onRefreshRateChange(r)}>{r}s</button>
              {/each}
            </div>
          </div>
          <!-- Process monitoring toggle -->
          <div class="surface-2" style="padding:10px 14px;display:flex;align-items:center;justify-content:space-between;cursor:pointer"
               onclick={() => save({ enable_process_list: !config.enable_process_list })}
               role="button" tabindex="0"
               onkeydown={e => e.key === 'Enter' && save({ enable_process_list: !config.enable_process_list })}>
            <span style="font-size:13px;color:var(--ink-2)">Process monitoring</span>
            <div style="width:36px;height:20px;border-radius:10px;position:relative;flex-shrink:0;transition:background 0.2s;background:{config.enable_process_list ? 'var(--accent-soft)' : 'var(--bg-2)'};border:1px solid {config.enable_process_list ? 'var(--accent-line)' : 'var(--line)'}">
              <div style="position:absolute;top:2px;width:14px;height:14px;border-radius:50%;transition:left 0.2s,background 0.2s;left:{config.enable_process_list ? '18px' : '2px'};background:{config.enable_process_list ? 'var(--accent)' : 'var(--ink-4)'}"></div>
            </div>
          </div>
        </div>
      </section>

      <!-- Alerts -->
      <section>
        <div class="eyebrow" style="margin-bottom:14px">Alerts</div>
        <div style="display:flex;flex-direction:column;gap:16px">
          <!-- CPU threshold -->
          <div>
            <div style="display:flex;justify-content:space-between;margin-bottom:6px">
              <span style="font-size:12px;color:var(--ink-2)">CPU threshold</span>
              <span class="tnum font-mono" style="font-size:12px;color:var(--accent)">{config.alert_cpu.toFixed(0)}%</span>
            </div>
            <input type="range" min="50" max="100" step="1" value={config.alert_cpu}
                   oninput={e => config.alert_cpu = parseFloat((e.target as HTMLInputElement).value)}
                   onchange={e => save({ alert_cpu: parseFloat((e.target as HTMLInputElement).value) })}
                   style="width:100%;accent-color:var(--accent)" />
          </div>
          <!-- RAM threshold -->
          <div>
            <div style="display:flex;justify-content:space-between;margin-bottom:6px">
              <span style="font-size:12px;color:var(--ink-2)">Memory threshold</span>
              <span class="tnum font-mono" style="font-size:12px;color:var(--accent)">{config.alert_ram.toFixed(0)}%</span>
            </div>
            <input type="range" min="50" max="100" step="1" value={config.alert_ram}
                   oninput={e => config.alert_ram = parseFloat((e.target as HTMLInputElement).value)}
                   onchange={e => save({ alert_ram: parseFloat((e.target as HTMLInputElement).value) })}
                   style="width:100%;accent-color:var(--accent)" />
          </div>
          <!-- Slack webhook -->
          <div>
            <div style="font-size:12px;color:var(--ink-2);margin-bottom:6px">Slack webhook URL</div>
            <div style="display:flex;gap:8px">
              <input type="url" bind:value={webhookInput}
                     placeholder="https://hooks.slack.com/…"
                     class="font-mono focus-ring"
                     style="flex:1;background:var(--bg-2);border:1px solid var(--line);border-radius:8px;padding:8px 10px;font-size:11px;color:var(--ink);outline:none;min-width:0" />
              <button onclick={() => save({ slack_webhook: webhookInput || null })}
                      disabled={saving} class="btn" style="flex-shrink:0">Save</button>
            </div>
            {#if config.slack_webhook}
              <div style="font-size:10px;color:var(--ok);margin-top:4px">Webhook active</div>
            {/if}
          </div>
        </div>
      </section>

      <!-- Data -->
      <section>
        <div class="eyebrow" style="margin-bottom:14px">Data</div>
        <div style="display:flex;flex-direction:column;gap:12px">
          <div>
            <div style="font-size:12px;color:var(--ink-2);margin-bottom:8px">Retention period</div>
            <div class="seg-control">
              {#each RETAIN as d}
                <button class="seg-btn {config.retention_days === d ? 'active' : ''}"
                        onclick={() => save({ retention_days: d })}>{d}d</button>
              {/each}
            </div>
          </div>
          <a href="/api/history/export?window=all" download="astral-history.csv"
             style="font-size:12px;color:var(--accent);text-decoration:none;display:inline-flex;align-items:center;gap:4px">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
              <polyline points="7 10 12 15 17 10"/>
              <line x1="12" y1="15" x2="12" y2="3"/>
            </svg>
            Export CSV
          </a>
        </div>
      </section>

      <!-- Account -->
      <section>
        <div class="eyebrow" style="margin-bottom:14px">Account</div>
        <div style="display:flex;flex-direction:column;gap:8px">
          <button onclick={revokeAll} disabled={revoking}
                  style="padding:9px 14px;border-radius:8px;border:1px solid rgba(244,63,94,0.3);background:var(--crit-soft);color:var(--crit);font-size:12px;font-weight:500;cursor:pointer;text-align:left;font-family:inherit;transition:opacity 0.15s"
                  class:opacity-50={revoking}>
            {revoking ? 'Revoking…' : 'Revoke all sessions'}
          </button>
          <button onclick={() => { onClose(); onLogout(); }}
                  style="padding:9px 14px;border-radius:8px;border:1px solid rgba(244,63,94,0.2);background:transparent;color:var(--crit);font-size:12px;font-weight:500;cursor:pointer;text-align:left;font-family:inherit;opacity:0.85">
            Sign out
          </button>
        </div>
      </section>
    </div>

    <!-- Footer -->
    <div style="padding:16px 24px;border-top:1px solid var(--line);display:flex;align-items:center;justify-content:space-between;flex-shrink:0">
      <span class="font-mono" style="font-size:11px;color:var(--ink-4)">Astral v1.1.0</span>
      <button onclick={onClose} class="btn btn-primary">Done</button>
    </div>
  </div>
{/if}
