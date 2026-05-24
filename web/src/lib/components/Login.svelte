<script lang="ts">
  let { onLogin }: { onLogin: () => void } = $props();

  let username = $state('');
  let password = $state('');
  let error    = $state('');
  let loading  = $state(false);

  async function submit(e: Event) {
    e.preventDefault();
    error = ''; loading = true;
    try {
      const res = await fetch('/api/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      });
      if (res.ok) onLogin();
      else error = 'Invalid credentials';
    } catch { error = 'Connection failed'; }
    finally { loading = false; }
  }
</script>

<div style="min-height:100vh;display:grid;grid-template-columns:1fr 1fr;background:var(--bg)">
  <!-- Left: form panel -->
  <div class="flex items-center justify-center" style="padding:48px">
    <div style="width:100%;max-width:360px" class="anim-fade-up">
      <!-- Sigil + name -->
      <div class="flex items-center gap-3 mb-10">
        <div class="flex items-center justify-center flex-shrink-0"
             style="width:40px;height:40px;border-radius:10px;background:var(--bg-2);border:1px solid var(--line-2)">
          <svg width="22" height="22" viewBox="0 0 20 20" fill="none">
            <circle cx="10" cy="10" r="2.5"  stroke="var(--accent)" stroke-width="1.5"/>
            <circle cx="10" cy="10" r="6"    stroke="var(--accent)" stroke-width="1"    stroke-opacity="0.5"/>
            <circle cx="10" cy="10" r="9"    stroke="var(--accent)" stroke-width="0.75" stroke-opacity="0.25"/>
          </svg>
        </div>
        <span style="font-size:18px;font-weight:600;color:var(--ink)">Astral</span>
      </div>

      <h1 style="font-size:26px;font-weight:600;color:var(--ink);margin:0 0 6px">Welcome back.</h1>
      <p style="font-size:13px;color:var(--ink-3);margin:0 0 32px">Sign in to your monitoring dashboard</p>

      <form onsubmit={submit} style="display:flex;flex-direction:column;gap:16px">
        {#if error}
          <div style="font-size:12px;color:var(--crit);background:var(--crit-soft);border:1px solid rgba(244,63,94,0.25);border-radius:8px;padding:8px 12px">
            {error}
          </div>
        {/if}

        <div>
          <label for="username" class="eyebrow" style="display:block;margin-bottom:6px">Username</label>
          <input id="username" type="text" bind:value={username}
                 autocomplete="username" required class="focus-ring"
                 placeholder="admin"
                 style="width:100%;box-sizing:border-box;background:var(--bg-2);border:1px solid var(--line);border-radius:8px;padding:10px 14px;font-size:13px;color:var(--ink);outline:none;transition:border-color 0.15s;font-family:inherit"
                 onfocus={e => (e.target as HTMLInputElement).style.borderColor = 'var(--accent-line)'}
                 onblur={e => (e.target as HTMLInputElement).style.borderColor = 'var(--line)'}/>
        </div>

        <div>
          <label for="password" class="eyebrow" style="display:block;margin-bottom:6px">Password</label>
          <input id="password" type="password" bind:value={password}
                 autocomplete="current-password" required class="focus-ring"
                 placeholder="••••••••"
                 style="width:100%;box-sizing:border-box;background:var(--bg-2);border:1px solid var(--line);border-radius:8px;padding:10px 14px;font-size:13px;color:var(--ink);outline:none;transition:border-color 0.15s;font-family:inherit"
                 onfocus={e => (e.target as HTMLInputElement).style.borderColor = 'var(--accent-line)'}
                 onblur={e => (e.target as HTMLInputElement).style.borderColor = 'var(--line)'}/>
        </div>

        <button type="submit" disabled={loading}
                style="width:100%;background:var(--accent);color:#0b0d12;border:none;border-radius:8px;padding:11px;font-size:13px;font-weight:600;cursor:pointer;transition:background 0.15s;margin-top:4px;font-family:inherit"
                onmouseover={e => { if (!loading) (e.currentTarget as HTMLElement).style.background = '#7dd3fc'; }}
                onmouseout={e => (e.currentTarget as HTMLElement).style.background = loading ? 'var(--accent)' : 'var(--accent)'}
                onfocus={e => { if (!loading) (e.currentTarget as HTMLElement).style.background = '#7dd3fc'; }}
                onblur={e => (e.currentTarget as HTMLElement).style.background = 'var(--accent)'}>
          {loading ? 'Signing in…' : 'Sign in'}
        </button>
      </form>

      <p style="font-size:11px;color:var(--ink-4);margin-top:24px;text-align:center">
        Connection encrypted · session expires in 24h
      </p>
    </div>
  </div>

  <!-- Right: ambient panel -->
  <div class="relative overflow-hidden" style="background:var(--bg-1);border-left:1px solid var(--line)">
    <svg class="absolute inset-0" width="100%" height="100%"
         viewBox="0 0 600 900" preserveAspectRatio="xMidYMid slice" style="display:block">
      <defs>
        <pattern id="lgrid" x="0" y="0" width="20" height="20" patternUnits="userSpaceOnUse">
          <circle cx="1" cy="1" r="0.8" fill="var(--line)"/>
        </pattern>
      </defs>
      <!-- Dot grid -->
      <rect width="600" height="900" fill="url(#lgrid)"/>
      <!-- Gradient blobs -->
      <ellipse cx="450" cy="220" rx="280" ry="280" fill="rgba(56,189,248,0.07)"/>
      <ellipse cx="120" cy="700" rx="250" ry="250" fill="rgba(245,158,11,0.05)"/>
      <!-- Wave 1 — accent, slow scroll -->
      <g style="animation:waveScroll 10s linear infinite">
        <path d="M-600,300 Q-450,275 -300,300 Q-150,325 0,300 Q150,275 300,300 Q450,325 600,300 Q750,275 900,300 Q1050,325 1200,300"
              fill="none" stroke="rgba(56,189,248,0.2)" stroke-width="1.5"/>
      </g>
      <!-- Wave 2 — warm, faster reverse -->
      <g style="animation:waveScroll 14s linear infinite reverse">
        <path d="M-600,420 Q-450,400 -300,420 Q-150,440 0,420 Q150,400 300,420 Q450,440 600,420 Q750,400 900,420 Q1050,440 1200,420"
              fill="none" stroke="rgba(245,158,11,0.14)" stroke-width="1"/>
      </g>
    </svg>

    <!-- Floating metric chips -->
    <div class="absolute inset-0 flex flex-col items-center justify-center gap-3">
      <div class="surface tnum font-mono" style="padding:10px 20px;font-size:13px;color:var(--accent)">CPU 24.3%</div>
      <div class="surface tnum font-mono" style="padding:10px 20px;font-size:13px;color:var(--warm)">MEM 6.1 GB</div>
    </div>
  </div>
</div>
