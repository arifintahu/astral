<script lang="ts">
  import { onMount } from 'svelte';
  import type { SystemMetrics, AlertEvent } from './lib/types';
  import TopBar from './lib/components/TopBar.svelte';
  import CpuCard from './lib/components/CpuCard.svelte';
  import MemoryCard from './lib/components/MemoryCard.svelte';
  import NetworkCard from './lib/components/NetworkCard.svelte';
  import DiskCard from './lib/components/DiskCard.svelte';
  import HistoryChart from './lib/components/HistoryChart.svelte';
  import ProcessList from './lib/components/ProcessList.svelte';
  import Toast from './lib/components/Toast.svelte';
  import Login from './lib/components/Login.svelte';

  let metrics: SystemMetrics | null = $state(null);
  let cpuHistory: number[] = $state([]);
  let txHistory: number[] = $state([]);
  let rxHistory: number[] = $state([]);
  let alerts: AlertEvent[] = $state([]);
  let authenticated = $state(false);
  let token = $state('');
  let refreshRate = $state(1);
  let lastUpdate = $state(0);

  let eventSource: EventSource | null = null;
  let alertSource: EventSource | null = null;

  function getToken(): string {
    return sessionStorage.getItem('astral_token') || '';
  }

  function handleLogin(newToken: string) {
    token = newToken;
    authenticated = true;
    connectStreams();
  }

  function handleLogout() {
    sessionStorage.removeItem('astral_token');
    sessionStorage.removeItem('astral_user');
    authenticated = false;
    token = '';
    metrics = null;
    cpuHistory = [];
    txHistory = [];
    rxHistory = [];
    if (eventSource) eventSource.close();
    if (alertSource) alertSource.close();
  }

  function handleRefreshRateChange(rate: number) {
    refreshRate = rate;
  }

  function dismissAlert(index: number) {
    alerts = alerts.filter((_, i) => i !== index);
  }

  // Auto-dismiss alerts after 10 seconds
  function scheduleAlertDismiss() {
    setTimeout(() => {
      if (alerts.length > 0) {
        alerts = alerts.slice(1);
      }
    }, 10000);
  }

  function connectStreams() {
    const t = getToken();

    // Metrics SSE — use polling via fetch with auth header since EventSource doesn't support custom headers
    // We'll use a polling approach with fetch for authenticated SSE
    const metricsUrl = '/api/stream';
    const alertsUrl = '/api/alerts';

    // For SSE with auth, we need to use fetch-based EventSource or pass token via query
    // Since standard EventSource doesn't support headers, we use a fetch-based approach
    startMetricsStream(t);
    startAlertStream(t);
  }

  function startMetricsStream(authToken: string) {
    // Use fetch with ReadableStream for SSE with auth headers
    const abortController = new AbortController();

    fetch('/api/stream', {
      headers: { 'Authorization': `Bearer ${authToken}` },
      signal: abortController.signal,
    }).then(response => {
      if (!response.ok) {
        if (response.status === 401) {
          handleLogout();
          return;
        }
        throw new Error(`HTTP ${response.status}`);
      }
      const reader = response.body!.getReader();
      const decoder = new TextDecoder();
      let buffer = '';

      function pump(): Promise<void> {
        return reader.read().then(({ done, value }) => {
          if (done) return;
          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split('\n');
          buffer = lines.pop() || '';

          for (const line of lines) {
            if (line.startsWith('data: ')) {
              const now = Date.now();
              if (now - lastUpdate < refreshRate * 1000 - 100) continue;
              lastUpdate = now;

              try {
                const data: SystemMetrics = JSON.parse(line.slice(6));
                metrics = data;

                if (metrics) {
                  cpuHistory = [...cpuHistory, metrics.cpu_usage].slice(-60);
                  txHistory = [...txHistory, metrics.network_tx].slice(-30);
                  rxHistory = [...rxHistory, metrics.network_rx].slice(-30);
                }
              } catch (e) {
                // ignore parse errors
              }
            }
          }
          return pump();
        });
      }
      pump();
    }).catch(e => {
      if (e.name !== 'AbortError') {
        console.error('Metrics stream error:', e);
        // Retry after 3s
        setTimeout(() => {
          if (authenticated) startMetricsStream(authToken);
        }, 3000);
      }
    });

    // Store abort controller for cleanup
    eventSource = { close: () => abortController.abort() } as any;
  }

  function startAlertStream(authToken: string) {
    const abortController = new AbortController();

    fetch('/api/alerts', {
      headers: { 'Authorization': `Bearer ${authToken}` },
      signal: abortController.signal,
    }).then(response => {
      if (!response.ok) return;
      const reader = response.body!.getReader();
      const decoder = new TextDecoder();
      let buffer = '';

      function pump(): Promise<void> {
        return reader.read().then(({ done, value }) => {
          if (done) return;
          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split('\n');
          buffer = lines.pop() || '';

          let nextIsAlertData = false;
          for (const line of lines) {
            if (line === 'event: alert') {
              nextIsAlertData = true;
            } else if (nextIsAlertData && line.startsWith('data: ')) {
              try {
                const alert: AlertEvent = JSON.parse(line.slice(6));
                alerts = [...alerts, alert].slice(-5);
                scheduleAlertDismiss();
              } catch (e) {}
              nextIsAlertData = false;
            }
          }
          return pump();
        });
      }
      pump();
    }).catch(e => {
      if (e.name !== 'AbortError') {
        setTimeout(() => {
          if (authenticated) startAlertStream(authToken);
        }, 5000);
      }
    });

    alertSource = { close: () => abortController.abort() } as any;
  }

  onMount(() => {
    // Check if we have a stored session
    const storedToken = sessionStorage.getItem('astral_token');
    if (storedToken) {
      // Validate the token
      fetch('/api/auth/check', {
        headers: { 'Authorization': `Bearer ${storedToken}` },
      }).then(res => {
        if (res.ok) {
          token = storedToken;
          authenticated = true;
          connectStreams();
        } else {
          sessionStorage.removeItem('astral_token');
          sessionStorage.removeItem('astral_user');
        }
      }).catch(() => {
        // If we can't reach server, show login
      });
    }

    return () => {
      if (eventSource) eventSource.close();
      if (alertSource) alertSource.close();
    };
  });
