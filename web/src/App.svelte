<script lang="ts">
  import { onMount } from 'svelte';
  import type { SystemMetrics, AlertEvent, DynamicConfig, LiveSeries, MetricKey, TileData } from './lib/types';
  import { fmtBytes, fmtRate, niceMaxRate, rateParts } from './lib/format';
  import { DEFAULT_THRESHOLDS, diskPercent, hasLoadAverage, memPercent, primaryDisk, type Thresholds } from './lib/status';
  import TopBar             from './lib/components/TopBar.svelte';
  import StatusHero         from './lib/components/StatusHero.svelte';
  import MetricTile         from './lib/components/MetricTile.svelte';
  import HeroChart          from './lib/components/HeroChart.svelte';
  import ProcessList        from './lib/components/ProcessList.svelte';
  import SystemDetails      from './lib/components/SystemDetails.svelte';
  import Toast              from './lib/components/Toast.svelte';
  import Login              from './lib/components/Login.svelte';
  import AlertHistoryDrawer from './lib/components/AlertHistoryDrawer.svelte';
  import Settings           from './lib/components/Settings.svelte';
  import SignOutDialog      from './lib/components/SignOutDialog.svelte';

  const EMPTY_LIVE: LiveSeries = { cpu: [], mem: [], rx: [], tx: [], rd: [], wr: [] };
  const LIVE_CAP = 60;

  let metrics: SystemMetrics | null = $state(null);
  let live: LiveSeries = $state({ ...EMPTY_LIVE });
  let alerts: AlertEvent[]  = $state([]);
  let selected: MetricKey = $state('cpu');
  let authenticated = $state(false);
  let refreshRate   = $state(1);
  let lastUpdate    = $state(0);
  let lastRefreshTs = $state(0);
  let showAlertHistory = $state(false);
  let showSettings     = $state(false);
  let showSignOut      = $state(false);
  let thresholds: Thresholds = $state({ ...DEFAULT_THRESHOLDS });

  let eventSource: { close: () => void } | null = null;
  let alertSource:  { close: () => void } | null = null;

  function handleLogin() { authenticated = true; connectStreams(); }

  async function handleLogout() {
    await fetch('/api/logout', { method: 'POST' }).catch(() => {});
    authenticated = false; metrics = null;
    live = { ...EMPTY_LIVE }; alerts = [];
    showSettings = false; showAlertHistory = false; showSignOut = false;
    eventSource?.close(); alertSource?.close();
  }

  function handleRefreshRateChange(rate: number) { refreshRate = rate; }
  function dismissAlert(i: number) { alerts = alerts.filter((_, idx) => idx !== i); }
  function connectStreams() { startMetricsStream(); startAlertStream(); loadThresholds(); }

  function applyConfig(c: DynamicConfig) { thresholds = { cpu: c.alert_cpu, ram: c.alert_ram }; }
  async function loadThresholds() {
    try {
      const r = await fetch('/api/settings');
      if (r.ok) applyConfig(await r.json() as DynamicConfig);
    } catch {/**/}
  }

  const push = (arr: number[], v: number) => [...arr, v].slice(-LIVE_CAP);

  function ingest(d: SystemMetrics) {
    metrics = d;
    live = {
      cpu: push(live.cpu, d.cpu_usage),
      mem: push(live.mem, memPercent(d)),
      rx:  push(live.rx,  d.network_rx),
      tx:  push(live.tx,  d.network_tx),
      rd:  push(live.rd,  d.disks.reduce((s, x) => s + x.read_bytes, 0)),
      wr:  push(live.wr,  d.disks.reduce((s, x) => s + x.written_bytes, 0)),
    };
  }

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
              try { ingest(JSON.parse(line.slice(6)) as SystemMetrics); } catch { /**/ }
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

  /* The four tiles double as the hero chart's tabs. */
  let tiles: TileData[] = $derived.by(() => {
    if (!metrics) return [];
    const m = metrics;
    const d = primaryDisk(m.disks);
    const diskUsed = d ? d.total_space - d.available_space : 0;
    const rxPeak = niceMaxRate(Math.max(...live.rx, 1));
    const rx = rateParts(m.network_rx);
    return [
      { id: 'cpu',  label: 'CPU',     meta: `${m.cpu_cores} cores`,
        value: String(Math.round(m.cpu_usage)), unit: '%', sub: hasLoadAverage(m) ? `Load ${m.cpu_load[0].toFixed(2)}` : 'Load n/a',
        pct: m.cpu_usage, zones: true },
      { id: 'mem',  label: 'Memory',  meta: `Swap ${fmtBytes(m.used_swap)}`,
        value: String(Math.round(memPercent(m))), unit: '%', sub: `${fmtBytes(m.used_memory)} of ${fmtBytes(m.total_memory)}`,
        pct: memPercent(m), zones: true },
      { id: 'net',  label: 'Network', meta: 'Download',
        value: rx.v, unit: rx.u, sub: `↑ ${fmtRate(m.network_tx)} upload`,
        pct: (m.network_rx / rxPeak) * 100, zones: false, meterColor: 'var(--violet)' },
      { id: 'disk', label: 'Disk',    meta: `${m.disks.length} mount${m.disks.length === 1 ? '' : 's'}`,
        value: String(Math.round(diskPercent(m))), unit: '%', sub: d ? `${fmtBytes(diskUsed)} of ${fmtBytes(d.total_space)}` : 'No disks detected',
        pct: diskPercent(m), zones: true },
    ];
  });

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
  <AlertHistoryDrawer
    open={showAlertHistory}
    onClose={() => showAlertHistory = false}
    onAdjust={() => { showAlertHistory = false; showSettings = true; }}
  />
  <SignOutDialog open={showSignOut} onCancel={() => showSignOut = false} onConfirm={handleLogout} />
  <Settings
    open={showSettings}
    onClose={() => showSettings = false}
    {refreshRate}
    onRefreshRateChange={handleRefreshRateChange}
    onLogout={handleLogout}
    onConfigChange={applyConfig}
  />

  <div class="mx-auto flex max-w-[1240px] flex-col gap-5 px-[clamp(16px,4vw,32px)] pb-10 pt-4">
    <TopBar
      {metrics}
      {refreshRate}
      {alerts}
      onShowSettings={() => showSettings = true}
      onLogout={() => showSignOut = true}
      onShowAlertHistory={() => showAlertHistory = true}
    />

    {#if metrics}
      <StatusHero {metrics} {thresholds} />

      <!-- Metric strip: 1px gaps over a hairline background draw the dividers -->
      <section
        class="anim-fade-up stagger-2 overflow-hidden rounded-[18px] border"
        style="display:grid;grid-template-columns:repeat(auto-fit,minmax(min(160px,100%),1fr));gap:1px;background:var(--line);border-color:var(--line)"
      >
        {#each tiles as t (t.id)}
          <MetricTile tile={t} active={selected === t.id} onSelect={() => (selected = t.id)} />
        {/each}
      </section>

      <HeroChart {live} totalMemory={metrics.total_memory} metric={selected} />

      <section class="anim-fade-up stagger-4 flex flex-wrap items-stretch gap-5">
        <div class="flex min-w-0 basis-[min(520px,100%)] flex-[2_1_auto] flex-col">
          <ProcessList processes={metrics.processes} />
        </div>
        <div class="flex min-w-0 basis-[min(300px,100%)] flex-[1_1_auto] flex-col">
          <SystemDetails
            load={metrics.cpu_load}
            showLoad={hasLoadAverage(metrics)}
            cores={metrics.cpu_cores}
            swapUsed={metrics.used_swap}
            swapTotal={metrics.total_swap}
            disks={metrics.disks}
          />
        </div>
      </section>

      <footer class="flex flex-wrap justify-between gap-x-6 gap-y-2 pt-1">
        <span class="font-mono text-[12px]" style="color:var(--ink-3)">Updated {fmtTs(lastRefreshTs)} · refresh {refreshRate}s</span>
        <span class="font-mono text-[12px]" style="color:var(--ink-3)">{metrics.hostname} · {metrics.os_name} {metrics.os_version}</span>
      </footer>

    {:else}
      <!-- Skeleton loading -->
      <div class="skeleton h-[150px]"></div>
      <div class="grid gap-px" style="grid-template-columns:repeat(auto-fit,minmax(min(160px,100%),1fr))">
        {#each [0, 1, 2, 3] as i}
          <div class="skeleton h-[190px]" style="animation-delay:{i * 60}ms"></div>
        {/each}
      </div>
      <div class="skeleton h-[420px]"></div>
    {/if}
  </div>
{/if}
