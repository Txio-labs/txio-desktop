#!/usr/bin/env bash
# Build the Txio web app as a static export into dist/, ready for the desktop bundle.
#
#   scripts/build-frontend.sh
#
# Source of the web app, in order:
#   TXIO_FRONTEND_DIR   a local checkout (default: ../Txio-frontend if it exists)
#   otherwise           a fresh clone of Txio-labs/txio at TXIO_FRONTEND_REF (default: main)
#
# The app is built with NEXT_PUBLIC_API_URL=/api/v1: the desktop shell serves the app and
# proxies /api/* to the Txio API from the same origin, so the session cookie works
# (see src-tauri/src/proxy.rs).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO_URL="${TXIO_FRONTEND_REPO:-https://github.com/Txio-labs/txio.git}"
REF="${TXIO_FRONTEND_REF:-main}"

SRC="${TXIO_FRONTEND_DIR:-}"
if [[ -z "$SRC" && -d "$ROOT/../Txio-frontend" ]]; then
  SRC="$(cd "$ROOT/../Txio-frontend" && pwd)"
fi
if [[ -z "$SRC" ]]; then
  SRC="$ROOT/.frontend"
  if [[ -d "$SRC/.git" ]]; then
    git -C "$SRC" fetch --depth 1 origin "$REF"
    git -C "$SRC" checkout -q --force FETCH_HEAD
  else
    rm -rf "$SRC"
    git clone --depth 1 --branch "$REF" "$REPO_URL" "$SRC"
  fi
fi
echo "Building the web app from $SRC" >&2

cd "$SRC"
if [[ ! -d node_modules ]]; then
  npm ci --no-audit --no-fund
fi
NEXT_PUBLIC_API_URL=/api/v1 NEXT_TELEMETRY_DISABLED=1 npm run build

rm -rf "$ROOT/dist"
cp -R "$SRC/out" "$ROOT/dist"
echo "Web app copied to $ROOT/dist ($(du -sh "$ROOT/dist" | cut -f1))" >&2
