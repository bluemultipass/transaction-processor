use sqlx::{Row, SqlitePool};

use crate::error::AppError;

pub async fn get_split_count(pool: &SqlitePool) -> Result<i64, AppError> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = 'split_count'")
        .fetch_one(pool)
        .await?;
    let value: String = row.try_get("value").map_err(AppError::Database)?;
    value
        .parse::<i64>()
        .map_err(|e| AppError::Other(e.to_string()))
}

pub async fn set_split_count(pool: &SqlitePool, count: i64) -> Result<(), AppError> {
    if count < 1 {
        return Err(AppError::Other(
            "split_count must be at least 1".to_string(),
        ));
    }
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('split_count', ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(count.to_string())
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqliteConnectOptions;

    async fn setup_db() -> SqlitePool {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE settings (
                key   TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO settings (key, value) VALUES ('split_count', '2')")
            .execute(&pool)
            .await
            .unwrap();
        pool
    }

    #[tokio::test]
    async fn get_returns_default() {
        let pool = setup_db().await;
        let count = get_split_count(&pool).await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn set_and_get_roundtrip() {
        let pool = setup_db().await;
        set_split_count(&pool, 4).await.unwrap();
        let count = get_split_count(&pool).await.unwrap();
        assert_eq!(count, 4);
    }

    #[tokio::test]
    async fn set_rejects_zero() {
        let pool = setup_db().await;
        let result = set_split_count(&pool, 0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn set_rejects_negative() {
        let pool = setup_db().await;
        let result = set_split_count(&pool, -1).await;
        assert!(result.is_err());
    }
}
