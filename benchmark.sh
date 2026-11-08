#!/bin/bash
set -e

echo "Starting Astral Benchmark..."

# 1. Build Release Binary
echo "Building release binary..."
# Ensure frontend is built first (assuming it is, or add instructions)
if [ ! -d "web/dist" ]; then
    echo "Building frontend..."
    cd web && npm install && npm run build && cd ..
fi

# Build quietly
cargo build --release --quiet

BINARY_PATH="./target/release/astral"
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    exit 1
fi

# 2. Measure Binary Size
BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
BINARY_SIZE_BYTES=$(wc -c < "$BINARY_PATH")


# 3. Measure Startup Time and Peak RSS
PORT=9094
LOG_FILE="benchmark.log"

# Create a small python script to wait for port
cat <<EOF > wait_for_port.py
import socket
import time
import sys

start_time = time.time()
target_host = "127.0.0.1"
target_port = $PORT
timeout = 10 # seconds

while True:
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(0.1)
        result = sock.connect_ex((target_host, target_port))
        if result == 0:
            end_time = time.time()
            print(f"{(end_time - start_time) * 1000:.2f}")
            sys.exit(0)
        sock.close()
    except Exception:
        pass
    
    if time.time() - start_time > timeout:
        sys.exit(1)
    time.sleep(0.01)
EOF

echo "Measuring startup time and RSS..."

# Start the binary in background, capturing PID
$BINARY_PATH --port $PORT > "$LOG_FILE" 2>&1 &
SERVER_PID=$!

# Measure time until port is open
# Allow python script to fail without crashing bash script immediately (set +e temporary)
set +e
STARTUP_TIME=$(python3 wait_for_port.py)
PY_EXIT_CODE=$?
set -e

if [ $PY_EXIT_CODE -ne 0 ]; then
    echo "Failed to connect to server within timeout."
    cat "$LOG_FILE"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

# Warm up so the readings cover the collector, DB pool and worker running,
# not just the moment the port opens.
sleep "${WARMUP_SECS:-10}"

# Idle CPU: the server's own user+system time over a window with no browser connected.
# Needs /proc (Linux); reported as a share of one core.
CPU_PCT="N/A"
if [ -r "/proc/$SERVER_PID/stat" ]; then
    CPU_SECS="${CPU_SECS:-30}"
    TICKS_PER_SEC=$(getconf CLK_TCK)
    T0=$(awk '{print $14+$15}' "/proc/$SERVER_PID/stat")
    sleep "$CPU_SECS"
    T1=$(awk '{print $14+$15}' "/proc/$SERVER_PID/stat")
    CPU_PCT=$(awk -v a="$T0" -v b="$T1" -v c="$TICKS_PER_SEC" -v s="$CPU_SECS" 'BEGIN{printf "%.2f", (b-a)/c/s*100}')
fi

# Peak RSS (VmHWM, kB) and current RSS (VmRSS) from /proc; falls back to ps where /proc is unavailable
RSS_KB=$(grep VmHWM "/proc/$SERVER_PID/status" 2>/dev/null | tr -s ' ' | cut -d' ' -f2)
NOW_KB=$(grep VmRSS "/proc/$SERVER_PID/status" 2>/dev/null | tr -s ' ' | cut -d' ' -f2)
THREADS=$(grep Threads "/proc/$SERVER_PID/status" 2>/dev/null | tr -s '[:space:]' ' ' | cut -d' ' -f2)
if [ -z "$RSS_KB" ]; then
    RSS_KB=$(ps -p $SERVER_PID -o rss --no-headers 2>/dev/null | tr -d ' ')
    NOW_KB="$RSS_KB"
fi
if [ -n "$RSS_KB" ]; then
    RSS_MB=$(echo "$RSS_KB" | awk '{printf "%.2f", $1/1024}')
    NOW_MB=$(echo "$NOW_KB" | awk '{printf "%.2f", $1/1024}')
else
    RSS_MB="N/A"
    NOW_MB="N/A"
fi

# Kill the server
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null || true

# Cleanup
rm -f "$LOG_FILE" wait_for_port.py

# 4. Report
echo ""
echo "========================================"
echo "           BENCHMARK RESULTS            "
echo "========================================"

echo "Binary Size:       $BINARY_SIZE ($BINARY_SIZE_BYTES bytes)"
echo "Execution Time:    ${STARTUP_TIME}ms (Time to Ready)"
echo "Idle CPU:          ${CPU_PCT}% of one core (no browser connected)"
echo "Memory (RSS):      ${NOW_MB} MB now, ${RSS_MB} MB peak (VmHWM)"
echo "Threads:           ${THREADS:-N/A}"
echo "========================================"
