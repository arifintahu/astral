# Astral

Astral is a pragmatic, ultra-lightweight virtual machine monitoring dashboard. It bridges the gap between basic CLI tools like `htop` and complex observability stacks like Prometheus/Grafana.

Astral provides a modern, single-page web UI that displays real-time system health and historical trends, secured by default, with built-in webhook alerting—all packaged into a single, easily deployable binary.

## Features

-   **Real-time Monitoring:** Live updates (1-second intervals) for CPU, Memory, Network, and Disk usage via Server-Sent Events (SSE).
-   **Historical Data:** Interactive charts for viewing trends over 6 hours, 24 hours, or 7 days, backed by an embedded SQLite database.
-   **Single Binary Deployment:** The Svelte frontend is embedded directly into the Rust binary. No external runtime dependencies (Node.js, Python, etc.) required on the host.
-   **Secure by Default:** Built-in HTTP Basic Authentication.
-   **Alerting:** Configurable webhook alerts for high CPU or Memory usage (sustained for 5 minutes).
-   **Lightweight:** Minimal resource footprint, designed for small to medium-sized VPS and servers.

## Prerequisites

To build Astral from source, you need:

-   **Rust:** Latest stable version (install via [rustup](https://rustup.rs/)).
-   **Node.js & npm:** For building the frontend assets (only required for build).

## Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/yourusername/astral.git
    cd astral
    ```

2.  **Build the Frontend:**
    ```bash
    cd web
    npm install
    npm run build
    cd ..
    ```

3.  **Run the Backend:**
    ```bash
    cargo run --release
    ```

    Or build a release binary:
    ```bash
    cargo build --release
    ./target/release/astral
    ```

## Usage

By default, Astral listens on port `8080`.

```bash
./astral [OPTIONS]
```

### Configuration Flags

| Flag | Description | Default |
| :--- | :--- | :--- |
| `--port <PORT>` | The port to listen on. | `8080` |
| `--auth <USER:PASS>` | HTTP Basic Auth credentials. | `admin:<random>` |
| `--retention <DAYS>` | Max historical data retention in days. | `7` |
| `--webhook <URL>` | Webhook URL for alerts. | `None` |
| `--alert-cpu <%>` | CPU threshold for alerts (0-100). | `90` |
| `--alert-ram <%>` | Memory threshold for alerts (0-100). | `90` |

### Examples

**Run with default settings (random password generated):**
```bash
./astral
```

**Run with custom credentials and port:**
```bash
./astral --port 3000 --auth admin:secret123
```

**Run with webhook alerting:**
```bash
./astral --webhook https://discord.com/api/webhooks/... --alert-cpu 80
```

## Development

To run the project in development mode:

1.  **Start the Frontend (HMR):**
    ```bash
    cd web
    npm run dev
    ```
    *Note: The Rust backend expects the frontend to be built in `web/dist`. For true hot-reloading dev experience, you might need to proxy requests or rebuild the frontend on changes.*

2.  **Run the Backend:**
    ```bash
    cargo run
    ```

## Architecture

-   **Backend:** Rust (Axum, Tokio, SQLx, Sysinfo)
-   **Frontend:** Svelte 5, Tailwind CSS, uPlot
-   **Database:** Embedded SQLite (managed by SQLx)

## License

MIT
