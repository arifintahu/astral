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
timeout = 20 # seconds

while True:
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(0.1)
        result = sock.connect_ex((target_host, target_port))
        if result == 0:
            end_time = time.time()
            print(f"{end_time - start_time:.4f}")
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

# Measure RSS now that it's running
# ps -p PID -o rss returns size in KB
RSS_KB=$(ps -p $SERVER_PID -o rss --no-headers)
if [ -n "$RSS_KB" ]; then
    RSS_MB=$(echo "$RSS_KB" | awk '{printf "%.2f", $1/1024}')
else
    RSS_MB="N/A"
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
echo "Execution Time:    ${STARTUP_TIME}s (Time to Ready)"
echo "Peak Memory (RSS): ${RSS_MB} MB"
echo "========================================"
