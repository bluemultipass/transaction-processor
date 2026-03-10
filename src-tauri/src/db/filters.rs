use serde::Serialize;
use sqlx::{Row, SqlitePool};

use crate::error::AppError;

#[derive(Debug, Serialize, specta::Type)]
pub struct Filter {
    #[serde(serialize_with = "crate::db::serialize_i64_safe")]
    pub id: i64,
    pub name: String,
    pub pattern: String,
}

pub async fn list_filters(pool: &SqlitePool) -> Result<Vec<Filter>, AppError> {
    let rows = sqlx::query("SELECT id, name, pattern FROM filters ORDER BY name ASC")
        .fetch_all(pool)
        .await?;

    rows.into_iter()
        .map(|row| {
            Ok(Filter {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                pattern: row.try_get("pattern")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(AppError::Database)
}

pub async fn create_filter(
    pool: &SqlitePool,
    name: &str,
    pattern: &str,
) -> Result<Filter, AppError> {
    let id = sqlx::query("INSERT INTO filters (name, pattern) VALUES (?, ?)")
        .bind(name)
        .bind(pattern)
        .execute(pool)
        .await?
        .last_insert_rowid();

    Ok(Filter {
        id,
        name: name.to_string(),
        pattern: pattern.to_string(),
    })
}

pub async fn update_filter(
    pool: &SqlitePool,
    id: i64,
    name: &str,
    pattern: &str,
) -> Result<Filter, AppError> {
    let rows_affected = sqlx::query("UPDATE filters SET name = ?, pattern = ? WHERE id = ?")
        .bind(name)
        .bind(pattern)
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::Other(format!("filter with id {id} not found")));
    }

    Ok(Filter {
        id,
        name: name.to_string(),
        pattern: pattern.to_string(),
    })
}

pub async fn delete_filter(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    let rows_affected = sqlx::query("DELETE FROM filters WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::Other(format!("filter with id {id} not found")));
    }

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
            "CREATE TABLE filters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                pattern TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn create_and_list_filters() {
        let pool = setup_db().await;
        create_filter(&pool, "Groceries", "WHOLEFDS").await.unwrap();
        create_filter(&pool, "Coffee", "STARBUCKS").await.unwrap();

        let filters = list_filters(&pool).await.unwrap();
        assert_eq!(filters.len(), 2);
        // ordered by name ascending
        assert_eq!(filters[0].name, "Coffee");
        assert_eq!(filters[1].name, "Groceries");
    }

    #[tokio::test]
    async fn create_filter_returns_inserted_row() {
        let pool = setup_db().await;
        let f = create_filter(&pool, "Rent", "LANDLORD").await.unwrap();
        assert_eq!(f.name, "Rent");
        assert_eq!(f.pattern, "LANDLORD");
        assert!(f.id > 0);
    }

    #[tokio::test]
    async fn update_filter_changes_fields() {
        let pool = setup_db().await;
        let f = create_filter(&pool, "Old Name", "OLD").await.unwrap();
        let updated = update_filter(&pool, f.id, "New Name", "NEW").await.unwrap();
        assert_eq!(updated.id, f.id);
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.pattern, "NEW");

        let filters = list_filters(&pool).await.unwrap();
        assert_eq!(filters[0].name, "New Name");
    }

    #[tokio::test]
    async fn update_filter_missing_id_returns_error() {
        let pool = setup_db().await;
        let result = update_filter(&pool, 999, "X", "Y").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_filter_removes_row() {
        let pool = setup_db().await;
        let f = create_filter(&pool, "To Delete", "DEL").await.unwrap();
        delete_filter(&pool, f.id).await.unwrap();

        let filters = list_filters(&pool).await.unwrap();
        assert!(filters.is_empty());
    }

    #[tokio::test]
    async fn delete_filter_missing_id_returns_error() {
        let pool = setup_db().await;
        let result = delete_filter(&pool, 999).await;
        assert!(result.is_err());
    }
}
