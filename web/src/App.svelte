<script lang="ts">
  import { onMount } from 'svelte';
  import type { SystemMetrics, AlertEvent } from './lib/types';
  import TopBar             from './lib/components/TopBar.svelte';
  import CpuCard            from './lib/components/CpuCard.svelte';
  import MemoryCard         from './lib/components/MemoryCard.svelte';
  import NetworkCard        from './lib/components/NetworkCard.svelte';
  import DiskCard           from './lib/components/DiskCard.svelte';
  import HistoryChart       from './lib/components/HistoryChart.svelte';
  import ProcessList        from './lib/components/ProcessList.svelte';
  import Toast              from './lib/components/Toast.svelte';
  import Login              from './lib/components/Login.svelte';
  import AlertHistoryDrawer from './lib/components/AlertHistoryDrawer.svelte';
  import Settings           from './lib/components/Settings.svelte';

  let metrics: SystemMetrics | null = $state(null);
  let cpuHistory: number[] = $state([]);
  let txHistory:  number[] = $state([]);
  let rxHistory:  number[] = $state([]);
  let alerts: AlertEvent[]  = $state([]);
  let authenticated = $state(false);
  let refreshRate   = $state(1);
  let lastUpdate    = $state(0);
  let lastRefreshTs = $state(0);
  let showAlertHistory = $state(false);
  let showSettings     = $state(false);

  let eventSource: { close: () => void } | null = null;
  let alertSource:  { close: () => void } | null = null;

  function handleLogin() { authenticated = true; connectStreams(); }

  async function handleLogout() {
    await fetch('/api/logout', { method: 'POST' }).catch(() => {});
    authenticated = false; metrics = null;
    cpuHistory = []; txHistory = []; rxHistory = []; alerts = [];
    showSettings = false; showAlertHistory = false;
    eventSource?.close(); alertSource?.close();
  }

  function handleRefreshRateChange(rate: number) { refreshRate = rate; }
  function dismissAlert(i: number) { alerts = alerts.filter((_, idx) => idx !== i); }
  function connectStreams() { startMetricsStream(); startAlertStream(); }

  function startMetricsStream() {
    const ac = new AbortController();
    fetch('/api/stream', { signal: ac.signal })
      .then(res => {
        if (!res.ok) { if (res.status === 401) handleLogout(); return; }
        const reader = res.body!.getReader();
        const dec = new TextDecoder();
        let buf = '';
        function pump(): Promise<void> {
          return reader.read().then(({ done, value }) => {
            if (done) return;
            buf += dec.decode(value, { stream: true });
            const lines = buf.split('\n');
            buf = lines.pop() ?? '';
            for (const line of lines) {
              if (!line.startsWith('data: ')) continue;
              const now = Date.now();
              if (now - lastUpdate < refreshRate * 1000 - 100) continue;
              lastUpdate = now;
              lastRefreshTs = Math.floor(now / 1000);
              try {
                const d: SystemMetrics = JSON.parse(line.slice(6));
                metrics = d;
                cpuHistory = [...cpuHistory, d.cpu_usage].slice(-60);
                txHistory  = [...txHistory,  d.network_tx].slice(-30);
                rxHistory  = [...rxHistory,  d.network_rx].slice(-30);
              } catch { /**/ }
            }
            return pump();
          });
        }
        pump();
      })
      .catch(e => {
        if (e.name !== 'AbortError' && authenticated) setTimeout(startMetricsStream, 3000);
      });
    eventSource = { close: () => ac.abort() };
  }

  function startAlertStream() {
    const ac = new AbortController();
    fetch('/api/alerts', { signal: ac.signal })
      .then(res => {
        if (!res.ok) return;
        const reader = res.body!.getReader();
        const dec = new TextDecoder();
        let buf = '', nextIsData = false;
        function pump(): Promise<void> {
          return reader.read().then(({ done, value }) => {
            if (done) return;
            buf += dec.decode(value, { stream: true });
            const lines = buf.split('\n');
            buf = lines.pop() ?? '';
            for (const line of lines) {
              if (line === 'event: alert') { nextIsData = true; continue; }
              if (nextIsData && line.startsWith('data: ')) {
                try {
                  const a: AlertEvent = JSON.parse(line.slice(6));
                  alerts = [...alerts, a].slice(-5);
                  setTimeout(() => { if (alerts.length) alerts = alerts.slice(1); }, 10000);
                } catch { /**/ }
                nextIsData = false;
              }
            }
            return pump();
          });
        }
        pump();
      })
      .catch(e => {
        if (e.name !== 'AbortError' && authenticated) setTimeout(startAlertStream, 5000);
      });
    alertSource = { close: () => ac.abort() };
  }

  function fmtTs(ts: number): string {
    if (!ts) return '—';
    return new Date(ts * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }

  onMount(() => {
    fetch('/api/auth/check')
      .then(r => { if (r.ok) { authenticated = true; connectStreams(); } })
      .catch(() => {});
    return () => { eventSource?.close(); alertSource?.close(); };
  });
</script>

{#if !authenticated}
  <Login onLogin={handleLogin} />
{:else}
  <Toast {alerts} onDismiss={dismissAlert} />
  <AlertHistoryDrawer open={showAlertHistory} onClose={() => showAlertHistory = false} />
  <Settings
    open={showSettings}
    onClose={() => showSettings = false}
    {refreshRate}
    onRefreshRateChange={handleRefreshRateChange}
    onLogout={handleLogout}
  />

  <div style="max-width:1440px;margin:0 auto;padding:26px 28px">
    <TopBar
      {metrics}
      {refreshRate}
      {alerts}
      onShowSettings={() => showSettings = true}
      onLogout={handleLogout}
      onShowAlertHistory={() => showAlertHistory = true}
    />

    {#if metrics}
      <!-- Metric cards grid -->
      <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:20px;margin-bottom:20px">
        <div class="anim-fade-up stagger-1" style="min-height:160px">
          <CpuCard usage={metrics.cpu_usage} cores={metrics.cpu_cores} history={cpuHistory} load={metrics.cpu_load} />
        </div>
        <div class="anim-fade-up stagger-2" style="min-height:160px">
          <MemoryCard used={metrics.used_memory} total={metrics.total_memory} swap_used={metrics.used_swap} swap_total={metrics.total_swap} />
        </div>
        <div class="anim-fade-up stagger-3" style="min-height:160px">
          <NetworkCard tx={metrics.network_tx} rx={metrics.network_rx} {txHistory} {rxHistory} />
        </div>
        <div class="anim-fade-up stagger-4" style="min-height:160px">
          <DiskCard disks={metrics.disks} />
        </div>
      </div>

      <!-- Bottom row: history + process list -->
      <div class="anim-fade-up stagger-5" style="display:grid;grid-template-columns:2fr 1fr;gap:20px;height:400px">
        <HistoryChart totalMemory={metrics.total_memory} />
        <ProcessList processes={metrics.processes} totalMemory={metrics.total_memory} />
      </div>

      <!-- Footer -->
      <div style="margin-top:16px;padding:10px 2px;border-top:1px solid var(--line);display:flex;align-items:center;gap:12px;flex-wrap:wrap">
        <span class="font-mono" style="font-size:10px;color:var(--ink-4)">{metrics.hostname}</span>
        <span style="color:var(--line-2)">·</span>
        <span style="font-size:10px;color:var(--ink-4)">Updated {fmtTs(lastRefreshTs)}</span>
        <span style="color:var(--line-2)">·</span>
        <span style="font-size:10px;color:var(--ink-4)">Rate {refreshRate}s</span>
        <span style="color:var(--line-2)">·</span>
        <span style="font-size:10px;color:var(--ink-4)">Retention {metrics.os_name ? '—' : '—'}</span>
      </div>

    {:else}
      <!-- Skeleton loading -->
      <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:20px;margin-bottom:20px">
        {#each [0,1,2,3] as i}
          <div class="skeleton" style="height:160px;animation-delay:{i * 60}ms"></div>
        {/each}
      </div>
      <div style="display:grid;grid-template-columns:2fr 1fr;gap:20px">
        <div class="skeleton" style="height:400px"></div>
        <div class="skeleton" style="height:400px"></div>
      </div>
    {/if}
  </div>
{/if}
