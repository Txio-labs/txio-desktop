#!/usr/bin/env bash
# Format, lint and test the Rust shell. tauri-build needs a dist/ folder to exist; the
# check uses a placeholder page so it doesn't have to build the whole web app.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
if [[ ! -f "$ROOT/dist/index.html" ]]; then
  mkdir -p "$ROOT/dist" && echo '<!doctype html><title>Txio</title>' > "$ROOT/dist/index.html"
fi
cd "$ROOT/src-tauri"
cargo fmt --check
cargo clippy --all-targets --quiet -- -D warnings
cargo test --quiet
python3 -m py_compile "$ROOT"/scripts/*.py
sh -n "$ROOT/site/install.sh"
echo "All checks passed."
