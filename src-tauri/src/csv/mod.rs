// Items in this module are used in step 6 (transaction commands).
#![allow(dead_code)]

use std::path::Path;

use crate::error::AppError;

#[derive(Debug, PartialEq)]
pub enum CsvFormat {
    Checking,
    CreditCard,
}

#[derive(Debug)]
pub struct ParsedTransaction {
    pub date: String,
    pub description: String,
    pub amount: f64,
}

/// Detects the Chase CSV format from the header row.
/// Credit card exports have "Transaction Date" and "Category";
/// checking exports have "Details" and "Balance".
pub fn detect_format(headers: &[&str]) -> CsvFormat {
    if headers.contains(&"Transaction Date") && headers.contains(&"Category") {
        CsvFormat::CreditCard
    } else {
        CsvFormat::Checking
    }
}

/// Parses a Chase CSV file and returns spend-only transactions.
///
/// Checking format: keeps rows where Amount < 0.
/// Credit card format: keeps rows where Type == "Sale" and Amount < 0.
/// The returned `amount` field is the absolute value.
pub fn parse_transactions(path: &Path) -> Result<Vec<ParsedTransaction>, AppError> {
    let mut reader = csv::Reader::from_path(path).map_err(|e| AppError::Csv(e.to_string()))?;

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| AppError::Csv(e.to_string()))?
        .iter()
        .map(|s| s.to_string())
        .collect();

    let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
    let format = detect_format(&header_refs);

    let mut transactions = Vec::new();

    for result in reader.records() {
        let record = result.map_err(|e| AppError::Csv(e.to_string()))?;

        match format {
            CsvFormat::Checking => {
                let date = get_field(&record, &headers, "Posting Date")?;
                let description = get_field(&record, &headers, "Description")?;
                let amount_str = get_field(&record, &headers, "Amount")?;
                let amount = parse_amount(&amount_str)?;

                if amount >= 0.0 {
                    continue;
                }

                transactions.push(ParsedTransaction {
                    date,
                    description,
                    amount: amount.abs(),
                });
            }
            CsvFormat::CreditCard => {
                let date = get_field(&record, &headers, "Transaction Date")?;
                let description = get_field(&record, &headers, "Description")?;
                let amount_str = get_field(&record, &headers, "Amount")?;
                let type_str = get_field(&record, &headers, "Type")?;
                let amount = parse_amount(&amount_str)?;

                if type_str != "Sale" || amount >= 0.0 {
                    continue;
                }

                transactions.push(ParsedTransaction {
                    date,
                    description,
                    amount: amount.abs(),
                });
            }
        }
    }

    Ok(transactions)
}

fn get_field(
    record: &csv::StringRecord,
    headers: &[String],
    field: &str,
) -> Result<String, AppError> {
    let idx = headers
        .iter()
        .position(|h| h == field)
        .ok_or_else(|| AppError::Csv(format!("missing column: {field}")))?;

    record
        .get(idx)
        .map(|s| s.trim().to_string())
        .ok_or_else(|| AppError::Csv(format!("missing value for column: {field}")))
}

fn parse_amount(s: &str) -> Result<f64, AppError> {
    s.trim()
        .replace(',', "")
        .parse::<f64>()
        .map_err(|_| AppError::Csv(format!("invalid amount: {s}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ── helpers ────────────────────────────────────────────────────────────────

    fn write_csv(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    // ── detect_format ──────────────────────────────────────────────────────────

    #[test]
    fn detects_credit_card_format() {
        let headers = vec![
            "Transaction Date",
            "Post Date",
            "Description",
            "Category",
            "Type",
            "Amount",
            "Memo",
        ];
        assert_eq!(detect_format(&headers), CsvFormat::CreditCard);
    }

    #[test]
    fn detects_checking_format() {
        let headers = vec![
            "Details",
            "Posting Date",
            "Description",
            "Amount",
            "Type",
            "Balance",
            "Check or Slip #",
        ];
        assert_eq!(detect_format(&headers), CsvFormat::Checking);
    }

    // ── checking CSV ───────────────────────────────────────────────────────────

    const CHECKING_CSV: &str = "\
Details,Posting Date,Description,Amount,Type,Balance,Check or Slip #
DEBIT,01/15/2026,AMAZON.COM,-45.99,DEBIT_CARD,1234.56,
CREDIT,01/16/2026,PAYROLL,2000.00,ACH_CREDIT,3234.56,
DEBIT,01/17/2026,STARBUCKS,-5.50,DEBIT_CARD,3229.06,
DEBIT,01/18/2026,WHOLE FOODS,-32.10,DEBIT_CARD,3196.96,
";

    #[test]
    fn checking_filters_out_credits() {
        let f = write_csv(CHECKING_CSV);
        let txs = parse_transactions(f.path()).unwrap();
        assert!(txs.iter().all(|t| t.description != "PAYROLL"));
    }

    #[test]
    fn checking_keeps_debits_as_positive_amounts() {
        let f = write_csv(CHECKING_CSV);
        let txs = parse_transactions(f.path()).unwrap();
        assert_eq!(txs.len(), 3);
        assert!(txs.iter().all(|t| t.amount > 0.0));
    }

    #[test]
    fn checking_preserves_correct_values() {
        let f = write_csv(CHECKING_CSV);
        let txs = parse_transactions(f.path()).unwrap();
        let amazon = txs.iter().find(|t| t.description == "AMAZON.COM").unwrap();
        assert_eq!(amazon.date, "01/15/2026");
        assert!((amazon.amount - 45.99).abs() < f64::EPSILON);
    }

    // ── credit card CSV ────────────────────────────────────────────────────────

    const CREDIT_CSV: &str = "\
Transaction Date,Post Date,Description,Category,Type,Amount,Memo
01/15/2026,01/17/2026,AMAZON.COM,Shopping,Sale,-45.99,
01/16/2026,01/18/2026,AUTOPAY,Payment,Payment,100.00,
01/17/2026,01/19/2026,STARBUCKS,Food & Drink,Sale,-5.50,
01/18/2026,01/20/2026,RETURN CREDIT,Shopping,Return,15.00,
";

    #[test]
    fn credit_filters_out_payments_and_returns() {
        let f = write_csv(CREDIT_CSV);
        let txs = parse_transactions(f.path()).unwrap();
        assert!(txs.iter().all(|t| t.description != "AUTOPAY"));
        assert!(txs.iter().all(|t| t.description != "RETURN CREDIT"));
    }

    #[test]
    fn credit_keeps_sales_as_positive_amounts() {
        let f = write_csv(CREDIT_CSV);
        let txs = parse_transactions(f.path()).unwrap();
        assert_eq!(txs.len(), 2);
        assert!(txs.iter().all(|t| t.amount > 0.0));
    }

    #[test]
    fn credit_preserves_correct_values() {
        let f = write_csv(CREDIT_CSV);
        let txs = parse_transactions(f.path()).unwrap();
        let starbucks = txs.iter().find(|t| t.description == "STARBUCKS").unwrap();
        assert_eq!(starbucks.date, "01/17/2026");
        assert!((starbucks.amount - 5.50).abs() < f64::EPSILON);
    }
}
