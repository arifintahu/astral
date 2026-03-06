<script lang="ts">
  let { onLogin }: { onLogin: (token: string) => void } = $props();

  let username = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = '';
    loading = true;

    try {
      const res = await fetch('/api/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      });

      if (res.ok) {
        const data = await res.json();
        sessionStorage.setItem('astral_token', data.token);
        sessionStorage.setItem('astral_user', data.username);
        onLogin(data.token);
      } else {
        error = 'Invalid credentials';
      }
    } catch (e) {
      error = 'Connection failed';
    } finally {
      loading = false;
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center p-4">
  <div class="w-full max-w-sm animate-fade-in-up">
    <!-- Logo -->
    <div class="text-center mb-8">
      <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-gradient-to-br from-purple-500/20 to-cyan-500/20 border border-white/[0.08] mb-4">
        <svg class="w-8 h-8 text-cyan-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="3" />
          <path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4M4.22 19.78l2.83-2.83M16.95 7.05l2.83-2.83" stroke-linecap="round"/>
        </svg>
      </div>
      <h1 class="text-2xl font-bold">
        <span class="bg-clip-text text-transparent bg-gradient-to-r from-cyan-300 via-blue-400 to-purple-500">Astral</span>
      </h1>
      <p class="text-sm text-slate-500 mt-1">Sign in to your dashboard</p>
    </div>

    <form onsubmit={handleSubmit} class="glass-panel p-6 space-y-4">
      {#if error}
        <div class="text-sm text-rose-400 bg-rose-500/10 border border-rose-500/20 rounded-lg px-3 py-2 animate-fade-in">
          {error}
        </div>
      {/if}

      <div>
        <label for="username" class="block text-[11px] text-slate-500 font-semibold uppercase tracking-[0.15em] mb-2">Username</label>
        <input
          id="username"
          type="text"
          bind:value={username}
          autocomplete="username"
          required
          class="w-full bg-white/[0.04] border border-white/[0.08] rounded-xl px-4 py-3 text-sm text-white placeholder-slate-600 focus:outline-none focus:border-cyan-500/30 focus:ring-1 focus:ring-cyan-500/20 transition-colors"
          placeholder="Enter username"
        />
      </div>

      <div>
        <label for="password" class="block text-[11px] text-slate-500 font-semibold uppercase tracking-[0.15em] mb-2">Password</label>
        <input
          id="password"
          type="password"
          bind:value={password}
          autocomplete="current-password"
          required
          class="w-full bg-white/[0.04] border border-white/[0.08] rounded-xl px-4 py-3 text-sm text-white placeholder-slate-600 focus:outline-none focus:border-cyan-500/30 focus:ring-1 focus:ring-cyan-500/20 transition-colors"
          placeholder="Enter password"
        />
      </div>

      <button
        type="submit"
        disabled={loading}
        class="w-full bg-gradient-to-r from-cyan-500/20 to-purple-500/20 hover:from-cyan-500/30 hover:to-purple-500/30 border border-white/[0.1] hover:border-white/[0.15] rounded-xl px-4 py-3 text-sm font-semibold text-white transition-all duration-200 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {#if loading}
          <span class="inline-flex items-center gap-2">
            <span class="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin"></span>
            Signing in...
          </span>
        {:else}
          Sign in
        {/if}
      </button>
    </form>
  </div>
</div>
