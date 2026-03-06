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
