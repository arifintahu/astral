// app.jsx — Root: state, screen routing, tweaks, demo controls

const { useState: useStateA, useEffect: useEffectA, useMemo: useMemoA, useRef: useRefA } = React;

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "accent": "#818cf8",
  "dark": true,
  "density": "regular",
  "chartStyle": "sparkline",
  "showProcesses": true,
  "demoState": "live"
}/*EDITMODE-END*/;

const ACCENT_OPTIONS = [
  "#818cf8", // indigo (default)
  "#22d3ee", // cyan
  "#34d399", // emerald
  "#f472b6", // pink
];

function App() {
  const [t, setTweak] = useTweaks(TWEAK_DEFAULTS);

  // Apply tweaks → CSS vars
  useEffectA(() => {
    const r = document.documentElement;
    if (t.dark) r.classList.remove("is-light");
    else r.classList.add("is-light");
  }, [t.dark]);

  useEffectA(() => {
    const r = document.documentElement;
    r.style.setProperty("--accent", t.accent);
    const hex = t.accent.replace("#", "");
    const num = parseInt(hex, 16);
    const rC = (num >> 16) & 255, gC = (num >> 8) & 255, bC = num & 255;
    r.style.setProperty("--accent-soft", `rgba(${rC},${gC},${bC},0.16)`);
    r.style.setProperty("--accent-line", `rgba(${rC},${gC},${bC},0.4)`);
  }, [t.accent]);

  // ── Screen state ─────────────────────────────────────────────────────────
  const [authed, setAuthed] = useStateA(true);
  const [showSettings, setShowSettings] = useStateA(false);
  const [showAlerts, setShowAlerts] = useStateA(false);

  const [settings, setSettings] = useStateA({
    refreshRate: 1, processList: true,
    alertCpu: 90, alertRam: 90,
    webhook: "https://hooks.slack.com/services/T0XXX/B0YYY/zzz",
    retention: 30,
  });

  // ── Mock live data ───────────────────────────────────────────────────────
  const [metrics, setMetrics] = useStateA(null);
  const [cpuHist, setCpuHist] = useStateA([]);
  const [txHist, setTxHist] = useStateA([]);
  const [rxHist, setRxHist] = useStateA([]);
  const [alerts, setAlerts] = useStateA([]);
  const [history, setHistory] = useStateA([]);
  const [range, setRange] = useStateA("6h");
  const [metricKey, setMetricKey] = useStateA("cpu");
  const prevRef = useRefA(null);
  const tickRef = useRefA(0);

  // Initial seed (different lengths by range)
  useEffectA(() => {
    const points = range === "6h" ? 180 : range === "24h" ? 288 : range === "7d" ? 168 : 220;
    setHistory(window.AstralMock.buildHistory(points, range === "7d" ? 14 : 10));
  }, [range]);

  // Demo: choose state ─────────────────────────────────────────────────────
  // "live" = streaming  | "loading" = skeleton | "logged-out" = login | "alert" = trigger alert
  useEffectA(() => {
    if (t.demoState === "logged-out") { setAuthed(false); return; }
    setAuthed(true);

    if (t.demoState === "loading") {
      setMetrics(null);
      return;
    }

    // Live or alert — setInterval keeps running in hidden tabs / iframes
    function tick() {
      const now = Date.now();
      const m = window.AstralMock.gen(prevRef.current);
      // In "alert" mode, force high CPU & memory
      if (t.demoState === "alert") {
        m.cpu_usage = 92 + Math.random() * 6;
        m.used_memory = m.total_memory * 0.91;
      }
      prevRef.current = m;
      setMetrics(m);
      setCpuHist(h => [...h, m.cpu_usage].slice(-60));
      setTxHist(h => [...h, m.network_tx].slice(-30));
      setRxHist(h => [...h, m.network_rx].slice(-30));
      // Also append to history feed (so the chart's leading dot moves)
      setHistory(hh => {
        if (!hh.length) return hh;
        return [...hh.slice(-(hh.length - 1)), {
          timestamp: now,
          cpu_usage: m.cpu_usage,
          used_memory: m.used_memory,
          network_tx: m.network_tx,
          network_rx: m.network_rx,
          disk_read_rate: m.disks[0].read_bytes,
          disk_write_rate: m.disks[0].written_bytes
        }];
      });
      tickRef.current++;
    }
    // Prime once synchronously so the dashboard never shows the skeleton on mount
    tick();
    // refreshRate is in seconds between ticks (1s, 2s, 5s, 10s)
    const interval = Math.max(250, settings.refreshRate * 1000);
    const id = setInterval(tick, interval);
    return () => clearInterval(id);
  }, [t.demoState, settings.refreshRate]);

  // Demo: maintain alert list when in "alert" state ────────────────────────
  useEffectA(() => {
    if (t.demoState !== "alert") { setAlerts([]); return; }
    const a = {
      kind: "cpu",
      message: "CPU usage sustained above 90% for 5m",
      value: 94.6, threshold: settings.alertCpu, timestamp: Date.now()
    };
    setAlerts([a]);
  }, [t.demoState, settings.alertCpu]);

  // Historical alerts (drawer)
  const historicalAlerts = useMemoA(() => window.AstralMock.buildAlerts(), []);

  function dismissToast(i) { setAlerts(arr => arr.filter((_, j) => j !== i)); }

  // ── Render ───────────────────────────────────────────────────────────────
  return (
    <>
      {!authed && <LoginScreen onLogin={() => setAuthed(true)} />}

      {authed && (
        <div style={{ maxWidth: 1440, margin: "0 auto", padding: t.density === "compact" ? "20px 22px" : t.density === "comfy" ? "32px 32px" : "26px 28px" }}>
          <Toast alerts={alerts} onDismiss={dismissToast} />

          <TopBar
            metrics={metrics}
            alerts={alerts}
            refreshRate={settings.refreshRate}
            onShowAlerts={() => setShowAlerts(true)}
            onShowSettings={() => setShowSettings(true)}
            onLogout={() => setTweak("demoState", "logged-out")}
          />

          {!metrics ? (
            <DashboardSkeleton />
          ) : (
            <>
              <div className="grid gap-4 md:gap-5"
                   style={{ gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))" }}>
                <div className="fade-up" style={{ animationDelay: "0ms" }}>
                  <CpuCard metrics={metrics} history={cpuHist} chartStyle={t.chartStyle} density={t.density} />
                </div>
                <div className="fade-up" style={{ animationDelay: "40ms" }}>
                  <MemoryCard metrics={metrics} chartStyle={t.chartStyle} density={t.density} />
                </div>
                <div className="fade-up" style={{ animationDelay: "80ms" }}>
                  <NetworkCard metrics={metrics} txHistory={txHist} rxHistory={rxHist} chartStyle={t.chartStyle} density={t.density} />
                </div>
                <div className="fade-up" style={{ animationDelay: "120ms" }}>
                  <StorageCard metrics={metrics} density={t.density} />
                </div>
              </div>

              <div className="grid gap-4 md:gap-5 mt-5"
                   style={{ gridTemplateColumns: t.showProcesses ? "minmax(0,2fr) minmax(0,1fr)" : "minmax(0,1fr)" }}>
                <div className="fade-up" style={{ animationDelay: "160ms" }}>
                  <HistoryChart range={range} setRange={setRange} metric={metricKey} setMetric={setMetricKey} history={history} live />
                </div>
                {t.showProcesses && (
                  <div className="fade-up" style={{ animationDelay: "200ms" }}>
                    <ProcessList processes={metrics.processes} totalMemory={metrics.total_memory} enabled={settings.processList} />
                  </div>
                )}
              </div>

              {/* Footer hairline */}
              <div className="mt-6 flex items-center justify-between text-[10.5px] font-mono tnum" style={{ color: "var(--ink-4)" }}>
                <span>edge-fra-01 · {settings.processList ? "processes on" : "processes off"} · {settings.retention}d retention</span>
                <span>updated {new Date().toLocaleTimeString()} · {settings.refreshRate}s</span>
              </div>
            </>
          )}
        </div>
      )}

      <SettingsPanel open={showSettings} onClose={() => setShowSettings(false)} settings={settings} setSettings={setSettings} />
      <AlertDrawer open={showAlerts} onClose={() => setShowAlerts(false)} alerts={historicalAlerts} />

      <TweaksPanel>
        <TweakSection label="Theme" />
        <TweakToggle label="Dark mode" value={t.dark} onChange={v => setTweak("dark", v)} />
        <TweakColor label="Accent" value={t.accent}
          options={ACCENT_OPTIONS}
          onChange={v => setTweak("accent", v)} />

        <TweakSection label="Layout" />
        <TweakRadio label="Density" value={t.density}
          options={["compact", "regular", "comfy"]}
          onChange={v => setTweak("density", v)} />
        <TweakToggle label="Show processes" value={t.showProcesses}
          onChange={v => setTweak("showProcesses", v)} />

        <TweakSection label="Card chart style" />
        <TweakRadio label="Variant" value={t.chartStyle}
          options={["sparkline", "bar", "dot", "radial"]}
          onChange={v => setTweak("chartStyle", v)} />

        <TweakSection label="Demo state" />
        <TweakSelect label="State" value={t.demoState}
          options={[
            { value: "live", label: "Live streaming" },
            { value: "alert", label: "Active alert" },
            { value: "loading", label: "Loading skeleton" },
            { value: "logged-out", label: "Login screen" },
          ]}
          onChange={v => setTweak("demoState", v)} />
      </TweaksPanel>
    </>
  );
}

const root = ReactDOM.createRoot(document.getElementById("root"));
root.render(<App />);
