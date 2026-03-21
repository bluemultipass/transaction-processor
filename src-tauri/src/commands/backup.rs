use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;

use crate::db::backup;
use crate::error::AppError;

#[derive(Debug, Serialize, specta::Type)]
pub struct ImportDbResult {
    pub transactions: usize,
    pub filters: usize,
    pub settings: usize,
}

#[tauri::command]
#[specta::specta]
pub async fn export_db(state: State<'_, SqlitePool>, path: String) -> Result<(), AppError> {
    let data = backup::export_all(&state).await?;
    let json = serde_json::to_string_pretty(&data).map_err(|e| AppError::Other(e.to_string()))?;
    std::fs::write(&path, json).map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn import_db(
    state: State<'_, SqlitePool>,
    path: String,
) -> Result<ImportDbResult, AppError> {
    let content = std::fs::read_to_string(&path).map_err(|e| AppError::Other(e.to_string()))?;
    let data: backup::BackupData = serde_json::from_str(&content).map_err(|_| {
        AppError::Other("The selected file is not a valid Ledger backup.".to_string())
    })?;

    if data.version != 1 {
        return Err(AppError::Other(format!(
            "Unsupported backup version: {}. This version of Ledger supports version 1.",
            data.version
        )));
    }

    let result = ImportDbResult {
        transactions: data.transactions.len(),
        filters: data.filters.len(),
        settings: data.settings.len(),
    };

    backup::import_all(&state, data).await?;

    Ok(result)
}
