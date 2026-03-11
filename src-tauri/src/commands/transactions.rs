use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;

use crate::csv;
use crate::db::transactions::{self, Transaction};
use crate::error::AppError;

#[derive(Debug, Serialize, specta::Type)]
pub struct ImportResult {
    pub imported: usize,
    pub possible_duplicates: Vec<Transaction>,
}

#[tauri::command]
#[specta::specta]
pub async fn import_transactions(
    state: State<'_, SqlitePool>,
    paths: Vec<String>,
) -> Result<ImportResult, AppError> {
    let mut all_transactions = Vec::new();
    for path in &paths {
        let parsed = csv::parse_transactions(std::path::Path::new(path))?;
        all_transactions.extend(parsed);
    }
    let imported = all_transactions.len();

    let possible_duplicates = if all_transactions.is_empty() {
        Vec::new()
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
            .filter(|db_tx| {
                all_transactions.iter().any(|incoming| {
                    incoming.date == db_tx.date
                        && incoming.description == db_tx.description
                        && (incoming.amount - db_tx.amount).abs() < 1e-9
                })
            })
            .collect()
    };

    transactions::insert_transactions(&state, all_transactions).await?;
    Ok(ImportResult {
        imported,
        possible_duplicates,
    })
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
