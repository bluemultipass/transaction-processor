# Claude Code Instructions

## System packages

If a task requires installing additional system packages (e.g. via `apt`, `brew`, or any system package manager), do not install them. Instead, stop and tell the user what package is needed so they can add it to the setup script in the Claude Code web interface.

## Uncertainty

When unsure about a framework API, library behavior, or implementation detail — say so explicitly before writing code. Do not guess and do not silently fall back to patterns from similar frameworks (e.g., do not apply React patterns to Solid.js). Flag the uncertainty and either ask or look it up first.

