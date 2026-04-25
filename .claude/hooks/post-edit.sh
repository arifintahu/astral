#!/usr/bin/env bash
# Called by Claude Code PostToolUse hook after Edit or Write tool uses.
# Stdin: JSON hook payload. Requires node (available via npm).

set -euo pipefail

PAYLOAD="$(cat)"

FILE_PATH="$(node -e "
  try {
    const p = JSON.parse(process.argv[1]);
    const fp = (p.tool_input && p.tool_input.file_path) ? p.tool_input.file_path : '';
    process.stdout.write(fp);
  } catch(e) { process.stdout.write(''); }
" "$PAYLOAD" 2>/dev/null || echo "")"

# Normalise backslashes to forward slashes (Windows paths via git bash)
FILE_PATH="${FILE_PATH//\\//}"

if [ -z "$FILE_PATH" ]; then
  exit 0
fi

if [[ "$FILE_PATH" == *.rs ]]; then
  echo "[hook] Rust changed: $FILE_PATH — running cargo check"
  cargo check 2>&1 | head -20
elif [[ "$FILE_PATH" == *web/src/* || "$FILE_PATH" == *.svelte || "$FILE_PATH" == *.ts ]]; then
  echo "[hook] Frontend changed: $FILE_PATH — running vite build (type check)"
  (cd web && node_modules/.bin/vite build --mode development 2>&1 | tail -10) \
    || (cd web && npx --no-install vite build --mode development 2>&1 | tail -10)
fi
