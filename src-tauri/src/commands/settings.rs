use sqlx::SqlitePool;
use tauri::State;

use crate::db::settings;
use crate::error::AppError;

#[tauri::command]
#[specta::specta]
pub async fn get_split_count(state: State<'_, SqlitePool>) -> Result<i64, AppError> {
    settings::get_split_count(&state).await
}

#[tauri::command]
#[specta::specta]
pub async fn set_split_count(state: State<'_, SqlitePool>, count: i64) -> Result<(), AppError> {
    settings::set_split_count(&state, count).await
}
