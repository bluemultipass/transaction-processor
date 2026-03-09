# Transaction Processor — Tech Spec

## Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri v2 |
| Frontend | SolidJS + TypeScript |
| Backend | Rust |
| Database | SQLite via sqlx |
| Package manager | pnpm |

---

## Project Structure

```
transaction-processor/
├── src/                        # SolidJS frontend
│   ├── components/
│   ├── screens/
│   ├── store/                  # Global SolidJS context/stores
│   └── main.tsx
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/           # Tauri command handlers, grouped by domain
│   │   │   ├── transactions.rs
│   │   │   ├── filters.rs
│   │   │   └── reports.rs
│   │   ├── db/                 # sqlx queries and types
│   │   ├── csv/                # CSV parsing logic
│   │   └── error.rs            # AppError
│   ├── migrations/             # sqlx migration files
│   ├── capabilities/           # Tauri v2 permission config
│   └── Cargo.toml
├── docs/
├── .husky/
├── rust-toolchain.toml
├── package.json
└── tsconfig.json
```

---

## Rust Toolchain

Pin the stable channel via `rust-toolchain.toml`:

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

---

## Error Handling

All Tauri commands return `Result<T, AppError>`. A single `AppError` type covers all backend error cases, serialized as a string for the frontend.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("csv parse error: {0}")]
    Csv(String),
    #[error("{0}")]
    Other(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
```

The frontend receives either the success value or an error string. No error codes or structured error objects — the app is simple enough that a message is sufficient.

---

## Type Safety Across the Tauri Boundary

Use **tauri-specta** to auto-generate TypeScript types and typed command bindings from Rust. This eliminates hand-written bindings and catches frontend/backend type mismatches at compile time.

- All Tauri commands are registered through tauri-specta
- Generated bindings are committed to the repo so diffs are reviewable
- The frontend imports commands from the generated file rather than calling `invoke` directly

---

## Database

### Location

The SQLite file lives in the OS-appropriate app data directory via Tauri's `app_data_dir()` resolver:

| OS | Path |
|----|------|
| Windows | `%APPDATA%\transaction-processor\db.sqlite` |
| macOS | `~/Library/Application Support/transaction-processor/db.sqlite` |
| Linux | `~/.local/share/transaction-processor/db.sqlite` |

### Migrations

- Managed by sqlx's built-in migration runner
- Migration files live in `src-tauri/migrations/`, named `{timestamp}_{description}.sql`
- Migrations run automatically at app startup before any commands are available
- Use `sqlx migrate add <description>` to create new migrations

### Queries

Raw sqlx — no ORM. Queries live in `src/db/`, organized by domain (transactions, filters). Use `sqlx::query_as!` with compile-time checked queries where possible.

---

## Frontend

### TypeScript

Max strictness `tsconfig.json`:

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride": true,
    "exactOptionalPropertyTypes": true,
    "noPropertyAccessFromIndexSignature": true,
    "jsx": "preserve",
    "jsxImportSource": "solid-js"
  }
}
```

### Linting and Formatting

- **ESLint** with:
  - `typescript-eslint` — strict + strictTypeChecked presets
  - `eslint-plugin-solid` — SolidJS-specific rules (reactive usage, prop destructuring, etc.)
  - `eslint-config-prettier` — disables ESLint formatting rules that conflict with Prettier
- **Prettier** for formatting

### State Management

SolidJS primitives only — no external state library.

- Component-local state: `createSignal`, `createMemo`
- App-wide state (transactions, filters, active date range): `createContext` + `createStore`, provided at the app root
- No prop drilling beyond one level — lift to context instead

### Build Tool

Vite (standard for Tauri v2, configured by `create-tauri-app`).

---

## Testing

### Rust

Inline `#[cfg(test)]` modules in the same file as the code under test. For tests that require a database, use an in-memory SQLite connection (`sqlite::memory:`).

Integration tests (e.g., full command handler tests) go in `src-tauri/tests/`.

### Frontend

```
vitest
jsdom
@solidjs/testing-library
@testing-library/user-event
@testing-library/jest-dom
```

Test files colocated with components (`*.test.tsx`). Focus tests on behavior, not implementation — render a component, simulate user events, assert on output.

---

## Logging

Use the `tracing` crate in Rust. Tauri v2 integrates with `tracing` via the log bridge. Logs go to:

| OS | Location |
|----|----------|
| Windows | `%APPDATA%\transaction-processor\logs\` |
| macOS | `~/Library/Logs/transaction-processor/` |
| Linux | `~/.local/share/transaction-processor/logs/` |

Log level defaults to `info` in release, `debug` in development.

---

## Tauri v2 Capabilities

Tauri v2 uses a `capabilities/` directory to grant the frontend access to APIs. Grant only what is needed:

- `fs:allow-read` / `fs:allow-write` — scoped to app data dir and user-selected files (CSV import via dialog)
- `dialog:allow-open` — for the CSV file picker
- No shell, HTTP, or other access needed

---

## Pre-commit Hooks

Managed by Husky. Runs on every commit:

**Rust:**
- `cargo fmt --check`
- `cargo clippy -- -D warnings`

**Frontend:**
- `eslint --max-warnings 0`
- `prettier --check`

---

## CI

GitHub Actions. Three workflows:

### PR checks (`pr.yml`)
Runs on every pull request:
- Rust: `cargo fmt --check`, `cargo clippy`, `cargo test`
- Frontend: `eslint`, `prettier --check`, `pnpm typecheck`, `vitest run`

### Main branch (`main.yml`)
Same checks as PR, runs on push to `main`.

### Release (`release.yml`)
Triggers on version tag push (`v*`). Builds and uploads installers for:
- Windows (`.msi`)
- macOS (`.dmg`)
- Linux (`.AppImage`)

No code signing — personal use only.

---

## Distribution

Personal use. No code signing or notarization. Install by running the platform-appropriate installer from the release artifacts.
