#!/bin/sh
# Install Txio on Linux with one command:
#
#   curl -fsSL https://txio-labs.github.io/txio-desktop/install.sh | sudo sh
#
# Debian, Ubuntu, Kali (apt) and Fedora, RHEL, openSUSE (dnf, yum, zypper): adds Txio's signed
# package repository once, then installs Txio. After that your package manager installs and
# updates it like any other package (sudo apt upgrade, sudo dnf upgrade, sudo zypper update).
# Any other distribution: installs the AppImage to /opt/txio, which updates itself.
set -eu

BASE="https://txio-labs.github.io/txio-desktop"
KEYRING="/usr/share/keyrings/txio.gpg"

if [ "$(id -u)" -ne 0 ]; then
  echo "Run this with sudo: curl -fsSL $BASE/install.sh | sudo sh" >&2
  exit 1
fi
case "$(uname -m)" in
  x86_64|amd64) ;;
  *) echo "Txio's Linux builds are for 64-bit Intel/AMD (x86_64) only." >&2; exit 1 ;;
esac

if command -v apt-get >/dev/null 2>&1; then
  echo "Adding Txio's APT repository..."
  install -d -m 0755 /usr/share/keyrings
  curl -fsSL "$BASE/txio.gpg" -o "$KEYRING.tmp" && mv "$KEYRING.tmp" "$KEYRING" && chmod 0644 "$KEYRING"
  echo "deb [arch=amd64 signed-by=$KEYRING] $BASE/apt stable main" > /etc/apt/sources.list.d/txio.list
  apt-get update
  apt-get install -y txio
  echo "Done. Updates arrive with: sudo apt upgrade"
elif command -v dnf >/dev/null 2>&1 || command -v yum >/dev/null 2>&1; then
  echo "Adding Txio's RPM repository..."
  curl -fsSL "$BASE/txio.repo" -o /etc/yum.repos.d/txio.repo
  if command -v dnf >/dev/null 2>&1; then dnf install -y txio; else yum install -y txio; fi
  echo "Done. Updates arrive with: sudo dnf upgrade"
elif command -v zypper >/dev/null 2>&1; then
  echo "Adding Txio's RPM repository..."
  zypper --non-interactive --gpg-auto-import-keys addrepo --refresh "$BASE/txio.repo"
  zypper --non-interactive --gpg-auto-import-keys install txio
  echo "Done. Updates arrive with: sudo zypper update"
else
  echo "No apt, dnf, yum or zypper found; installing the AppImage..."
  url="$(curl -fsSL https://api.github.com/repos/Txio-labs/txio-desktop/releases/latest | grep -o 'https://[^"]*_amd64\.AppImage' | head -n 1)"
  [ -n "$url" ] || { echo "Could not find the latest AppImage." >&2; exit 1; }
  install -d -m 0755 /opt/txio
  curl -fL "$url" -o /opt/txio/Txio.AppImage
  chmod 0755 /opt/txio/Txio.AppImage
  chown -R "${SUDO_USER:-root}" /opt/txio  # lets the AppImage update itself without sudo
  ln -sf /opt/txio/Txio.AppImage /usr/local/bin/txio
  cat > /usr/share/applications/txio.desktop <<EOF
[Desktop Entry]
Name=Txio
Comment=One console for every chain
Exec=/opt/txio/Txio.AppImage
Terminal=false
Type=Application
Categories=Development;
EOF
  echo "Done. Run 'txio' or open Txio from your applications menu. It updates itself."
fi
