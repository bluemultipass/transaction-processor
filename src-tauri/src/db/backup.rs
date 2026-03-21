use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
    pub version: u32,
    pub exported_at: String,
    pub transactions: Vec<BackupTransaction>,
    pub filters: Vec<BackupFilter>,
    pub settings: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupTransaction {
    pub date: String,
    pub description: String,
    pub amount: f64,
    pub accounted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupFilter {
    pub name: String,
    pub pattern: String,
}

pub async fn export_all(pool: &SqlitePool) -> Result<BackupData, AppError> {
    let tx_rows = sqlx::query(
        "SELECT date, description, amount, accounted FROM transactions ORDER BY date DESC",
    )
    .fetch_all(pool)
    .await?;

    let transactions = tx_rows
        .into_iter()
        .map(|row| {
            Ok(BackupTransaction {
                date: row.try_get("date")?,
                description: row.try_get("description")?,
                amount: row.try_get("amount")?,
                accounted: row.try_get::<i64, _>("accounted")? != 0,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(AppError::Database)?;

    let filter_rows = sqlx::query("SELECT name, pattern FROM filters ORDER BY name ASC")
        .fetch_all(pool)
        .await?;

    let filters = filter_rows
        .into_iter()
        .map(|row| {
            Ok(BackupFilter {
                name: row.try_get("name")?,
                pattern: row.try_get("pattern")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(AppError::Database)?;

    let settings_rows = sqlx::query("SELECT key, value FROM settings")
        .fetch_all(pool)
        .await?;

    let settings = settings_rows
        .into_iter()
        .map(|row| {
            let key: String = row.try_get("key").map_err(AppError::Database)?;
            let value: String = row.try_get("value").map_err(AppError::Database)?;
            Ok((key, value))
        })
        .collect::<Result<HashMap<_, _>, AppError>>()?;

    use std::time::{SystemTime, UNIX_EPOCH};
    let exported_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();

    Ok(BackupData {
        version: 1,
        exported_at,
        transactions,
        filters,
        settings,
    })
}

pub async fn import_all(pool: &SqlitePool, data: BackupData) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM transactions")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM filters").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM settings")
        .execute(&mut *tx)
        .await?;

    for t in &data.transactions {
        sqlx::query(
            "INSERT INTO transactions (date, description, amount, accounted) VALUES (?, ?, ?, ?)",
        )
        .bind(&t.date)
        .bind(&t.description)
        .bind(t.amount)
        .bind(t.accounted as i64)
        .execute(&mut *tx)
        .await?;
    }

    for f in &data.filters {
        sqlx::query("INSERT INTO filters (name, pattern) VALUES (?, ?)")
            .bind(&f.name)
            .bind(&f.pattern)
            .execute(&mut *tx)
            .await?;
    }

    for (key, value) in &data.settings {
        sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(&mut *tx)
            .await?;
    }

    // Ensure any settings keys introduced in future migrations have defaults.
    sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES ('split_count', '2')")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

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
            "CREATE TABLE transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                description TEXT NOT NULL,
                amount REAL NOT NULL,
                accounted INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&pool)
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
        sqlx::query("CREATE TABLE settings (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)")
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
    async fn export_empty_db() {
        let pool = setup_db().await;
        let backup = export_all(&pool).await.unwrap();
        assert_eq!(backup.version, 1);
        assert!(backup.transactions.is_empty());
        assert!(backup.filters.is_empty());
        assert_eq!(backup.settings.get("split_count"), Some(&"2".to_string()));
    }

    #[tokio::test]
    async fn round_trip_preserves_data() {
        let pool = setup_db().await;

        sqlx::query(
            "INSERT INTO transactions (date, description, amount, accounted) VALUES (?, ?, ?, ?)",
        )
        .bind("01/15/2026")
        .bind("AMAZON")
        .bind(45.99f64)
        .bind(1i64)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO filters (name, pattern) VALUES (?, ?)")
            .bind("Groceries")
            .bind("WHOLEFDS")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("UPDATE settings SET value = '4' WHERE key = 'split_count'")
            .execute(&pool)
            .await
            .unwrap();

        let backup = export_all(&pool).await.unwrap();

        // Wipe via import
        import_all(&pool, backup).await.unwrap();

        let tx_rows = sqlx::query("SELECT date, description, amount, accounted FROM transactions")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(tx_rows.len(), 1);
        assert_eq!(
            tx_rows[0].try_get::<String, _>("description").unwrap(),
            "AMAZON"
        );
        assert_eq!(tx_rows[0].try_get::<i64, _>("accounted").unwrap(), 1);

        let filter_rows = sqlx::query("SELECT name, pattern FROM filters")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(filter_rows.len(), 1);
        assert_eq!(
            filter_rows[0].try_get::<String, _>("name").unwrap(),
            "Groceries"
        );

        let split: String = sqlx::query("SELECT value FROM settings WHERE key = 'split_count'")
            .fetch_one(&pool)
            .await
            .unwrap()
            .try_get("value")
            .unwrap();
        assert_eq!(split, "4");
    }

    #[tokio::test]
    async fn import_replaces_existing_data() {
        let pool = setup_db().await;

        // Insert some initial data
        sqlx::query(
            "INSERT INTO transactions (date, description, amount) VALUES ('01/10/2026', 'OLD', 10.0)",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Import a backup with different data
        let new_backup = BackupData {
            version: 1,
            exported_at: "0".to_string(),
            transactions: vec![BackupTransaction {
                date: "02/01/2026".to_string(),
                description: "NEW".to_string(),
                amount: 99.0,
                accounted: false,
            }],
            filters: vec![],
            settings: {
                let mut m = HashMap::new();
                m.insert("split_count".to_string(), "3".to_string());
                m
            },
        };

        import_all(&pool, new_backup).await.unwrap();

        let tx_rows = sqlx::query("SELECT description FROM transactions")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(tx_rows.len(), 1);
        assert_eq!(
            tx_rows[0].try_get::<String, _>("description").unwrap(),
            "NEW"
        );
    }
}
