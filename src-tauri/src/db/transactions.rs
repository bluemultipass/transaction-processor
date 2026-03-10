use serde::Serialize;
use sqlx::{Row, SqlitePool};

use crate::csv::ParsedTransaction;
use crate::error::AppError;

#[derive(Debug, Serialize, specta::Type)]
pub struct Transaction {
    #[serde(serialize_with = "crate::db::serialize_i64_safe")]
    pub id: i64,
    pub date: String,
    pub description: String,
    pub amount: f64,
    pub accounted: bool,
}

pub async fn insert_transactions(
    pool: &SqlitePool,
    transactions: Vec<ParsedTransaction>,
) -> Result<(), AppError> {
    for tx in transactions {
        sqlx::query("INSERT INTO transactions (date, description, amount) VALUES (?, ?, ?)")
            .bind(&tx.date)
            .bind(&tx.description)
            .bind(tx.amount)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn list_transactions(
    pool: &SqlitePool,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> Result<Vec<Transaction>, AppError> {
    let rows = sqlx::query(
        "SELECT id, date, description, amount, accounted FROM transactions \
         WHERE (? IS NULL OR date >= ?) AND (? IS NULL OR date <= ?) \
         ORDER BY date DESC",
    )
    .bind(date_from)
    .bind(date_from)
    .bind(date_to)
    .bind(date_to)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(Transaction {
                id: row.try_get("id")?,
                date: row.try_get("date")?,
                description: row.try_get("description")?,
                amount: row.try_get("amount")?,
                accounted: row.try_get::<i64, _>("accounted")? != 0,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(AppError::Database)
}

pub async fn mark_accounted(pool: &SqlitePool, ids: &[i64]) -> Result<(), AppError> {
    for id in ids {
        sqlx::query("UPDATE transactions SET accounted = 1 WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
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
        pool
    }

    fn tx(date: &str, description: &str, amount: f64) -> ParsedTransaction {
        ParsedTransaction {
            date: date.to_string(),
            description: description.to_string(),
            amount,
        }
    }

    #[tokio::test]
    async fn insert_and_list_transactions() {
        let pool = setup_db().await;
        let transactions = vec![
            tx("01/15/2026", "AMAZON.COM", 45.99),
            tx("01/17/2026", "STARBUCKS", 5.50),
        ];
        insert_transactions(&pool, transactions).await.unwrap();

        let result = list_transactions(&pool, None, None).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn list_transactions_ordered_by_date_desc() {
        let pool = setup_db().await;
        let transactions = vec![
            tx("01/15/2026", "AMAZON.COM", 45.99),
            tx("01/17/2026", "STARBUCKS", 5.50),
        ];
        insert_transactions(&pool, transactions).await.unwrap();

        let result = list_transactions(&pool, None, None).await.unwrap();
        assert_eq!(result[0].date, "01/17/2026");
        assert_eq!(result[1].date, "01/15/2026");
    }

    #[tokio::test]
    async fn list_transactions_filters_by_date_from() {
        let pool = setup_db().await;
        let transactions = vec![
            tx("01/15/2026", "AMAZON.COM", 45.99),
            tx("01/17/2026", "STARBUCKS", 5.50),
        ];
        insert_transactions(&pool, transactions).await.unwrap();

        let result = list_transactions(&pool, Some("01/17/2026"), None)
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "STARBUCKS");
    }

    #[tokio::test]
    async fn list_transactions_filters_by_date_to() {
        let pool = setup_db().await;
        let transactions = vec![
            tx("01/15/2026", "AMAZON.COM", 45.99),
            tx("01/17/2026", "STARBUCKS", 5.50),
        ];
        insert_transactions(&pool, transactions).await.unwrap();

        let result = list_transactions(&pool, None, Some("01/15/2026"))
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description, "AMAZON.COM");
    }

    #[tokio::test]
    async fn mark_accounted_updates_flag() {
        let pool = setup_db().await;
        insert_transactions(&pool, vec![tx("01/15/2026", "AMAZON.COM", 45.99)])
            .await
            .unwrap();

        let rows = list_transactions(&pool, None, None).await.unwrap();
        assert!(!rows[0].accounted);

        mark_accounted(&pool, &[rows[0].id]).await.unwrap();

        let rows = list_transactions(&pool, None, None).await.unwrap();
        assert!(rows[0].accounted);
    }
}