</script>

{#if !authenticated}
  <Login onLogin={handleLogin} />
{:else}
  <Toast {alerts} onDismiss={dismissAlert} />

  <div class="min-h-screen p-4 md:p-6 lg:p-8 max-w-screen-2xl mx-auto">
    <TopBar
      {metrics}
      {refreshRate}
      onRefreshRateChange={handleRefreshRateChange}
      onLogout={handleLogout}
    />

    {#if metrics}
      <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4 md:gap-5">
        <div class="animate-fade-in-up stagger-1 h-full">
          <CpuCard usage={metrics.cpu_usage} cores={metrics.cpu_cores} history={cpuHistory} />
        </div>
        <div class="animate-fade-in-up stagger-2 h-full">
          <MemoryCard
            used={metrics.used_memory}
            total={metrics.total_memory}
            swap_used={metrics.used_swap}
            swap_total={metrics.total_swap}
          />
        </div>
        <div class="animate-fade-in-up stagger-3 h-full">
          <NetworkCard tx={metrics.network_tx} rx={metrics.network_rx} {txHistory} {rxHistory} />
        </div>
        <div class="animate-fade-in-up stagger-4 h-full">
          <DiskCard disks={metrics.disks} />
        </div>
      </div>

      <div class="grid grid-cols-1 xl:grid-cols-3 gap-4 md:gap-5 mt-5">
        <div class="xl:col-span-2 animate-fade-in-up stagger-5 h-full">
          <HistoryChart {token} />
        </div>
        <div class="animate-fade-in-up h-full" style="animation-delay: 0.35s">
          <ProcessList processes={metrics.processes} totalMemory={metrics.total_memory} />
        </div>
      </div>
    {:else}
      <!-- Skeleton loading state -->
      <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4 md:gap-5">
        {#each [1, 2, 3, 4] as i}
          <div class="glass-card p-6 animate-fade-in" style="animation-delay: {i * 0.05}s">
            <div class="flex justify-between items-center mb-5">
              <div class="skeleton h-3 w-20"></div>
              <div class="skeleton h-5 w-16"></div>
            </div>
            <div class="flex items-end gap-4">
              <div class="skeleton h-12 w-24"></div>
              <div class="flex-1 skeleton h-8"></div>
            </div>
          </div>
        {/each}
      </div>
      <div class="grid grid-cols-1 xl:grid-cols-3 gap-4 md:gap-5 mt-5">
        <div class="xl:col-span-2 glass-panel p-6 animate-fade-in" style="animation-delay: 0.25s">
          <div class="flex justify-between items-center mb-5">
            <div class="skeleton h-3 w-16"></div>
            <div class="skeleton h-7 w-40"></div>
          </div>
          <div class="skeleton h-64 w-full"></div>
        </div>
        <div class="glass-panel p-6 animate-fade-in" style="animation-delay: 0.3s">
          <div class="flex justify-between items-center mb-4">
            <div class="skeleton h-3 w-24"></div>
            <div class="skeleton h-6 w-20"></div>
          </div>
          {#each [1, 2, 3, 4, 5] as i}
            <div class="skeleton h-6 w-full mb-2"></div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
{/if}
