use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;

use crate::csv;
use crate::db::transactions::{self, Transaction};
use crate::error::AppError;

#[derive(Debug, Serialize, specta::Type)]
pub struct ImportResult {
    pub imported: usize,
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
    transactions::insert_transactions(&state, all_transactions).await?;
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
