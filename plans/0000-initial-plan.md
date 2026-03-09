# Implementation Plan: Transaction Processor

## Overview

Build a Tauri v2 + SolidJS + SQLite desktop app from scratch. The project currently has only docs and a README. Steps are ordered to minimize risk, with each step testable in isolation before moving on.

---

## Step 1 — Scaffold Tauri v2 + SolidJS project

**Goal:** Get a "Hello World" app that compiles and runs.

Files to create:
- `package.json` — pnpm workspace, vite/tauri deps
- `vite.config.ts` — standard Tauri+Vite config
- `index.html` — entry HTML
- `src/main.tsx` — SolidJS root render
- `src/App.tsx` — placeholder App component
- `src-tauri/Cargo.toml` — with tauri, tauri-build deps
- `src-tauri/build.rs` — tauri build script
- `src-tauri/tauri.conf.json` — app identifier, window config, bundle targets (nsis)
- `src-tauri/src/main.rs` — minimal Tauri app setup

**Done when:** `pnpm tauri dev` launches a window with "Hello World".

---

## Step 2 — Tooling: rust-toolchain, tsconfig, ESLint, Prettier, Husky

**Goal:** All linters/formatters pass on a clean repo; pre-commit hook runs them; Claude Code session start hook ensures the dev environment is ready.

Files to create/configure:
- `rust-toolchain.toml` — pin stable, rustfmt, clippy
- `tsconfig.json` — strict TypeScript as specified in tech-spec
- `.eslintrc.cjs` — typescript-eslint strict + strictTypeChecked, eslint-plugin-solid, eslint-config-prettier
- `.prettierrc` — minimal config
- `.husky/pre-commit` — runs cargo fmt --check, cargo clippy -D warnings, eslint, prettier --check
- `package.json` scripts: `typecheck`, `lint`, `format:check`
- `.claude/hooks/session-start.sh` — Claude Code SessionStart hook that installs dependencies (`pnpm install`) and verifies the Rust toolchain is present, so Claude Code web sessions start in a known-good state

**Done when:** `pnpm lint`, `pnpm typecheck`, `cargo fmt --check`, `cargo clippy` all pass; Claude Code session start hook runs without error.

---

## Step 3 — Database: sqlx setup + migrations

**Goal:** SQLite DB initializes at app startup with the right schema.

Changes:
- Add `sqlx`, `tokio` to `src-tauri/Cargo.toml`
- Create `src-tauri/migrations/20260309000001_initial.sql`:
  ```sql
  CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    description TEXT NOT NULL,
    amount REAL NOT NULL,
    source_account TEXT NOT NULL,
    accounted INTEGER NOT NULL DEFAULT 0
  );

  CREATE TABLE filters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    pattern TEXT NOT NULL
  );
  ```
- Create `src-tauri/src/db/mod.rs` — `init_db()` function: opens SQLite at `app_data_dir/db.sqlite`, runs `sqlx::migrate!()`
- Wire `init_db()` into `main.rs` before Tauri builder

**Done when:** App starts without error and `db.sqlite` is created at the correct path.

---

## Step 4 — AppError + tauri-specta wiring

**Goal:** Typed command boundary is established; frontend will import generated bindings.

Changes:
- Add `thiserror`, `tauri-specta`, `specta`, `specta-typescript` to Cargo.toml
- Create `src-tauri/src/error.rs` — `AppError` enum as specified in tech-spec (Database, Csv, Other variants + Serialize impl)
- Create `src-tauri/src/commands/mod.rs` — empty for now, registers command collection with tauri-specta
- In `main.rs`: set up tauri-specta `Builder`, export TS bindings to `src/bindings.ts` on build
- Create `src/bindings.ts` placeholder (will be overwritten by specta export)

**Done when:** `cargo build` generates `src/bindings.ts` with no errors.

---

## Step 5 — CSV parsing logic

**Goal:** Parse both Chase CSV formats; filter to spend-only; deduplicate within session.

Files:
- `src-tauri/src/csv/mod.rs`:
  - `detect_format(headers: &[&str]) -> CsvFormat` (Checking | CreditCard)
  - `parse_transactions(path: &Path, source_account: &str) -> Result<Vec<ParsedTransaction>, AppError>`
  - Filters: checking keeps negative Amount; credit keeps negative Amount (Sale type)
  - Dedup within a Vec: retain only first occurrence per `(date, description, amount)` composite key
