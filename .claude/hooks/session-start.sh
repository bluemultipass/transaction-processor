#!/bin/bash
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

echo "Installing Node dependencies..."
pnpm install

echo "Verifying Rust toolchain..."
if ! rustup show active-toolchain > /dev/null 2>&1; then
  echo "ERROR: Rust toolchain not found. Install rustup from https://rustup.rs" >&2
  exit 1
fi

echo "Session start setup complete."
