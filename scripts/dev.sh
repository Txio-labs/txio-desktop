#!/usr/bin/env bash
# Run the desktop app against the web app's dev server (../Txio-frontend or TXIO_FRONTEND_DIR).
set -euo pipefail
cd "$(dirname "$0")/.."
exec npx tauri dev
