#!/usr/bin/env bash
# One-time developer setup.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
(cd "$ROOT" && npm ci)
cat <<'MSG'

Setup complete.
Linux system packages needed by Tauri (Debian / Ubuntu / Kali):
  sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev librsvg2-dev \
    libayatana-appindicator3-dev build-essential libssl-dev patchelf file

The web app is expected at ../Txio-frontend (a checkout of Txio-labs/txio), or set TXIO_FRONTEND_DIR.

Run the app in development:   scripts/dev.sh
Run all checks:               scripts/test.sh
Build installable packages:   scripts/build-linux.sh
MSG
