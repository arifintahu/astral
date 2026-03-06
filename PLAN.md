# Product Requirements Document: Astral

## 1. Product Vision
**Astral** is a pragmatic, ultra-lightweight virtual machine monitoring dashboard. It aims to bridge the gap between running basic CLI tools like `htop` and deploying heavy, complex observability stacks like Prometheus and Grafana. Astral provides a modern, single-page web UI that displays both real-time system health and historical trends, secured by default, with built-in webhook alerting, all packaged into a single, easily deployable compiled binary. 

## 2. Core Philosophy & Design Principles
* **Pragmatic Low-Dependency:** Leverage modern build-time tools (Svelte, Tailwind) and efficient libraries (uPlot, axum) to ensure a great developer experience without burdening the end-user with runtime dependencies (Node.js, Python, external databases).
* **Frictionless Deployment:** The end user only needs to execute `./astral`. The backend, frontend, and embedded database are bundled together.
* **Single-Page Elegance:** All critical metrics—live and historical—must be immediately visible without navigating menus or tabs.
* **Self-Managing Storage:** Historical data uses an embedded database (SQLite) with automated round-robin retention policies. 

---

## 3. Functional Requirements

### 3.1. Live Metrics Collection & Streaming
The system must collect the following metrics directly from the host OS and stream them to the UI in real-time (1-second intervals) via Server-Sent Events (SSE):
* **System Info:** Hostname, OS version, total Uptime.
* **CPU:** Overall utilization percentage and core count.
* **Memory:** Total RAM, Used RAM, Total Swap, Used Swap.
* **Network I/O:** Real-time transmit (Tx) and receive (Rx) speeds.
* **Storage/Disk:** Capacity and usage per mounted partition.

### 3.2. Historical Data, Rollups & Retention


The system must store historical metrics locally and provide APIs for the frontend to render time-series charts. To prevent infinite database growth, data must be aggregated (downsampled) at specific intervals.

**Data Retention Policy & Validation:**
By default, the maximum historical retention is **7 days** (Customizable via CLI flag, Max **90 days**). 

| View Window | Datapoint Resolution | Total Points Displayed | Data Source / Table | Cleanup Routine (Default) |
| :--- | :--- | :--- | :--- | :--- |
| **6 Hours** | 5 Minutes | 72 | `metrics_5m` | Delete data > 24 hours old |
| **24 Hours** | 15 Minutes | 96 | `metrics_15m` | Delete data > 3 days old |
| **7 Days** | 1 Hour | 168 | `metrics_1h` | Delete data > 7 days old (Customizable) |
| **All/Max** | 1 Hour | Variable | `metrics_1h` | Bounded by `--retention` flag (Max 90d) |

### 3.3. Security (Authentication)
The web UI and all API endpoints must be protected. 
* The system will use standard HTTP Basic Authentication.
* Credentials are provided at startup via the CLI. If no credentials are provided, the system defaults to generating a random secure password on startup and printing it to the console.

### 3.4. Alerting (Webhooks)
The system must evaluate system health and push alerts to external services (like Discord, Slack, or custom endpoints) to notify administrators of degradation.
* **Evaluation Interval:** The background worker evaluates the 1-minute rolling average of CPU and RAM.
* **Trigger Condition:** If the metric exceeds the configured threshold for **5 consecutive minutes**, an alert is fired.
* **Cooldown:** Once an alert fires, a 30-minute cooldown is applied to that specific metric to prevent webhook spam.

### 3.5. API Endpoints
* `GET /`: Serves the bundled Svelte application (Protected by Auth).
* `GET /api/stream`: Opens an SSE connection for live updates (Protected by Auth).
* `GET /api/history?window={6h|24h|7d|all}`: Returns JSON arrays of historical aggregated data (Protected by Auth).

---

## 4. Technical Architecture

### 4.1. Backend (Rust)
* **Metrics Engine:** `sysinfo` (Cross-platform OS data collection).
* **Web Server:** `axum` + `tokio` (Async routing and SSE streaming).
* **Middleware:** `tower-http` (Provides the HTTP Basic Authentication layer).
* **HTTP Client:** `reqwest` (Asynchronous client used by Tokio background tasks to dispatch Webhook JSON payloads without blocking metrics collection).
* **Embedded Database:** `sqlx` with SQLite (Asynchronous SQL driver).
* **Asset Bundling:** `rust-embed` (Compiles the Vite/Svelte build output directly into the binary).
* **CLI Parsing:** `clap` (For handling configuration flags and input validation).

### 4.2. Frontend (Pragmatic Web Stack)
* **Framework:** Svelte (via Vite) for a lightweight, compiler-driven component structure.
* **Styling:** Tailwind CSS for rapid, utility-first styling.
* **Charting:** `uPlot` for highly performant, low-memory time-series charts.

---

## 5. UI/UX Requirements
* **Theme:** Default dark mode with high-contrast, modern accent colors.
* **Layout:** Responsive CSS Grid. 
  * **Top Bar:** Hostname, Uptime, Live Status Indicator, and a toggle for Historical Timeframes.
  * **Main Grid:** Four primary quadrants for CPU, Memory, Network, and Disk.
* **Interactivity:** * Live gauge/sparkline components for real-time SSE data.
  * uPlot line/area charts for historical data that update dynamically.
  * Tooltips on historical charts showing exact timestamps and values.

---

## 6. Configuration Flags
### 6.1. Core Settings
* `--port <PORT>`: Define the listening port (Default: `8080`).
* `--retention <DAYS>`: Maximum historical data retention in days (Max: `90`, Default: `7`).

### 6.2. Security & Alerting
* `--auth <USER:PASS>`: Define the HTTP Basic Auth credentials. (Default: `admin:<RANDOM_GENERATED_STRING>`).
* `--webhook <URL>`: The endpoint URL to receive POST requests when an alert triggers.
* `--alert-cpu <PERCENT>`: The CPU usage threshold (0-100) that triggers an alert if sustained for 5 minutes. (Default: `90`).
* `--alert-ram <PERCENT>`: The Memory usage threshold (0-100) that triggers an alert if sustained for 5 minutes. (Default: `90`).