# Security Review — Astral VM Monitoring Dashboard

**Date:** 2026-04-22  
**Reviewer:** Senior Security Engineer  
**Scope:** Full codebase — `src/` (Rust backend) and `web/src/` (Svelte frontend)  
**Branch:** `claude/security-review-vm-metrics-udago`

---

## Executive Summary

Astral is a single-binary VM monitoring dashboard exposing real-time and historical system metrics via HTTP. The codebase demonstrates good practices in several areas (Argon2 password hashing, table-name allowlist for SQL, HMAC-based session tokens, login rate limiting), but contains two critical issues that allow credential theft without any exploitation sophistication, plus several high-severity gaps in session management and request validation.

**Finding counts:** 2 Critical · 4 High · 5 Medium · 4 Low

---

## Critical

---

### C-1: Credentials Exposed via CLI Arguments

**File:** `src/main.rs:107`

The `--auth USER:PASS` flag accepts the admin password as a plain command-line argument. This exposes credentials in:

- `ps aux` / `/proc/*/cmdline` — readable by any local user on the host
- Shell history files (`~/.bash_history`, `~/.zsh_history`)
- System audit logs, Docker logs, systemd journal (`journalctl`)
- CI/CD pipeline logs if the flag appears in scripts or `docker run` commands

Any process-level read access on the host silently leaks the admin password with no indication to the operator.

```rust
// src/main.rs:107 — password visible in process argument list
let parts: Vec<&str> = auth_str.split(':').collect();
```

**Recommendation:** Accept credentials via environment variables or a restricted config file (mode `0600`). Never accept secrets as flag arguments. Example:

```rust
#[arg(long, env = "ASTRAL_AUTH")]
auth: Option<String>,
```

---

### C-2: No TLS — Credentials and Session Tokens Transmitted in Plaintext

**File:** `src/main.rs:219-223`

The server binds exclusively on plain HTTP (`0.0.0.0:8080`) with no TLS support. Every login request (username + password JSON body) and every subsequent `Authorization: Bearer <token>` header is transmitted unencrypted. A passive network observer (same LAN, rogue Wi-Fi, cloud VPC tap) can capture credentials and replay the 24-hour session token with no active attack required.

```rust
// src/main.rs:219 — plain TCP, no TLS
let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
let listener = tokio::net::TcpListener::bind(addr).await?;
```

**Recommendation:** Require TLS termination at a reverse proxy (nginx, Caddy, Traefik) and document this as a hard deployment prerequisite. Optionally, enforce it at the application layer by rejecting requests where `X-Forwarded-Proto: http` is present.

---

## High

---

### H-1: Session Tokens Contain No Random Nonce — Identical Tokens Possible

**File:** `src/main.rs:228-237`

The HMAC payload is deterministic: `"astral:{username}:session:{unix_secs}"`. Two logins within the same second produce identical tokens. There is no random component.

```rust
let payload = format!("astral:{}:session:{}", username, now);
// now = seconds granularity — no nonce
```

This means multiple browser sessions opened simultaneously share one token. If one session is stolen, there is no way to distinguish which session to invalidate.

**Recommendation:** Include a cryptographically random nonce in the payload and embed it in the token so the verifier can reconstruct it:

```rust
let nonce = hex::encode(&OsRng.gen::<[u8; 16]>());
let payload = format!("astral:{}:session:{}:{}", username, now, nonce);
let token = format!("{}.{}.{}", now, nonce, sig);
```

---

### H-2: No Server-Side Session Revocation — Logout Is Client-Side Only

**File:** `src/main.rs:308-326`, `web/src/App.svelte:37-48`

The auth middleware is fully stateless: it validates the HMAC and checks the timestamp. `handleLogout()` removes the token from `sessionStorage` only. The token remains cryptographically valid for its full 24-hour lifetime after logout. A stolen token cannot be invalidated without restarting the server (which rotates the HMAC secret).

**Recommendation:** Maintain an in-memory revocation set (or a `revoked_tokens` SQLite table) keyed by token signature. Check it in `auth_middleware` before accepting a token. On logout, `POST /api/logout` adds the signature to the revocation set.

