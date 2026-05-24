// screens.jsx — Login, Settings panel, Alert history drawer, Loading skeleton

const { useState: useStateS } = React;

// ─── Login ──────────────────────────────────────────────────────────────────
function LoginScreen({ onLogin }) {
  const [username, setU] = useStateS("");
  const [password, setP] = useStateS("");
  const [error, setError] = useStateS("");
  const [loading, setLoading] = useStateS(false);

  function submit(e) {
    e.preventDefault();
    setError("");
    setLoading(true);
    setTimeout(() => {
      if (!username || !password) { setError("Invalid credentials"); setLoading(false); return; }
      setLoading(false);
      onLogin();
    }, 600);
  }

  return (
    <div style={{ minHeight: "100vh", display: "grid", gridTemplateColumns: "1fr 1fr", alignItems: "stretch" }}>
      {/* Left: form */}
      <div className="flex items-center justify-center p-12">
        <div className="w-full" style={{ maxWidth: 360 }}>
          <div className="mb-8 flex items-center gap-2.5">
            <Sigil />
            <div>
              <div className="text-[15px] font-semibold tracking-tight">Astral</div>
              <div className="text-[11px]" style={{ color: "var(--ink-3)" }}>Server monitoring</div>
            </div>
          </div>

          <h1 className="text-[28px] font-semibold tracking-tight" style={{ letterSpacing: "-0.02em" }}>
            Welcome back.
          </h1>
          <p className="mt-2 text-[13.5px]" style={{ color: "var(--ink-3)" }}>
            Sign in to view real-time metrics for your fleet.
          </p>

          <form onSubmit={submit} className="mt-8 flex flex-col gap-3">
            {error && (
              <div className="text-[12.5px] px-3 py-2 rounded-[10px] toast-in" style={{ color: "var(--crit)", background: "var(--crit-soft)", border: "1px solid rgba(244,63,94,.28)" }}>
                {error}
              </div>
            )}

            <Field label="Username">
              <input type="text" value={username} onChange={e => setU(e.target.value)} autoComplete="username" placeholder="admin"
                     className="focus-ring w-full text-[13px] font-mono"
                     style={{ background: "var(--bg-1)", border: "1px solid var(--line)", color: "var(--ink)", borderRadius: 10, padding: "11px 13px" }} />
            </Field>
            <Field label="Password">
              <input type="password" value={password} onChange={e => setP(e.target.value)} autoComplete="current-password" placeholder="••••••••••"
                     className="focus-ring w-full text-[13px] font-mono"
                     style={{ background: "var(--bg-1)", border: "1px solid var(--line)", color: "var(--ink)", borderRadius: 10, padding: "11px 13px" }} />
            </Field>

            <button type="submit" disabled={loading}
              className="text-[13px] font-medium mt-2"
              style={{
                background: "var(--accent)", color: "#0b0d12",
                borderRadius: 10, padding: "12px 14px",
                opacity: loading ? .8 : 1, cursor: "default"
              }}>
              {loading ? "Signing in…" : "Sign in →"}
            </button>

            <div className="mt-2 text-[11px]" style={{ color: "var(--ink-4)" }}>
              Astral does not terminate TLS · place behind your reverse proxy.
            </div>
          </form>
        </div>
      </div>

      {/* Right: ambient panel */}
      <div className="relative overflow-hidden" style={{ borderLeft: "1px solid var(--line)", background: "var(--bg-1)" }}>
        <AmbientCanvas />
        <div className="absolute inset-0 p-12 flex flex-col justify-end">
          <div className="eyebrow">Live · fra1</div>
          <div className="mt-2 text-[34px] font-semibold tracking-tight" style={{ letterSpacing: "-0.02em" }}>
            Real-time signal. <span style={{ color: "var(--ink-3)" }}>Quiet by design.</span>
          </div>
          <div className="mt-3 text-[13.5px] max-w-[420px]" style={{ color: "var(--ink-3)" }}>
            1-second SSE updates · HMAC sessions · Argon2 · webhook alerts.
            One binary, 7 MB, 9 MB RSS.
          </div>
          <div className="mt-8 flex flex-wrap gap-2">
            {["1s refresh", "12 cores", "edge-fra-01", "Debian 12", "18d 7h up"].map(t => (
              <span key={t} className="text-[11px] font-mono tnum px-2.5 py-1 rounded-[6px]"
                    style={{ background: "var(--bg-2)", border: "1px solid var(--line)", color: "var(--ink-2)" }}>{t}</span>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

function Field({ label, children }) {
  return (
    <label className="flex flex-col gap-1.5">
      <span className="eyebrow" style={{ fontSize: 10 }}>{label}</span>
      {children}
    </label>
  );
}

// Ambient SVG: a faint live mini-dashboard preview on the right side of login.
function AmbientCanvas() {
  // Simulated metric squiggles
  const pts1 = Array.from({ length: 60 }, (_, i) =>
    [i * 8, 80 + Math.sin(i * 0.32) * 22 + Math.sin(i * 0.81) * 8]);
  const pts2 = Array.from({ length: 60 }, (_, i) =>
    [i * 8, 140 + Math.sin(i * 0.21 + 1) * 18 + Math.cos(i * 0.5) * 6]);
  const d1 = `M ${pts1.map(p => p.join(",")).join(" L ")}`;
  const d2 = `M ${pts2.map(p => p.join(",")).join(" L ")}`;
  return (
    <svg width="100%" height="100%" viewBox="0 0 480 720" preserveAspectRatio="xMidYMid slice" style={{ position: "absolute", inset: 0 }}>
      <defs>
        <radialGradient id="amb-blob" cx="50%" cy="20%" r="60%">
          <stop offset="0%" stopColor="var(--accent)" stopOpacity="0.22" />
          <stop offset="100%" stopColor="var(--accent)" stopOpacity="0" />
        </radialGradient>
        <radialGradient id="amb-blob2" cx="80%" cy="80%" r="60%">
          <stop offset="0%" stopColor="var(--warm)" stopOpacity="0.16" />
          <stop offset="100%" stopColor="var(--warm)" stopOpacity="0" />
        </radialGradient>
      </defs>
      <rect width="100%" height="100%" fill="url(#amb-blob)" />
      <rect width="100%" height="100%" fill="url(#amb-blob2)" />
      {/* dotted grid */}
      <g opacity="0.5">
        {Array.from({ length: 24 }).map((_, i) =>
          Array.from({ length: 36 }).map((_, j) => (
            <circle key={`${i}-${j}`} cx={20 + j * 18} cy={20 + i * 18} r="0.8" fill="var(--line-2)" />
          ))
        )}
      </g>
      {/* curves */}
      <g transform="translate(20, 240)">
        <path d={d1} fill="none" stroke="var(--accent)" strokeWidth="1.4" opacity="0.95" />
        <path d={d2} fill="none" stroke="var(--warm)" strokeWidth="1.4" opacity="0.85" />
      </g>
      {/* floating chips */}
      <g transform="translate(40, 480)">
        <rect x="0" y="0" width="170" height="56" rx="10" fill="var(--bg-2)" stroke="var(--line-2)" />
        <text x="14" y="22" fontFamily="Geist" fontSize="10" fill="var(--ink-3)" letterSpacing="1">CPU</text>
        <text x="14" y="42" fontFamily="Geist" fontWeight="600" fontSize="20" fill="var(--ink)">42%</text>
        <circle cx="148" cy="32" r="3" fill="var(--accent)" className="live-dot" />
      </g>
      <g transform="translate(230, 510)">
        <rect x="0" y="0" width="170" height="56" rx="10" fill="var(--bg-2)" stroke="var(--line-2)" />
        <text x="14" y="22" fontFamily="Geist" fontSize="10" fill="var(--ink-3)" letterSpacing="1">MEM</text>
        <text x="14" y="42" fontFamily="Geist" fontWeight="600" fontSize="20" fill="var(--ink)">7.6 GB</text>
      </g>
    </svg>
  );
}

// ─── Settings Panel ─────────────────────────────────────────────────────────
function SettingsPanel({ open, onClose, settings, setSettings }) {
  if (!open) return null;
  function set(k, v) { setSettings(s => ({ ...s, [k]: v })); }

  return (
    <div style={{ position: "fixed", inset: 0, zIndex: 50 }}>
      <div className="backdrop-in" style={{ position: "absolute", inset: 0, background: "rgba(0,0,0,0.5)" }} onClick={onClose} />
      <div className="drawer-in surface" style={{ position: "absolute", top: 0, right: 0, height: "100vh", width: "min(440px, 100vw)", borderRadius: 0, borderTop: 0, borderRight: 0, borderBottom: 0, display: "flex", flexDirection: "column" }}>
        <div className="flex items-center justify-between" style={{ padding: "18px 22px", borderBottom: "1px solid var(--line)" }}>
          <div>
            <div className="eyebrow">Settings</div>
            <div className="text-[16px] font-semibold tracking-tight mt-0.5">Configure Astral</div>
          </div>
          <button className="btn btn-ghost" onClick={onClose}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>

        <div className="nice-scroll" style={{ flex: 1, overflowY: "auto", padding: 22, display: "flex", flexDirection: "column", gap: 22 }}>
          <Section title="General">
            <SegRow label="Refresh rate"
              options={[["1s", 1], ["2s", 2], ["5s", 5], ["10s", 10]]}
              value={settings.refreshRate}
              onChange={v => set("refreshRate", v)} />
            <ToggleRow label="Process monitoring" sub="Stream top processes with CPU/memory"
              value={settings.processList} onChange={v => set("processList", v)} />
          </Section>

          <Section title="Alerts">
            <SliderRow label="CPU threshold" value={settings.alertCpu} min={50} max={100} step={1} unit="%" onChange={v => set("alertCpu", v)} />
            <SliderRow label="Memory threshold" value={settings.alertRam} min={50} max={100} step={1} unit="%" onChange={v => set("alertRam", v)} />
            <InputRow label="Slack webhook" placeholder="https://hooks.slack.com/services/…"
              value={settings.webhook} onChange={v => set("webhook", v)} mono />
            <div className="text-[10.5px]" style={{ color: "var(--ink-4)" }}>
              HTTPS only · 15-minute cooldown per alert kind to prevent floods.
            </div>
          </Section>

          <Section title="Data">
            <SegRow label="Retention"
              options={[["7d", 7], ["30d", 30], ["90d", 90]]}
              value={settings.retention}
              onChange={v => set("retention", v)} />
            <ButtonRow label="Database" sub="SQLite · 12.4 MB · 412k samples"
              action="Export"
            />
          </Section>

          <Section title="Account">
            <ButtonRow label="Sessions" sub="2 active · last login 22m ago" action="Revoke all" danger />
            <ButtonRow label="Sign out" sub="Current device" action="Sign out" danger />
          </Section>
        </div>

        <div className="flex items-center justify-between" style={{ padding: "14px 22px", borderTop: "1px solid var(--line)" }}>
          <div className="text-[10.5px] font-mono tnum" style={{ color: "var(--ink-4)" }}>
            astral v1.1.0 · 7.1 MB · Rust 1.86
          </div>
          <button className="btn btn-primary" onClick={onClose}>Done</button>
        </div>
      </div>
    </div>
  );
}

function Section({ title, children }) {
  return (
    <div className="flex flex-col gap-3">
      <div className="eyebrow">{title}</div>
      <div className="flex flex-col gap-2.5">{children}</div>
    </div>
  );
}

function SegRow({ label, options, value, onChange }) {
  return (
    <div className="surface-2 flex items-center justify-between" style={{ padding: "10px 12px" }}>
      <span className="text-[12.5px]" style={{ color: "var(--ink-2)" }}>{label}</span>
      <div className="flex rounded-[7px]" style={{ background: "var(--bg)", border: "1px solid var(--line)", padding: 2 }}>
        {options.map(([lbl, v]) => (
          <button key={v} onClick={() => onChange(v)}
            className="text-[11px] font-medium font-mono tnum"
            style={{
              padding: "4px 9px", borderRadius: 5,
              background: value === v ? "var(--bg-1)" : "transparent",
              border: value === v ? "1px solid var(--line-2)" : "1px solid transparent",
              color: value === v ? "var(--ink)" : "var(--ink-3)",
              cursor: "default"
            }}>{lbl}</button>
        ))}
      </div>
    </div>
  );
}

function ToggleRow({ label, sub, value, onChange }) {
  return (
    <div className="surface-2 flex items-center justify-between gap-3" style={{ padding: "10px 12px" }}>
      <div>
        <div className="text-[12.5px]" style={{ color: "var(--ink-2)" }}>{label}</div>
        {sub && <div className="text-[10.5px]" style={{ color: "var(--ink-4)" }}>{sub}</div>}
      </div>
      <button onClick={() => onChange(!value)}
        style={{
          width: 34, height: 20, borderRadius: 999,
          background: value ? "var(--accent)" : "var(--line-2)",
          position: "relative", border: 0, cursor: "default",
          transition: "background .2s ease"
        }}>
        <span style={{
          position: "absolute", top: 2, left: value ? 16 : 2,
          width: 16, height: 16, borderRadius: 999, background: "var(--bg-1)",
          transition: "left .2s cubic-bezier(.2,.7,.2,1)"
        }} />
      </button>
    </div>
  );
}

function SliderRow({ label, value, min, max, step, unit, onChange }) {
  return (
    <div className="surface-2" style={{ padding: "10px 12px" }}>
      <div className="flex items-center justify-between mb-2">
        <span className="text-[12.5px]" style={{ color: "var(--ink-2)" }}>{label}</span>
        <span className="text-[12px] font-mono tnum" style={{ color: "var(--ink)" }}>{value}{unit}</span>
      </div>
      <input type="range" min={min} max={max} step={step} value={value}
        onChange={e => onChange(Number(e.target.value))}
        className="w-full"
        style={{ accentColor: "var(--accent)" }} />
    </div>
  );
}

function InputRow({ label, value, placeholder, onChange, mono }) {
  return (
    <div className="surface-2" style={{ padding: "10px 12px" }}>
      <div className="text-[12.5px] mb-1.5" style={{ color: "var(--ink-2)" }}>{label}</div>
      <input type="text" value={value} onChange={e => onChange(e.target.value)} placeholder={placeholder}
        className={"focus-ring w-full text-[12px] " + (mono ? "font-mono" : "")}
        style={{ background: "var(--bg)", border: "1px solid var(--line)", color: "var(--ink)", borderRadius: 7, padding: "8px 10px" }} />
    </div>
  );
}

function ButtonRow({ label, sub, action, danger }) {
  return (
    <div className="surface-2 flex items-center justify-between gap-3" style={{ padding: "10px 12px" }}>
      <div>
        <div className="text-[12.5px]" style={{ color: "var(--ink-2)" }}>{label}</div>
        {sub && <div className="text-[10.5px] font-mono tnum" style={{ color: "var(--ink-4)" }}>{sub}</div>}
      </div>
      <button className="btn" style={danger ? { color: "var(--crit)", borderColor: "var(--crit-soft)", background: "transparent" } : {}}>
        {action}
      </button>
    </div>
  );
}

// ─── Alert Drawer ───────────────────────────────────────────────────────────
function AlertDrawer({ open, onClose, alerts }) {
  if (!open) return null;
  const grouped = groupByDay(alerts);

  return (
    <div style={{ position: "fixed", inset: 0, zIndex: 50 }}>
      <div className="backdrop-in" style={{ position: "absolute", inset: 0, background: "rgba(0,0,0,0.5)" }} onClick={onClose} />
      <div className="drawer-in surface" style={{ position: "absolute", top: 0, right: 0, height: "100vh", width: "min(440px, 100vw)", borderRadius: 0, borderTop: 0, borderRight: 0, borderBottom: 0, display: "flex", flexDirection: "column" }}>
        <div className="flex items-center justify-between" style={{ padding: "18px 22px", borderBottom: "1px solid var(--line)" }}>
          <div>
            <div className="eyebrow">Alert history</div>
            <div className="text-[16px] font-semibold tracking-tight mt-0.5">
              {alerts.length} alert{alerts.length === 1 ? "" : "s"} · last 7 days
            </div>
          </div>
          <button className="btn btn-ghost" onClick={onClose}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
          </button>
        </div>

        <div className="nice-scroll" style={{ flex: 1, overflowY: "auto", padding: "8px 22px 22px" }}>
          {grouped.map(([day, list], gi) => (
            <div key={gi} className="mt-4">
              <div className="eyebrow mb-2">{day}</div>
              <div style={{ position: "relative" }}>
                {/* vertical timeline rail */}
                <div style={{ position: "absolute", left: 7, top: 6, bottom: 6, width: 1, background: "var(--line)" }} />
                {list.map((a, i) => (
                  <div key={i} className="flex gap-3 py-2.5">
                    <div style={{ position: "relative", width: 16, flexShrink: 0 }}>
                      <span style={{
                        position: "absolute", left: 3, top: 5, width: 9, height: 9, borderRadius: 999,
                        background: a.kind === "cpu" ? "var(--accent)" : "var(--warm)",
                        border: "2px solid var(--bg-1)", boxShadow: "0 0 0 1px var(--line-2)"
                      }} />
                    </div>
                    <div className="surface-2 flex-1" style={{ padding: "10px 12px" }}>
                      <div className="flex items-center justify-between mb-1">
                        <span className="text-[11px] eyebrow" style={{ color: a.kind === "cpu" ? "var(--accent)" : "var(--warm)" }}>
                          {a.kind === "cpu" ? "CPU" : "Memory"}
                        </span>
                        <span className="text-[10.5px] font-mono tnum" style={{ color: "var(--ink-4)" }}>
                          {formatRelative(a.timestamp)}
                        </span>
                      </div>
                      <div className="text-[12.5px]" style={{ color: "var(--ink-2)" }}>{a.message}</div>
                      <div className="mt-1.5 flex items-center gap-2 text-[11px] font-mono tnum" style={{ color: "var(--ink-3)" }}>
                        <span>peak <span style={{ color: a.value >= a.threshold ? "var(--crit)" : "var(--ink-2)" }}>{a.value.toFixed(1)}%</span></span>
                        <span style={{ color: "var(--ink-4)" }}>·</span>
                        <span>threshold {a.threshold}%</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
          {!alerts.length && (
            <div className="flex flex-col items-center justify-center text-center py-16 gap-3">
              <div className="surface-2 flex items-center justify-center" style={{ width: 44, height: 44, borderRadius: 12 }}>
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="var(--ok)" strokeWidth="1.6">
                  <polyline points="20 6 9 17 4 12" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              </div>
              <div className="text-[13px]" style={{ color: "var(--ink-2)" }}>No alerts in the last 7 days</div>
              <div className="text-[11px]" style={{ color: "var(--ink-4)" }}>All quiet on edge-fra-01.</div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function groupByDay(alerts) {
  const map = new Map();
  for (const a of alerts) {
    const d = new Date(a.timestamp);
    const today = new Date(); today.setHours(0,0,0,0);
    const yesterday = new Date(today); yesterday.setDate(today.getDate() - 1);
    const aDay = new Date(d); aDay.setHours(0,0,0,0);
    let key;
    if (aDay.getTime() === today.getTime()) key = "Today";
    else if (aDay.getTime() === yesterday.getTime()) key = "Yesterday";
    else key = aDay.toLocaleDateString(undefined, { weekday: "short", month: "short", day: "numeric" });
    if (!map.has(key)) map.set(key, []);
    map.get(key).push(a);
  }
  return Array.from(map.entries());
}

// ─── Skeleton loading ───────────────────────────────────────────────────────
function DashboardSkeleton() {
  return (
    <div className="fade-up">
      <div className="grid gap-4 md:gap-5" style={{ gridTemplateColumns: "repeat(4, 1fr)" }}>
        {[1, 2, 3, 4].map(i => (
          <div key={i} className="surface" style={{ padding: 20, minHeight: 156 }}>
            <div className="flex items-center justify-between mb-4">
              <div className="skeleton" style={{ width: 60, height: 10 }} />
              <div className="skeleton" style={{ width: 40, height: 10 }} />
            </div>
            <div className="flex items-center gap-3">
              <div className="skeleton" style={{ width: 56, height: 56, borderRadius: 999 }} />
              <div className="flex-1 skeleton" style={{ height: 44 }} />
            </div>
          </div>
        ))}
      </div>
      <div className="grid gap-4 md:gap-5 mt-5" style={{ gridTemplateColumns: "2fr 1fr" }}>
        <div className="surface" style={{ padding: 22, minHeight: 280 }}>
          <div className="flex items-center justify-between mb-4">
            <div className="skeleton" style={{ width: 200, height: 12 }} />
            <div className="skeleton" style={{ width: 160, height: 22 }} />
          </div>
          <div className="skeleton" style={{ height: 220 }} />
        </div>
        <div className="surface" style={{ padding: 22 }}>
          <div className="skeleton mb-4" style={{ width: 120, height: 12 }} />
          {[1,2,3,4,5,6].map(i => (
            <div key={i} className="skeleton mt-2" style={{ height: 14 }} />
          ))}
        </div>
      </div>
    </div>
  );
}

Object.assign(window, { LoginScreen, SettingsPanel, AlertDrawer, DashboardSkeleton });