- `ParsedTransaction` struct: `{ date, description, amount (absolute f64), source_account }`
- Inline `#[cfg(test)]` with sample CSV strings for both formats

**Done when:** `cargo test` passes with unit tests covering both formats, spend filtering, and dedup.

---

## Step 6 — Transaction commands

**Goal:** Frontend can import CSV files and list stored transactions.

Files:
- `src-tauri/src/db/transactions.rs`:
  - `insert_transactions(pool, Vec<ParsedTransaction>) -> Result<(), AppError>`
  - `list_transactions(pool, date_from: Option<&str>, date_to: Option<&str>) -> Result<Vec<Transaction>, AppError>`
  - `mark_accounted(pool, ids: &[i64]) -> Result<(), AppError>`
  - `Transaction` struct (all DB columns) — `#[derive(Serialize, specta::Type)]`
- `src-tauri/src/commands/transactions.rs`:
  - `import_transactions(paths: Vec<String>) -> Result<ImportResult, AppError>`
    - Parse all files, merge, deduplicate across files (same composite key logic)
    - Warn on duplicates found in DB from prior sessions (query existing, compare)
    - Store new ones
    - Returns `ImportResult { imported: usize, duplicates_skipped: usize, cross_session_warnings: usize }`
  - `list_transactions(date_from: Option<String>, date_to: Option<String>) -> Result<Vec<Transaction>, AppError>`
- Register both commands with tauri-specta, regenerate `src/bindings.ts`

**Done when:** `cargo test` passes; bindings regenerated with new command types.

---

## Step 7 — Filter commands

**Goal:** Frontend can CRUD filters.

Files:
- `src-tauri/src/db/filters.rs`:
  - `Filter` struct `{ id, name, pattern }` — `#[derive(Serialize, specta::Type)]`
  - `list_filters`, `create_filter`, `update_filter`, `delete_filter`
- `src-tauri/src/commands/filters.rs`:
  - `list_filters() -> Result<Vec<Filter>, AppError>`
  - `create_filter(name: String, pattern: String) -> Result<Filter, AppError>`
  - `update_filter(id: i64, name: String, pattern: String) -> Result<Filter, AppError>`
  - `delete_filter(id: i64) -> Result<(), AppError>`
- Register with tauri-specta, regenerate bindings

**Done when:** `cargo test` passes with unit tests for DB filter operations.

---

## Step 8 — Report command

**Goal:** Frontend can generate the tab-separated Google Sheets output.

Files:
- `src-tauri/src/commands/reports.rs`:
  - `generate_report(date_from: Option<String>, date_to: Option<String>) -> Result<ReportOutput, AppError>`
    - Load all filters
    - For each filter: query transactions matching `LIKE '%{pattern}%'` in the date range, sum amounts, find max date
    - Collect `ReportRow { filter_name, last_date, total_amount }`
    - Mark matched transaction IDs as `accounted = true`
    - Return `ReportOutput { rows: Vec<ReportRow>, text: String }` where `text` is the tab-separated string
  - `ReportRow` and `ReportOutput` structs — `#[derive(Serialize, specta::Type)]`
- Register with tauri-specta, regenerate bindings

**Done when:** `cargo test` passes with an integration test using in-memory SQLite.

---

## Step 9 — Frontend: global store + app shell

**Goal:** SolidJS context wires up global state; basic screen navigation works.

Files:
- `src/store/AppStore.tsx` — `createContext` + `createStore` with:
  - `transactions: Transaction[]`
  - `filters: Filter[]`
  - `dateFrom: string | null`, `dateTo: string | null`
  - `reportOutput: string | null`
  - Actions: `loadTransactions`, `loadFilters`, `setDateRange`, `setReportOutput`
- `src/App.tsx` — provides AppStore context; renders nav tabs + active screen
- `src/components/Nav.tsx` — tab bar: "Transactions", "Filters", "Report"
- Three placeholder screen components: `src/screens/TransactionsScreen.tsx`, `FiltersScreen.tsx`, `ReportScreen.tsx`

**Done when:** App launches, three tabs are visible, clicking switches screens.