---

### H-3: Missing Content-Security-Policy Header

**File:** `src/main.rs:205-212`

The security headers middleware sets three headers but omits `Content-Security-Policy`:

```rust
headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
headers.insert("X-Frame-Options", "DENY".parse().unwrap());
headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
// CSP missing
```

Without CSP, any XSS vector (e.g., a process name containing `<script>` rendered in the DOM) executes with no browser-level restriction. The process list in `metrics.rs:104-114` is collected from the OS and streamed directly to the frontend.

**Recommendation:**

```rust
headers.insert(
    "Content-Security-Policy",
    "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'"
        .parse().unwrap()
);
```

---

### H-4: Webhook URL Not Validated — Server-Side Request Forgery (SSRF)

**File:** `src/worker.rs:163-173`

The `--webhook` CLI argument is forwarded to `reqwest::Client::post()` without any validation of scheme, host, or port:

```rust
if let Some(url) = &self.webhook_url {
    let client = reqwest::Client::new(); // no timeout, no URL validation
    client.post(url).json(&payload).send().await
}
```

An attacker who can set this argument can direct the server to POST to internal services: cloud metadata endpoints (`http://169.254.169.254/`), internal databases, or `localhost` admin interfaces. There is also no request timeout, so a slow endpoint blocks the worker indefinitely.

**Recommendation:** Validate that the URL uses `https://` and resolves to a public IP address. Build the client with an explicit timeout:

```rust
reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(10))
    .build()?
```

---

### H-5: Rate Limiter HashMap Grows Unbounded — Memory Exhaustion

**File:** `src/main.rs:73-98`

`LoginLimiter` stores `(attempt_count, Instant)` per source IP in a `HashMap` that is never pruned:

```rust
attempts: std::sync::Arc<tokio::sync::Mutex<HashMap<std::net::IpAddr, (u32, Instant)>>>,
```

Entries are only reset when the same IP makes a new request after its window expires. A distributed attacker sending a single request from many unique IPs (trivial with IPv6 — 2^128 addresses) continuously grows this map, exhausting server memory without triggering any rate limit.

**Recommendation:** Add periodic eviction of entries older than `RATE_LIMIT_WINDOW_SECS`. Run a background task every 5 minutes that drains stale entries, or cap the map size with an LRU cache (e.g., the `lru` crate).

---

## Medium

---

### M-1: Auth Token Stored in `sessionStorage` — Vulnerable to XSS Theft

**File:** `web/src/lib/components/Login.svelte:22`, `web/src/App.svelte:28`

```js
sessionStorage.setItem('astral_token', data.token);
```

`sessionStorage` is fully readable by any JavaScript executing on the page. If any XSS vector exists, the token is trivially exfiltrated with `sessionStorage.getItem('astral_token')`. While `sessionStorage` does not persist across browser restarts (unlike `localStorage`), the 24-hour token window provides ample time for exploitation.

**Recommendation:** Use `HttpOnly; Secure; SameSite=Strict` cookies for session tokens. `HttpOnly` cookies are inaccessible to JavaScript entirely. This requires changing the auth flow: the server sets the cookie on login response and reads it in `auth_middleware` instead of checking the `Authorization` header.

---

### M-2: Colon in Password Causes Opaque Startup Failure

**File:** `src/main.rs:107-111`

```rust
let parts: Vec<&str> = auth_str.split(':').collect();
if parts.len() != 2 {
    anyhow::bail!("Invalid auth format. Expected USER:PASS");
}
```

A password containing a colon (e.g., `--auth admin:pass:word`) produces `parts.len() == 3`, failing with a generic error that gives no indication the password itself is the problem. This silently prevents a broad class of strong passwords.

**Recommendation:** Use `splitn(2, ':')` to split only on the first colon:

```rust
let (username, password) = auth_str.split_once(':')
    .ok_or_else(|| anyhow::anyhow!("Invalid auth format. Expected USER:PASS"))?;
```

---

### M-3: Generated Credentials Printed to stdout / Log Streams

**File:** `src/main.rs:115`

