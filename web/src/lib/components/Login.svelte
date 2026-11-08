<script lang="ts">
  let { onLogin }: { onLogin: () => void } = $props();

  let username = $state('');
  let password = $state('');
  let showPw   = $state(false);
  let error    = $state('');
  let loading  = $state(false);

  const LIT = 17;
  const TICKS = Array.from({ length: 28 }, (_, j) => j < LIT);
  const CURVE = 'M-20,560 C40,560 70,520 110,540 S170,610 215,560 S270,330 315,410 S375,620 430,540 S520,470 620,500';

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

<div class="flex min-h-screen flex-wrap" style="background:var(--bg)">
  <!-- Form side -->
  <main class="box-border flex min-w-0 flex-[1_1_440px] items-center justify-center p-[clamp(24px,5vw,64px)]">
    <div class="anim-fade-up flex w-full max-w-[380px] flex-col gap-8">
      <div class="flex items-center gap-3">
        <svg width="32" height="32" viewBox="0 0 28 28" fill="none" aria-hidden="true">
          <circle cx="14" cy="14" r="3.2" fill="var(--accent)" />
          <circle cx="14" cy="14" r="8.5" stroke="var(--accent)" stroke-opacity=".45" stroke-width="1.2" />
          <g class="orbit"><circle cx="22.5" cy="14" r="2" fill="var(--accent)" /></g>
        </svg>
        <span class="text-[20px] font-semibold tracking-[-0.02em]">Astral</span>
      </div>

      <div class="flex flex-col gap-2">
        <h1 class="m-0 text-[clamp(30px,4vw,40px)] font-medium leading-[1.05] tracking-[-0.04em]">Welcome back.</h1>
        <p class="m-0 text-[16px] leading-normal" style="color:var(--ink-2)">Sign in to your monitoring dashboard.</p>
      </div>

      <form onsubmit={submit} class="flex flex-col gap-[18px]">
        {#if error}
          <div role="alert" class="flex items-center gap-2.5 rounded-xl border px-3.5 py-3 text-[14px]"
               style="color:var(--crit);background:color-mix(in srgb, var(--crit) 12%, transparent);border-color:color-mix(in srgb, var(--crit) 40%, transparent)">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" />
            </svg>
            {error}
          </div>
        {/if}

        <div class="flex flex-col gap-2">
          <label for="username" class="eyebrow">Username</label>
          <input id="username" type="text" bind:value={username} autocomplete="username" required
                 placeholder="admin" class="field h-[52px] px-4" />
        </div>

        <div class="flex flex-col gap-2">
          <label for="password" class="eyebrow">Password</label>
          <div class="relative">
            <input id="password" type={showPw ? 'text' : 'password'} value={password}
                   oninput={(e) => (password = (e.target as HTMLInputElement).value)}
                   autocomplete="current-password" required placeholder="••••••••"
                   class="field h-[52px] pl-4 pr-[76px]" />
            <button type="button" aria-pressed={showPw} onclick={() => (showPw = !showPw)}
                    class="absolute right-1 top-1 h-11 cursor-pointer rounded-[10px] border-0 bg-transparent px-3.5 text-[13px] font-medium"
                    style="color:var(--ink-3);font-family:inherit">{showPw ? 'Hide' : 'Show'}</button>
          </div>
        </div>

        <button type="submit" disabled={loading} class="cta mt-1.5">
          {#if loading}
            <span class="size-4 rounded-full border-2" style="border-color:rgba(9,9,11,0.3);border-top-color:#09090b;animation:spin 0.8s linear infinite"></span>
          {/if}
          {loading ? 'Signing in…' : 'Sign in'}
        </button>
      </form>

      <p class="font-mono m-0 text-center text-[12px]" style="color:var(--ink-3)">
        Connection encrypted · session expires in 24h
      </p>
    </div>
  </main>

  <!-- Signal side -->
  <aside aria-hidden="true" class="relative min-h-[320px] min-w-0 flex-[1.15_1_520px] overflow-hidden border-l"
         style="background:var(--bg-1);border-color:var(--line)">
    <svg viewBox="0 0 600 900" preserveAspectRatio="xMidYMid slice" class="absolute inset-0 block h-full w-full">
      <defs>
        <pattern id="login-dots" x="0" y="0" width="22" height="22" patternUnits="userSpaceOnUse">
          <circle cx="1" cy="1" r="1" fill="rgba(255,255,255,.07)" />
        </pattern>
        <linearGradient id="login-area" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" style="stop-color:var(--accent);stop-opacity:.16" />
          <stop offset="1" style="stop-color:var(--accent);stop-opacity:0" />
        </linearGradient>
      </defs>
      <rect width="600" height="900" fill="url(#login-dots)" />
      <path d="{CURVE} L620,900 L-20,900 Z" fill="url(#login-area)" />
      <path d={CURVE} fill="none" stroke="rgba(255,255,255,.14)" stroke-width="2" stroke-linecap="round" />
      <path class="trace" pathLength="1000" d={CURVE} fill="none" stroke="var(--accent)" stroke-width="3" stroke-linecap="round"
            style="filter:drop-shadow(0 0 8px color-mix(in srgb, var(--accent) 60%, transparent))" />
    </svg>

    <div class="absolute inset-x-[clamp(24px,5vw,56px)] bottom-[clamp(24px,5vw,56px)] flex flex-col gap-[18px]">
      <div class="flex h-3.5 max-w-[340px] gap-[3px]">
        {#each TICKS as lit}
          <span class="flex-1 rounded-[1.5px]" style="background:{lit ? 'var(--accent)' : 'rgba(255,255,255,0.09)'}"></span>
        {/each}
      </div>
      <span class="max-w-[15ch] text-balance text-[clamp(24px,3vw,34px)] font-medium leading-[1.15] tracking-[-0.03em]">
        Ultra-lightweight server monitoring.
      </span>
      <span class="max-w-[36ch] text-[15px] leading-normal" style="color:var(--ink-2)">
        One small binary. Live and historical CPU, memory, network and disk for your VM.
      </span>
    </div>
  </aside>
</div>
