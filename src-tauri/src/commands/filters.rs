use sqlx::SqlitePool;
use tauri::State;

use crate::db::filters::{self, Filter};
use crate::error::AppError;

#[tauri::command]
#[specta::specta]
pub async fn list_filters(state: State<'_, SqlitePool>) -> Result<Vec<Filter>, AppError> {
    filters::list_filters(&state).await
}

#[tauri::command]
#[specta::specta]
pub async fn create_filter(
    state: State<'_, SqlitePool>,
    name: String,
    pattern: String,
) -> Result<Filter, AppError> {
    filters::create_filter(&state, &name, &pattern).await
}

#[tauri::command]
#[specta::specta]
pub async fn update_filter(
    state: State<'_, SqlitePool>,
    id: i64,
    name: String,
    pattern: String,
) -> Result<Filter, AppError> {
    filters::update_filter(&state, id, &name, &pattern).await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_filter(state: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    filters::delete_filter(&state, id).await
}