```rust
println!("No auth provided. Generated credentials: {}:{}", username, password);
```

In containerized or systemd-managed deployments, stdout is captured and persisted in log aggregation systems (Docker logs, CloudWatch, ELK, Loki, journald) that may be accessible to a broader audience than the server operator. The generated password appears in plain text in potentially long-lived log archives.

**Recommendation:** Print the password once to stderr with a clear "save this now" warning, and do not emit it through structured logging pipelines:

```rust
eprintln!("⚠ SAVE THESE CREDENTIALS — they will not be shown again:");
eprintln!("  Username: {}", username);
eprintln!("  Password: {}", password);
```

---

### M-4: No Request Timeout on Webhook HTTP Client

**File:** `src/worker.rs:170`

```rust
let client = reqwest::Client::new(); // default: no timeout
```

The default `reqwest::Client` has no connect or read timeout configured. A slow or unresponsive webhook endpoint causes `send_alert` to block indefinitely. Because this runs inside the `Worker::run` select loop, it delays or drops subsequent metric aggregation ticks while the request hangs.

**Fix:** Covered together with H-4 above — build a shared `reqwest::Client` with an explicit 10-second timeout stored on the `Worker` struct.

---

### M-5: Full Process List Streamed to All Authenticated Clients

**File:** `src/metrics.rs:104-114`

Every SSE message includes up to 15 processes with names, PIDs, CPU, and memory usage:

```rust
let mut procs: Vec<ProcessInfo> = self.system.processes().iter().map(|(pid, proc_)| {
    ProcessInfo { pid: pid.as_u32(), name: proc_.name()... }
}).collect();
procs.truncate(15);
```

In shared or multi-tenant hosting environments, the full process list may reveal sensitive information about co-located services to any authenticated operator.

**Recommendation:** Gate process-list inclusion behind an explicit opt-in flag (`--enable-process-list`) that defaults to off, or provide it only via a separate authenticated endpoint.

---

## Low

---

### L-1: `unwrap()` / `expect()` in Cryptographic Hot Paths Can Crash the Server

**File:** `src/main.rs:120-123`, `main.rs:233`, `main.rs:257`

```rust
let password_hash = Argon2::default()
    .hash_password(password.as_bytes(), &salt)
    .expect("Failed to hash password"); // panics on OOM

HmacSha256::new_from_slice(secret.as_bytes()).unwrap(); // panics if key is empty
```

While these are unlikely to fail under normal conditions, panics in async Tokio tasks terminate the task (potentially the entire runtime if in `main`). Using `?` propagation is idiomatic Rust and prevents unexpected crashes.

---

### L-2: SQLite Not Configured with WAL Mode

**File:** `src/main.rs:134`

```rust
Db::new("sqlite:astral.db?mode=rwc").await?
```

Without WAL (Write-Ahead Logging), concurrent readers block during writes and a crash mid-write can corrupt the database. The worker writes aggregated metrics every 60 seconds, while history reads can arrive at any time.

**Recommendation:** Add WAL mode to the connection string or execute `PRAGMA journal_mode=WAL` immediately after connection:

```rust
Db::new("sqlite:astral.db?mode=rwc&_journal_mode=WAL").await?
```

---

### L-3: Missing `Strict-Transport-Security` Header

**File:** `src/main.rs:205-212`

Even when TLS is terminated by a reverse proxy, the app does not emit `Strict-Transport-Security`. Browsers will not enforce HTTPS-only connections, leaving the path open for protocol downgrade attacks in mixed-content or misconfigured proxy scenarios.

**Recommendation:**

```rust
headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
```

---

### L-4: No Alert Cooldown — Alert Flood During Sustained High Load

**File:** `src/worker.rs:106-115`

`check_alert_condition` re-triggers on every `process_1m` tick (every 60 seconds) during a sustained high-CPU or high-memory event. There is no cooldown after an alert fires. A 1-hour CPU spike generates 60 repeated alerts to both the SSE stream and the webhook endpoint, overwhelming the operator and potentially the webhook receiver.

