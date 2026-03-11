use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;
use tauri::State;

use crate::csv;
use crate::db::transactions::{self, Transaction};
use crate::error::AppError;

/// A parsed transaction returned to the frontend for user review before insertion.
#[derive(Debug, Serialize, Deserialize, Clone, specta::Type)]
pub struct PendingTransaction {
    pub date: String,
    pub description: String,
    pub amount: f64,
    pub is_possible_duplicate: bool,
}

#[derive(Debug, Serialize, specta::Type)]
pub struct PreviewResult {
    pub transactions: Vec<PendingTransaction>,
}

#[derive(Debug, Serialize, specta::Type)]
pub struct ImportResult {
    pub imported: usize,
}

/// Parse CSV files and detect potential duplicates against existing DB rows.
/// Does NOT insert anything — the frontend confirms which rows to keep.
#[tauri::command]
#[specta::specta]
pub async fn preview_import(
    state: State<'_, SqlitePool>,
    paths: Vec<String>,
) -> Result<PreviewResult, AppError> {
    let mut all_transactions = Vec::new();
    for path in &paths {
        let parsed = csv::parse_transactions(std::path::Path::new(path))?;
        all_transactions.extend(parsed);
    }

    let dup_keys: HashSet<(String, String, String)> = if all_transactions.is_empty() {
        HashSet::new()
    } else {
        let min_date = all_transactions
            .iter()
            .map(|t| t.date.as_str())
            .min()
            .unwrap();
        let max_date = all_transactions
            .iter()
            .map(|t| t.date.as_str())
            .max()
            .unwrap();
        let existing = transactions::list_transactions_in_range(&state, min_date, max_date).await?;
        existing
            .into_iter()
            .map(|t| (t.date, t.description, format!("{:.2}", t.amount)))
            .collect()
    };

    let pending = all_transactions
        .into_iter()
        .map(|t| {
            let is_possible_duplicate = dup_keys.contains(&(
                t.date.clone(),
                t.description.clone(),
                format!("{:.2}", t.amount),
            ));
            PendingTransaction {
                date: t.date,
                description: t.description,
                amount: t.amount,
                is_possible_duplicate,
            }
        })
        .collect();

    Ok(PreviewResult {
        transactions: pending,
    })
}

/// Insert the transactions the user confirmed (i.e. not marked as duplicates).
#[tauri::command]
#[specta::specta]
pub async fn confirm_import(
    state: State<'_, SqlitePool>,
    transactions: Vec<PendingTransaction>,
) -> Result<ImportResult, AppError> {
    let imported = transactions.len();
    let parsed: Vec<csv::ParsedTransaction> = transactions
        .into_iter()
        .map(|t| csv::ParsedTransaction {
            date: t.date,
            description: t.description,
            amount: t.amount,
        })
        .collect();
    transactions::insert_transactions(&state, parsed).await?;
    Ok(ImportResult { imported })
}

#[tauri::command]
#[specta::specta]
pub async fn list_transactions(
    state: State<'_, SqlitePool>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<Vec<Transaction>, AppError> {
    transactions::list_transactions(&state, date_from.as_deref(), date_to.as_deref()).await
}
