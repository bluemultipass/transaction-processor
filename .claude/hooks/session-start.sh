#!/bin/bash
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

echo "Installing Node dependencies..."
pnpm install

echo "Verifying Rust toolchain..."
rustup show active-toolchain > /dev/null 2>&1 || rustup install stable

echo "Installing Tauri system dependencies..."
if command -v apt-get &>/dev/null; then
  apt-get install -y --no-install-recommends \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    patchelf \
    2>/dev/null || true
fi

echo "Session start setup complete."