**Recommendation:** Track `last_cpu_alert_at` and `last_ram_alert_at` timestamps on `Worker`. Only fire if the previous alert of the same kind was more than a configurable interval ago (e.g., 15 minutes):

```rust
if alert_triggered && now - self.last_alert_at > ALERT_COOLDOWN_SECS {
    self.send_alert(...).await?;
    self.last_alert_at = now;
}
```

---

## Summary Table

| ID  | Severity     | File                    | Issue                                              | Status |
|-----|--------------|-------------------------|----------------------------------------------------|--------|
| C-1 | **Critical** | `src/main.rs:107`       | Password exposed in CLI argument / process list    | ✅ Fixed — `env = "ASTRAL_AUTH"` added; `split_once` used |
| C-2 | **Critical** | `src/main.rs:219`       | No TLS — credentials transmitted in plaintext      | ⚠️ Deployment — TLS must be terminated at reverse proxy (nginx/Caddy); HSTS header added |
| H-1 | **High**     | `src/main.rs:228`       | No nonce in session token — identical tokens possible | ✅ Fixed — token format changed to `{ts}.{nonce}.{hmac}` |
| H-2 | **High**     | `src/main.rs:308`       | No server-side session revocation                  | ✅ Fixed — `RevocationSet` + `POST /api/logout` endpoint |
| H-3 | **High**     | `src/main.rs:205`       | Missing Content-Security-Policy header             | ✅ Fixed — CSP header added to security middleware |
| H-4 | **High**     | `src/worker.rs:163`     | Webhook URL not validated — SSRF risk              | ✅ Fixed — `https://` scheme required; HTTP rejected |
| H-5 | **High**     | `src/main.rs:73`        | Rate limiter HashMap unbounded — memory exhaustion | ✅ Fixed — background eviction task every 5 minutes |
| M-1 | **Medium**   | `Login.svelte:22`       | Auth token in sessionStorage (XSS-stealable)       | ✅ Fixed — `HttpOnly; SameSite=Strict` cookie; `sessionStorage` removed |
| M-2 | **Medium**   | `src/main.rs:107`       | Colon in password causes opaque startup failure    | ✅ Fixed — `split_once(':')` replaces `split(':').collect()` |
| M-3 | **Medium**   | `src/main.rs:115`       | Generated credentials printed to stdout/logs       | ✅ Fixed — credentials written to `stderr` with save warning |
| M-4 | **Medium**   | `src/worker.rs:170`     | No timeout on webhook HTTP client                  | ✅ Fixed — `reqwest::ClientBuilder::timeout(10s)` |
| M-5 | **Medium**   | `src/metrics.rs:104`    | Full process list streamed to all authenticated clients | ✅ Fixed — opt-in via `--enable-process-list` flag (default off) |
| L-1 | **Low**      | `src/main.rs:120`       | `unwrap()`/`expect()` in crypto paths can crash server | ✅ Fixed — `.map_err(...)?` propagation in `main()` |
| L-2 | **Low**      | `src/main.rs:134`       | SQLite missing WAL mode                            | ✅ Fixed — `PRAGMA journal_mode=WAL` in `Db::init()` |
| L-3 | **Low**      | `src/main.rs:205`       | Missing `Strict-Transport-Security` header         | ✅ Fixed — HSTS header added to security middleware |
| L-4 | **Low**      | `src/worker.rs:106`     | No alert cooldown — alert flood on sustained high load | ✅ Fixed — 15-minute per-kind cooldown on `Worker` |

**15 of 16 findings fixed in code. C-2 requires a reverse proxy with TLS — not addressable in the binary itself.**

---

## Remediation Priority

1. **Immediate (before any network exposure):** C-1 ✅, C-2 ⚠️ — without these, all other controls are undermined
2. **Before first production deployment:** H-1 ✅, H-2 ✅, H-3 ✅, H-4 ✅, H-5 ✅, M-1 ✅
3. **Next sprint:** M-2 ✅, M-3 ✅, M-4 ✅, M-5 ✅
4. **Backlog / hardening:** L-1 ✅, L-2 ✅, L-3 ✅, L-4 ✅
