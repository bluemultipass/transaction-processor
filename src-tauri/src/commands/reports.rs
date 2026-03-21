use serde::Serialize;
use sqlx::{Row, SqlitePool};
use tauri::State;

use crate::db::filters;
use crate::db::transactions::{mark_accounted, Transaction};
use crate::error::AppError;

#[derive(Debug, Serialize, specta::Type)]
pub struct ReportRow {
    pub filter_name: String,
    pub last_date: String,
    pub total_amount: f64,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Serialize, specta::Type)]
pub struct ReportOutput {
    pub rows: Vec<ReportRow>,
    pub text: String,
}

/// Convert MM/DD/YYYY to YYYYMMDD for correct chronological string comparison.
fn date_sort_key(date: &str) -> String {
    let parts: Vec<&str> = date.split('/').collect();
    if parts.len() == 3 {
        format!("{}{}{}", parts[2], parts[0], parts[1])
    } else {
        date.to_string()
    }
}

pub async fn generate_report_inner(
    pool: &SqlitePool,
    date_from: Option<&str>,
    date_to: Option<&str>,
    split_count: i64,
) -> Result<ReportOutput, AppError> {
    let all_filters = filters::list_filters(pool).await?;

    let mut rows: Vec<ReportRow> = Vec::new();
    let mut all_matched_ids: Vec<i64> = Vec::new();

    for filter in &all_filters {
        let pattern = format!("%{}%", filter.pattern);

        let matched = sqlx::query(
            "SELECT id, date, description, amount, accounted FROM transactions \
             WHERE description LIKE ? \
             AND (? IS NULL OR date >= ?) \
             AND (? IS NULL OR date <= ?)",
        )
        .bind(&pattern)
        .bind(date_from)
        .bind(date_from)
        .bind(date_to)
        .bind(date_to)
        .fetch_all(pool)
        .await?;

        if matched.is_empty() {
            continue;
        }

        let mut total_amount = 0.0f64;
        let mut last_date = String::new();
        let mut ids: Vec<i64> = Vec::new();
        let mut txs: Vec<Transaction> = Vec::new();

        for row in &matched {
            let id: i64 = row.try_get("id").map_err(AppError::Database)?;
            let date: String = row.try_get("date").map_err(AppError::Database)?;
            let description: String = row.try_get("description").map_err(AppError::Database)?;
            let amount: f64 = row.try_get("amount").map_err(AppError::Database)?;
            let accounted: bool = row
                .try_get::<i64, _>("accounted")
                .map_err(AppError::Database)?
                != 0;

            total_amount += amount;
            if date_sort_key(&date) > date_sort_key(&last_date) {
                last_date = date.clone();
            }
            ids.push(id);
            txs.push(Transaction {
                id,
                date,
                description,
                amount,
                accounted,
            });
        }

        rows.push(ReportRow {
            filter_name: filter.name.clone(),
            last_date,
            total_amount: total_amount / split_count as f64,
            transactions: txs,
        });
        all_matched_ids.extend(ids);
    }

    if !all_matched_ids.is_empty() {
        mark_accounted(pool, &all_matched_ids).await?;
    }

    let text = rows
        .iter()
        .map(|row| {
            format!(
                "{}\t{}\t{:.2}",
                row.filter_name, row.last_date, row.total_amount
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(ReportOutput { rows, text })
}

#[tauri::command]
#[specta::specta]
pub async fn generate_report(
    state: State<'_, SqlitePool>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<ReportOutput, AppError> {
    let split_count = crate::db::settings::get_split_count(&state).await?;
    generate_report_inner(
        &state,
        date_from.as_deref(),
        date_to.as_deref(),
        split_count,
    )
    .await
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
        pool
    }

    async fn insert_tx(pool: &SqlitePool, date: &str, description: &str, amount: f64) -> i64 {
        sqlx::query("INSERT INTO transactions (date, description, amount) VALUES (?, ?, ?)")
            .bind(date)
            .bind(description)
            .bind(amount)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    async fn insert_filter(pool: &SqlitePool, name: &str, pattern: &str) {
        sqlx::query("INSERT INTO filters (name, pattern) VALUES (?, ?)")
            .bind(name)
            .bind(pattern)
            .execute(pool)
            .await
            .unwrap();
    }

    async fn is_accounted(pool: &SqlitePool, id: i64) -> bool {
        let row = sqlx::query("SELECT accounted FROM transactions WHERE id = ?")
            .bind(id)
            .fetch_one(pool)
            .await
            .unwrap();
        row.try_get::<i64, _>("accounted").unwrap() != 0
    }

    #[tokio::test]
    async fn report_sums_amounts_per_filter() {
        let pool = setup_db().await;
        insert_filter(&pool, "Coffee", "STARBUCKS").await;
        insert_tx(&pool, "01/10/2026", "STARBUCKS STORE 123", 4.50).await;
        insert_tx(&pool, "01/15/2026", "STARBUCKS RESERVE", 6.75).await;
        insert_tx(&pool, "01/12/2026", "WHOLEFDS MARKET", 52.00).await;

        let output = generate_report_inner(&pool, None, None, 1).await.unwrap();

        assert_eq!(output.rows.len(), 1);
        assert_eq!(output.rows[0].filter_name, "Coffee");
        assert!((output.rows[0].total_amount - 11.25).abs() < 0.001);
        assert_eq!(output.rows[0].transactions.len(), 2);
    }

    #[tokio::test]
    async fn report_finds_last_date() {
        let pool = setup_db().await;
        insert_filter(&pool, "Coffee", "STARBUCKS").await;
        insert_tx(&pool, "01/10/2026", "STARBUCKS A", 4.00).await;
        insert_tx(&pool, "01/20/2026", "STARBUCKS B", 5.00).await;
        insert_tx(&pool, "01/05/2026", "STARBUCKS C", 3.00).await;

        let output = generate_report_inner(&pool, None, None, 1).await.unwrap();

        assert_eq!(output.rows[0].last_date, "01/20/2026");
        assert_eq!(output.rows[0].transactions.len(), 3);
    }

    #[tokio::test]
    async fn report_marks_matched_transactions_accounted() {
        let pool = setup_db().await;
        insert_filter(&pool, "Coffee", "STARBUCKS").await;
        let id1 = insert_tx(&pool, "01/10/2026", "STARBUCKS A", 4.00).await;
        let id2 = insert_tx(&pool, "01/12/2026", "OTHER STORE", 10.00).await;

        generate_report_inner(&pool, None, None, 1).await.unwrap();

        assert!(is_accounted(&pool, id1).await);
        assert!(!is_accounted(&pool, id2).await);
    }

    #[tokio::test]
    async fn report_filters_by_date_range() {
        let pool = setup_db().await;
        insert_filter(&pool, "Coffee", "STARBUCKS").await;
        insert_tx(&pool, "01/05/2026", "STARBUCKS EARLY", 3.00).await;
        insert_tx(&pool, "01/15/2026", "STARBUCKS MID", 5.00).await;
        insert_tx(&pool, "01/25/2026", "STARBUCKS LATE", 7.00).await;

        let output = generate_report_inner(&pool, Some("01/10/2026"), Some("01/20/2026"), 1)
            .await
            .unwrap();

        assert_eq!(output.rows.len(), 1);
        assert!((output.rows[0].total_amount - 5.00).abs() < 0.001);
    }

    #[tokio::test]
    async fn report_text_is_tab_separated() {
        let pool = setup_db().await;
        insert_filter(&pool, "Coffee", "STARBUCKS").await;
        insert_tx(&pool, "01/10/2026", "STARBUCKS A", 4.50).await;

        let output = generate_report_inner(&pool, None, None, 1).await.unwrap();

        assert!(output.text.contains('\t'));
        assert!(output.text.contains("Coffee"));
        assert!(output.text.contains("4.50"));
    }

    #[tokio::test]
    async fn report_multiple_filters() {
        let pool = setup_db().await;
        insert_filter(&pool, "Coffee", "STARBUCKS").await;
        insert_filter(&pool, "Groceries", "WHOLEFDS").await;
        insert_tx(&pool, "01/10/2026", "STARBUCKS A", 4.50).await;
        insert_tx(&pool, "01/12/2026", "WHOLEFDS MARKET", 55.00).await;

        let output = generate_report_inner(&pool, None, None, 1).await.unwrap();

        assert_eq!(output.rows.len(), 2);
        let lines: Vec<&str> = output.text.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[tokio::test]
    async fn report_empty_when_no_matches() {
        let pool = setup_db().await;
        insert_filter(&pool, "Coffee", "STARBUCKS").await;
        insert_tx(&pool, "01/10/2026", "AMAZON.COM", 20.00).await;

        let output = generate_report_inner(&pool, None, None, 1).await.unwrap();

        assert!(output.rows.is_empty());
        assert!(output.text.is_empty());
    }

    #[tokio::test]
    async fn date_sort_key_orders_correctly_across_years() {
        assert!(date_sort_key("01/17/2026") > date_sort_key("12/01/2025"));
        assert!(date_sort_key("12/31/2025") < date_sort_key("01/01/2026"));
    }

    #[tokio::test]
    async fn report_divides_amounts_by_split_count() {
        let pool = setup_db().await;
        insert_filter(&pool, "Coffee", "STARBUCKS").await;
        insert_tx(&pool, "01/10/2026", "STARBUCKS A", 4.50).await;
        insert_tx(&pool, "01/15/2026", "STARBUCKS B", 6.75).await;

        let output = generate_report_inner(&pool, None, None, 2).await.unwrap();

        assert_eq!(output.rows.len(), 1);
        // 11.25 / 2 = 5.625
        assert!((output.rows[0].total_amount - 5.625).abs() < 0.001);
        assert!(output.text.contains("5.6"));
    }
}
