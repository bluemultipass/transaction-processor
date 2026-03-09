# Transaction Processor — Use Cases

## Overview

A desktop application (Rust + Tauri + SQLite) for importing Chase bank transaction CSVs, filtering and aggregating spend, and producing formatted output for pasting into Google Sheets.

---

## Tech Stack

- **Rust** — core logic
- **Tauri** — desktop app shell
- **SQLite** — local persistence

---

## CSV Import

### Supported Formats

**Checking account (debit):**
```
Details,Posting Date,Description,Amount,Type,Balance,Check or Slip #
DEBIT,03/06/2026,"NETFLIX.COM",-17.99,ACH_DEBIT,999.99,,
```

**Credit card:**
```
Transaction Date,Post Date,Description,Category,Type,Amount,Memo
03/03/2026,03/03/2026,APPLE.COM/BILL,Shopping,Sale,-1.99,
```

### Multi-file Import

- Multiple CSVs for the same time period can be imported in a single session (Chase limits downloads to one account at a time)
- All imported files are merged into a unified transaction list before processing

### Deduplication

- Transactions have no unique IDs, so exact deduplication is not possible
- Best-effort deduplication based on a composite key: `(date, description, amount)` — transactions matching on all three fields within a single import session are treated as duplicates and only stored once
- The user is warned if potential duplicates are detected across separate import sessions

### Spend Filtering

Only transactions representing outgoing spend are imported into the database:
- Negative amounts on checking CSVs (debits)
- Negative amounts on credit card CSVs (sales/charges)
- Credits, refunds, and deposits are excluded

---

## Filters

Filters identify and group transactions by merchant or spend category.

### Defining a Filter

A filter is a named rule that matches transactions by description substring or pattern, e.g.:

| Filter Name     | Matches Description Containing |
|-----------------|--------------------------------|
| Netflix         | `NETFLIX`                      |
| Apple           | `APPLE.COM`                    |
| Foobar Market   | `FOOBAR MARKET`                |

### Saving Filters

- Filters are persisted in the local SQLite database
- Filters can be created, edited, and deleted from the UI

---

## Aggregation

For a given date range (or the set of currently uploaded transactions), the app can aggregate spend per filter:

- All transactions matching a filter within the date range are summed
- The result is a single row per filter showing total spend
- The "last date" of a matching transaction within the range is used as the row date

---

## Output: Google Sheets Copy Format

Each aggregated result produces a row in the following format:

```
<last date of spend>  <filter name>  <spend amount as positive value>
```

Example:
```
03/06/2026  Netflix         17.99
03/03/2026  Apple           1.99
03/05/2026  Foobar Market   143.50
```

- Values are tab-separated for direct paste into Google Sheets (one row per filter)
- The output is displayed in a copyable text area in the UI

### One-Click Full Report

- A single button runs all saved filters against the selected date range (or uploaded transactions) and produces the complete copyable output in one step

---

## Transaction Storage

The local SQLite database stores spend transactions with the following properties:

- **date** — posting/transaction date
- **description** — raw description from CSV
- **amount** — absolute (positive) spend amount
- **source_account** — which CSV file / account it came from
- **accounted** — boolean flag indicating whether this transaction has been included in a processed output report

The `accounted` flag allows the user to track which transactions have already been exported to Google Sheets and avoid double-counting across sessions.

---

## Workflows

### First-time Import

1. User opens the app
2. User drags in one or more Chase CSV files
3. App detects CSV format (checking vs. credit), filters to spend-only transactions, and deduplicates
4. Transactions are saved to the local SQLite database
5. User reviews imported transactions

### Producing a Report

1. User selects a date range (or uses all uploaded transactions)
2. User clicks "Generate Report" (runs all saved filters)
3. App aggregates spend per filter and displays the tab-separated output
4. User copies the output and pastes into Google Sheets
5. Transactions included in the report are marked as `accounted = true`

### Managing Filters

1. User navigates to the Filters screen
2. User creates, edits, or deletes named filters
3. Filters are immediately available for future report generation
