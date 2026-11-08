# Txio Desktop

**One console for every chain, in its own window.**

Txio builds, simulates, signs and runs RPC calls and transactions on Sui, Ethereum and EVM
chains, Solana, Aptos and Stellar. The desktop app is the Txio web app bundled into a native
[Tauri](https://tauri.app) shell written in Rust: a small binary that uses the system webview,
updates itself, and keeps your session in the app's own data folder.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Install

The [download page](https://txio-labs.github.io/txio-desktop/) picks the right file for your
computer. Every release is also on the [Releases](https://github.com/Txio-labs/txio-desktop/releases) page.

| System | File | How to install |
| --- | --- | --- |
| Debian, Ubuntu, Kali | `Txio_1.0.0_amd64.deb` | `sudo apt install ./Txio_1.0.0_amd64.deb` |
| Fedora, openSUSE | `Txio-1.0.0-1.x86_64.rpm` | `sudo dnf install ./Txio-1.0.0-1.x86_64.rpm` |
| Any Linux | `Txio_1.0.0_amd64.AppImage` | `chmod +x` it, then run it |
| macOS, Apple silicon | `Txio_1.0.0_aarch64.dmg` | Open, drag to Applications |
| macOS, Intel | `Txio_1.0.0_x64.dmg` | Open, drag to Applications |
| Windows 10 and 11 | `Txio_1.0.0_x64-setup.exe` | Run it |
| Windows, managed PCs | `Txio_1.0.0_x64_en-US.msi` | Deploy with your usual tools |

**With a package manager** (updates arrive with the rest of your system):

```bash
# Linux, any distribution: adds the signed APT/RPM repository, then installs Txio
curl -fsSL https://txio-labs.github.io/txio-desktop/install.sh | sudo sh

# macOS
brew tap txio-labs/txio https://github.com/Txio-labs/txio-desktop && brew install --cask txio

# Windows
scoop bucket add txio https://github.com/Txio-labs/txio-desktop; scoop install txio
```

**Updates.** Txio checks GitHub Releases on launch. When a new version is out it asks you to
install it and restarts. Update packages are signed; the app refuses any that don't match the
public key in `src-tauri/tauri.conf.json`.

**Not code-signed yet.** macOS: if it says Txio can't be opened, right-click it in Applications
and choose Open, or run `xattr -dr com.apple.quarantine /Applications/Txio.app`. Windows: if
SmartScreen warns, choose More info, then Run anyway.

## How it works

```
┌──────────── Txio window (system webview) ────────────┐
│  txio://localhost/…   the bundled Next.js static app  │
│  txio://localhost/api/*  ──────────────┐              │
└────────────────────────────────────────┼──────────────┘
                                         ▼
                      Rust shell (src-tauri/src/proxy.rs)
                      cookie jar in the app data folder
                                         │ HTTPS
                                         ▼
                               https://api.txio.xyz
```

- **Same origin for the app and the API.** The web app signs in with an HttpOnly session
  cookie. Served from the app bundle, the page is cross-site to `api.txio.xyz` and the webview
  would drop that cookie, so the web app is built with `NEXT_PUBLIC_API_URL=/api/v1` and the
  shell forwards `/api/*` itself. The session lives in `session.json` in the app data folder
  (owner-only permissions), never in page script.
- **Sign in with Google, GitHub or X** finishes inside the window: provider pages load in it,
  and the redirect back to the API or `txio.xyz` is rewritten onto the app origin.
- **Links** to anything else (explorers, docs, wallets' sites) open in your browser. The
  window never opens a second webview and never loads `file://` URLs.
- **Content Security Policy** comes from `tauri.conf.json`; Tauri hashes the app's inline
  scripts at build time.
- **Menu and tray.** macOS has a menu (Go to Workspace / Docs, Check for Updates, Reload).
  Linux and Windows have no menu bar. A tray icon offers Show, Open Workspace, Check for
  Updates and Quit. Starting Txio again focuses the running window; size and position are
  remembered.

**Browser-extension wallets** (MetaMask, Phantom, Sui Wallet, Freighter) can't inject into a
desktop webview. Use a wallet the web app connects to without an extension, or txio.xyz in
your browser for extension wallets.

## Development

Needs Node.js 22, Rust (stable) and, on Linux, the Tauri system libraries:

```bash
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev librsvg2-dev \
  libayatana-appindicator3-dev build-essential libssl-dev patchelf file
```

The web app comes from a checkout of [Txio-labs/txio](https://github.com/Txio-labs/txio) at
`../Txio-frontend`, or wherever `TXIO_FRONTEND_DIR` points.

```bash
scripts/setup-dev.sh     # npm ci
scripts/dev.sh           # tauri dev: window on the web app's dev server (localhost:3000)
scripts/test.sh          # cargo fmt, clippy, tests
scripts/build-linux.sh   # .deb, .rpm and AppImage into release/
```

`npm run frontend` builds the web app's static export into `dist/`. `tauri build` runs it
for you. Without a local checkout it clones `Txio-labs/txio` at `TXIO_FRONTEND_REF`
(default `main`). Set `TXIO_API_ORIGIN` at build time to point a build at another backend.

| Path | What it is |
| --- | --- |
| `src-tauri/src/lib.rs` | Window, navigation rules, menu, tray, plugins |
| `src-tauri/src/proxy.rs` | Serves the bundled app and forwards `/api/*` with the cookie jar |
| `src-tauri/src/nav.rs` | Which URLs load in the window, which open outside |
| `src-tauri/src/updates.rs` | Self-update from GitHub Releases |
| `scripts/` | Frontend build, release, package manifests, Linux repositories |
| `site/` | Download page and `install.sh`, published to GitHub Pages |
| `Casks/`, `bucket/` | Homebrew cask and Scoop manifest, written by each release |

## Releasing

Actions → **Cut release** → Run workflow, with `patch`, `minor`, `major` or an exact version.
It runs `scripts/release.py` (sets the version in `tauri.conf.json`, `package.json`,
`Cargo.toml`, `Cargo.lock` and this README), commits, tags `vX.Y.Z`, pushes, and starts
**Release**, which:

1. builds on Linux (deb, AppImage, rpm), macOS Apple silicon and Intel (dmg), and Windows
   (nsis, msi) with `tauri-action`, signs the update packages and uploads `latest.json`;
2. rewrites the release notes into a per-platform download table (`scripts/release_notes.py`);
3. updates the Homebrew cask and Scoop manifest on `main` (`scripts/package_manifests.py`);
4. starts **Download page**, which rebuilds the signed APT and RPM repositories and publishes `site/`.

Locally: `python3 scripts/release.py patch --push` does step 0 from your machine.

Repository secrets:

| Secret | Used for |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Signing update packages |
| `APT_SIGNING_KEY` | ASCII-armored GPG private key that signs the APT and RPM repositories |

Optional repository variable `TXIO_FRONTEND_REF` picks the web app branch or tag to bundle.
GitHub Pages must be set to deploy from **GitHub Actions**.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

MIT. See [LICENSE](LICENSE).
