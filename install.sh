#!/usr/bin/env bash
set -e

# ⚡ pnuke installer script
# Repository: https://github.com/almostordinary/pnuke

REPO="almostordinary/pnuke"
INSTALL_DIR="/usr/local/bin"

echo "⚡ Installing pnuke..."

# If cargo is installed, build from source
if command -v cargo >/dev/null 2>&1; then
    echo "✔ Found Cargo. Installing via cargo install..."
    cargo install --git "https://github.com/${REPO}.git" --bin pnuke
    echo "✔ pnuke installed successfully! Run 'pnuke --help' to get started."
    exit 0
fi

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
esac

echo "Detected OS: $OS ($ARCH)"
echo "Visit https://github.com/${REPO}/releases for pre-built release binaries."
