# Claude Code Instructions

## Uncertainty

When unsure about a framework API, library behavior, or implementation detail — say so explicitly before writing code. Do not guess and do not silently fall back to patterns from similar frameworks (e.g., do not apply React patterns to Solid.js). Flag the uncertainty and either ask or look it up first.

## Pre-commit Hooks

Run the following after initializing the project to install the pre-commit hooks:

```sh
pnpm install
```

Husky is configured to install automatically via the `prepare` script in `package.json`. No separate hook installation step is needed.
