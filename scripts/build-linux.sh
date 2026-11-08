#!/usr/bin/env bash
# Build Linux packages (.deb, .rpm and AppImage) into release/.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# Update packages are signed. Use the local key if present.
if [[ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" && -f "$HOME/.tauri/txio-updater.key" ]]; then
  export TAURI_SIGNING_PRIVATE_KEY="$(cat "$HOME/.tauri/txio-updater.key")"
  export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="$(cat "$HOME/.tauri/txio-updater.password" 2>/dev/null || true)"
fi
cd "$ROOT"
npx tauri build --bundles "${TXIO_BUNDLES:-deb,rpm,appimage}"
TARGET="${CARGO_TARGET_DIR:-$ROOT/src-tauri/target}"
mkdir -p "$ROOT/release"
find "$TARGET/release/bundle" \( -name '*.deb' -o -name '*.rpm' -o -name '*.AppImage' \) -exec cp {} "$ROOT/release/" \;
(cd "$ROOT/release" && sha256sum ./* > SHA256SUMS)
ls -lh "$ROOT/release"