---

## Step 10 — Frontend: Transactions screen (import + list)

**Goal:** User can import CSVs and see the transaction list.

Files:
- `src/screens/TransactionsScreen.tsx`:
  - "Import CSV" button → calls Tauri `dialog.open({ multiple: true, filters: [{ name: 'CSV', extensions: ['csv'] }] })` → calls `importTransactions` command → reloads transaction list
  - Shows import result summary (imported count, warnings)
  - Renders a table of transactions: date, description, amount, source_account, accounted badge
  - Date range inputs (from/to) update store, trigger reload
- `src/components/TransactionTable.tsx` — pure table component

**Done when:** File picker opens, CSV import works end-to-end, transactions display in table.

---

## Step 11 — Frontend: Filters screen (CRUD)

**Goal:** User can create, edit, and delete filters.

Files:
- `src/screens/FiltersScreen.tsx`:
  - Lists all filters (name + pattern)
  - "Add Filter" form: name + pattern text inputs → calls `createFilter`
  - Edit inline or via modal: calls `updateFilter`
  - Delete button per row: calls `deleteFilter`
- `src/components/FilterForm.tsx` — reusable form for create/edit

**Done when:** Full CRUD cycle works for filters.

---

## Step 12 — Frontend: Report screen

**Goal:** User can generate and copy the report output.

Files:
- `src/screens/ReportScreen.tsx`:
  - Date range inputs (shared from store)
  - "Generate Report" button → calls `generateReport` → stores output in store
  - Displays `<textarea readonly>` with the tab-separated output
  - "Copy to Clipboard" button
  - Shows per-row breakdown (filter name, last date, total) above the copyable text

**Done when:** Report generates correctly and text is copyable.

---

## Step 13 — Tauri capabilities config

**Goal:** Security permissions are minimal and explicit.

Files:
- `src-tauri/capabilities/default.json`:
  - `dialog:allow-open` (for CSV file picker)
  - `fs:allow-read` scoped to user-selected files
  - `fs:allow-read` + `fs:allow-write` scoped to app data dir (for SQLite)

**Done when:** App still works end-to-end after capability restrictions are in place.

---

## Step 14 — Frontend tests

**Goal:** Key behaviors are covered by Vitest component tests.

Test files (colocated):
- `src/components/TransactionTable.test.tsx` — renders rows correctly
- `src/screens/FiltersScreen.test.tsx` — add/delete filter interactions
- `src/screens/ReportScreen.test.tsx` — generate button calls command, output displays

Setup:
- Add `vitest`, `jsdom`, `@solidjs/testing-library`, `@testing-library/user-event`, `@testing-library/jest-dom` to devDependencies
- `vite.config.ts` test config (environment: jsdom)
- Mock `src/bindings.ts` Tauri commands in tests

**Done when:** `pnpm test` passes all vitest tests.

---

## Step 15 — CI workflows

**Goal:** GitHub Actions runs all checks on PR and push to main.

Files:
- `.github/workflows/pr.yml` — checkout, install rust+node+pnpm, run: cargo fmt --check, cargo clippy, cargo test, eslint, prettier --check, pnpm typecheck, vitest run
- `.github/workflows/main.yml` — same, triggers on push to main
- `.github/workflows/release.yml` — triggers on `v*` tags, uses `tauri-action` to build NSIS/dmg/AppImage

**Done when:** Workflows are syntactically valid YAML; local checks all pass.

---

## Sequencing Summary

```
1. Scaffold       → compiles + runs
2. Tooling        → linters pass
3. Database       → schema initialized
4. Error/Specta   → typed boundary established
5. CSV parsing    → unit tested in isolation
6. Tx commands    → backend import + list working
7. Filter cmds    → backend CRUD working
8. Report cmd     → backend aggregation working
9. App shell      → navigation works
10. Tx screen     → import + table end-to-end
11. Filters screen → CRUD end-to-end
12. Report screen  → generate + copy end-to-end
13. Capabilities  → permissions locked down
14. FE tests      → vitest suite passing
15. CI            → workflows committed
```

Each step is self-contained: the backend steps (5–8) can be verified with `cargo test` alone before touching the frontend. Steps 9–12 build the UI incrementally so each screen can be manually verified before moving on.
