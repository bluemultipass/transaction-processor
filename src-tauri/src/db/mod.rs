pub mod filters;
pub mod settings;
pub mod transactions;

/// Serialize an i64 as a JSON number, returning a serde error if the value
/// exceeds Number.MAX_SAFE_INTEGER (2^53 − 1). Called via
/// `#[serde(serialize_with = "crate::db::serialize_i64_safe")]` on struct
/// fields that cross the Rust → JavaScript boundary.
pub fn serialize_i64_safe<S: serde::Serializer>(value: &i64, s: S) -> Result<S::Ok, S::Error> {
    const MAX_SAFE: i64 = 9_007_199_254_740_991; // 2^53 - 1
    if *value > MAX_SAFE {
        return Err(serde::ser::Error::custom(format!(
            "id {value} exceeds Number.MAX_SAFE_INTEGER ({MAX_SAFE})"
        )));
    }
    s.serialize_i64(*value)
}

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::path::Path;

use crate::error::AppError;

pub async fn init_db(db_path: &Path) -> Result<SqlitePool, AppError> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::Other(e.to_string()))?;
    }

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| AppError::Database(sqlx::Error::from(e)))?;

    Ok(pool)
}
