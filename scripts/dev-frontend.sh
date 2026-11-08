#!/usr/bin/env bash
# `tauri dev` runs this: start the web app's dev server on http://localhost:3000.
# In development the window loads the dev server directly and talks to the API at
# NEXT_PUBLIC_API_URL (default http://localhost:8000/api/v1), so no proxy is involved.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="${TXIO_FRONTEND_DIR:-$ROOT/../Txio-frontend}"
if [[ ! -d "$SRC" ]]; then
  echo "No web app checkout at $SRC. Clone Txio-labs/txio there or set TXIO_FRONTEND_DIR." >&2
  exit 1
fi
cd "$SRC"
[[ -d node_modules ]] || npm ci --no-audit --no-fund
exec npm run dev -- --port 3000
