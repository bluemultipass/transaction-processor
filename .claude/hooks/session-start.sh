#!/bin/bash
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

echo "Installing Node dependencies..."
pnpm install

echo "Verifying Rust toolchain..."
rustup show active-toolchain > /dev/null 2>&1 || rustup install stable

echo "Session start setup complete."
