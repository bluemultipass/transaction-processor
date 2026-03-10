#!/bin/bash
# Setup script for Claude Code on the web.
# Runs as root before Claude Code launches, on new sessions only.
# Install system-level dependencies that aren't in the default cloud image.

apt-get update && apt-get install -y --no-install-recommends \
  gh \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libsoup-3.0-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf
