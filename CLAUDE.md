# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is Astral

Astral is a lightweight VM monitoring dashboard — a single Rust binary with an embedded Svelte frontend that shows real-time and historical system metrics (CPU, memory, network, disk). It uses SSE for live updates, SQLite for historical data, and HTTP Basic Auth for security.

## Build & Run Commands

### Backend (Rust)
```bash
cargo build              # dev build
cargo build --release    # release build
cargo run                # run dev server (port 8080)
cargo run -- --port 3000 --auth admin:secret  # custom config
```

### Frontend (Svelte)
```bash
cd web && npm install    # install deps (first time)
cd web && npm run build  # build to web/dist (required before cargo build)
cd web && npm run dev    # dev server with HMR
```

**Important:** The frontend must be built (`web/dist`) before compiling the Rust binary — it's embedded via `rust-embed`.

## Architecture

### Data Flow
```
MetricsCollector (1s interval) → broadcast::channel → SSE endpoint (/api/stream)
                                                    → Worker → SQLite (aggregated)
```

### Backend Modules (`src/`)
- **main.rs** — Axum server setup, CLI args (clap), auth middleware, static file serving via `rust-embed`
- **metrics.rs** — `MetricsCollector` using `sysinfo` crate, collects every 1s, sends via broadcast channel
- **api.rs** — Two endpoints: `GET /api/stream` (SSE live metrics), `GET /api/history?window=6h|24h|7d|all` (historical data)
- **worker.rs** — Background task that aggregates metrics into time buckets and handles alert checking
- **db.rs** — SQLite via `sqlx`, four tables for different granularities: `metrics_1m`, `metrics_5m`, `metrics_15m`, `metrics_1h`

### Aggregation Pipeline (Worker)
Raw 1s samples → buffer averaged every 1m → `metrics_1m` → aggregated to `metrics_5m` → `metrics_15m` → `metrics_1h`. Each tier has its own retention/cleanup policy.

### Frontend (`web/src/`)
- Svelte 5 with runes (`$state`), Tailwind CSS v4 (via `@tailwindcss/vite`), uPlot for charts
- `App.svelte` — SSE connection to `/api/stream`, distributes metrics to card components
- Components: `TopBar`, `CpuCard`, `MemoryCard`, `NetworkCard`, `DiskCard`, `HistoryChart`
- Types mirror Rust structs in `web/src/lib/types.ts`

### Key Design Decisions
- Rust edition 2024
- No ORM migrations — tables created programmatically in `db.rs::init()`
- Auth is a closure-based Axum middleware, not tower-http's built-in
- History API maps window params to specific tables (e.g., `6h` → `metrics_5m`)
- Database file: `astral.db` in the working directory

## Rules of Thumb

- **Build order is mandatory**: Run `cd web && npm run build` before `cargo build`. The Rust binary embeds `web/dist/` via `rust-embed` — a missing or stale `web/dist/` produces a binary that silently serves wrong assets with no compile error.
- **TypeScript mirrors Rust structs**: Any field added/removed/renamed in a struct serialised over SSE or the history API must be mirrored in `web/src/lib/types.ts` immediately. Field names must match exactly — the frontend deserialises raw JSON with no schema validation.
- **SSE event shape is a stable contract**: Do not rename or remove top-level fields emitted on `/api/stream`. Additions are safe; renames break the frontend silently.
- **Four-tier aggregation is authoritative**: The DB has exactly four tables — `metrics_1m`, `metrics_5m`, `metrics_15m`, `metrics_1h`. Adding a fifth granularity requires updating `db.rs`, `api.rs` (history routing), and `worker.rs` (tickers) atomically.
- **Never hardcode port or credentials**: Port and auth are always configured via `--port` / `--auth` CLI args. Embedding a literal port or credential in source breaks Docker and any non-default deployment.
- **Error propagation uses `?`**: Handlers and background tasks use `?` with `anyhow::Result`. `.unwrap()` / `.expect()` are only acceptable in unit tests and `main()` startup (where panic is intentional).

## Code Guidelines

### Rust

- Rust edition 2024 — use edition-2024 idioms.
- Run `cargo fmt` before every commit.
- Run `cargo clippy -- -D warnings` before every commit. Resolve all warnings; do not `#[allow(...)]` without an explanatory comment.
- Use `anyhow::Result<T>` for all fallible cross-module functions. Define custom error types only when call sites need to match on variants.
- Use `tokio::spawn` for background tasks, `tokio::select!` for multi-branch async, `broadcast::channel` for fan-out.

### TypeScript / Svelte

- **Svelte 5 runes only**: `$state`, `$derived`, `$effect`. No Svelte 4 reactive syntax (`$:`, `writable()`, `export let` stores).
- All new data shapes go in `web/src/lib/types.ts` as TypeScript interfaces — no inline `any`.
- Style with Tailwind utility classes only. No `<style>` blocks. The project uses Tailwind v4 via `@tailwindcss/vite` — no `tailwind.config.js` is needed.
- `svelte-check` is intentionally not installed. Use `npm run build` as the type-check step.

### General

- Astral ships as a single self-contained binary. Avoid adding runtime file dependencies or secondary processes.
- `astral.db` is always created in the working directory via the `sqlite:astral.db?mode=rwc` connection string. Never hard-code an absolute path.
