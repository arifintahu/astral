# Changelog

All notable changes are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org).

## [Unreleased]

### Changed

- **New "volt" design:** a lime accent on near-black. A status headline ("All systems nominal." or "CPU is running hot.") sits above four metric tiles (CPU, Memory, Network, Disk) that are also the tabs of one hero chart. The chart starts on a live 60-second stream and switches to 6H / 24H / 7D / All. Below it are the top processes and a details panel with load average, swap and mounts. Login, Settings, alert history, alert toasts and a sign-out confirmation are restyled. (#13)
- **The headline follows your alert thresholds:** "running warm" and "running hot" use the CPU and memory limits from Settings instead of a fixed 70% / 90%. (#13)
- **Tab icon and install icon** match the new logo. (#13)

### Fixed

- **Login animation loop:** the signal trace no longer snaps back at the end of each loop. (#13)
- **Windows load average:** it is hidden on Windows, where the system reports no load average and the dashboard used to show 0.00. (#13)
- **Disk tile and mounts agree:** the tile and headline use the root (or system) mount, and the mounts panel lists it first. The disk headline no longer mentions an alert threshold, because there is no disk alert. (#13)
- **Login page over HTTP:** it says "Unencrypted connection (HTTP)" instead of always claiming the connection is encrypted. (#13)

## [1.1.1] - 2026-10-27

### Highlights

- **Lower memory use:** idle peak memory dropped from 23.5 MB to 13.5 MB (measured on WSL) by moving the startup password hash to the OWASP 9 MiB Argon2 profile. (#11)
- **A redesigned interface:** a new token-based design system with Geist fonts, a custom SVG history chart, and a settings drawer. (#9)

### Changed

- **Argon2 profile:** the startup hash uses 9 MiB (t=4) instead of the crate default of 19 MiB. (#11)
- **UI redesign:** new cards, sparklines, a custom SVG chart in place of uPlot, and a split-layout login page. (#9)

### Added

- **Revoke all sessions:** `POST /api/sessions/revoke` signs out every device. (#9)
- **Load average:** the live stream carries `cpu_load` (1, 5 and 15 minute averages). (#9)

### Fixed

- **Benchmark:** `benchmark.sh` reported current memory about 1 ms after start and called it the peak. It now warms up and reads `VmHWM`. (#11)

## [1.1.0] - 2026-04-25

### Highlights

- **Settings without a restart:** change the CPU and memory alert thresholds, process monitoring, retention and the Slack webhook from the dashboard.
- **Alert history:** past alerts are kept on the server and shown in a drawer.
- **Safer sign-in:** sessions use `HttpOnly` cookies instead of browser storage, and sign-in tokens can be revoked. (#7, #8)

### Added

- **Dynamic configuration:** `GET` and `POST /api/settings` for process monitoring, alert thresholds, retention days and a Slack webhook. (#8)
- **Alert history:** `GET /api/alerts/history`. (#8)
- **CSV export** of historical metrics at `/api/history/export`. (#8)
- **Disk I/O rates** are stored with each history point. (#8)

### Changed

- **Session auth:** `HttpOnly; SameSite=Strict` cookies replace `sessionStorage` tokens. (#8)
- **Docker Compose** binds the web UI to `127.0.0.1` by default. (#7)
- **Process list** is off unless you pass `--enable-process-list` or turn it on in Settings. (#7)

### Security

- Session revocation by token signature, with a per-login nonce so tokens are unique. (#7)
- Webhooks must be HTTPS and have a timeout. (#7)
- A Content Security Policy and HSTS are sent on every response. (#7)
- SQLite runs in WAL mode, and the login rate limiter no longer leaks memory. (#7)
- Credentials can come from `ASTRAL_AUTH`, and passwords containing `:` parse correctly. (#7)
- Alerts have a 15-minute cooldown per kind so a sustained problem does not flood a webhook. (#7)

## [1.0.0] - 2026-03-07

First stable release.

### Highlights

- **One small binary:** the Svelte frontend is embedded in the Rust binary, with no runtime dependencies on the host.
- **Live and historical metrics:** CPU, memory, network and disk over Server-Sent Events, with 6 hours, 24 hours and 7 days of history in an embedded SQLite database.
- **Alerts:** sustained high CPU or memory (5 minutes) sends a webhook and shows a toast in the dashboard.
- **Ready to deploy:** a Docker setup that monitors the host, and release binaries for Linux, Windows and macOS. (#2, #3)

### Added

- A top-processes list, per-disk I/O, network sparklines and a web app manifest with a service worker. (#1)
- Session-token sign-in with Argon2 password hashing and login rate limiting. (#1, #5)
- A benchmark script and a size-optimised release build: about 7 MB, ready in about 1.5 ms. (#4)
- The MIT license.

[Unreleased]: https://github.com/arifintahu/astral/compare/v1.1.1...HEAD
[1.1.1]: https://github.com/arifintahu/astral/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/arifintahu/astral/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/arifintahu/astral/releases/tag/v1.0.0
