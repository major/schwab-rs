//! Account transaction lookup command.

use clap::Args;
use serde_json::Value;
use time::{Date, Duration, OffsetDateTime, Time, format_description::well_known::Rfc3339};

use crate::{account, auth, cli::TransactionsArgs, error::AppError};

pub mod cli {
    use super::*;

    /// Arguments for `transactions`.
    #[derive(Debug, Args)]
    #[command(
        after_help = "Examples:\n  schwab-agent transactions\n      Get recent trades for the primary account.\n\n  schwab-agent transactions --account Trading --symbol AAPL\n      Get recent AAPL trades for the Trading account.\n\nDate filters accept RFC3339 timestamps or YYYY-MM-DD. Date-only --from starts at 00:00:00Z; date-only --to ends at 23:59:59Z."
    )]
    pub struct TransactionsArgs {
        /// Account hash or unique nickname. Omitted account uses primary.
        #[arg(long)]
        pub account: Option<String>,
        /// Symbol filter.
        #[arg(long)]
        pub symbol: Option<String>,
        /// Start date/time. Defaults to --days ago.
        #[arg(long)]
        pub from: Option<String>,
        /// End date/time. Defaults to now.
        #[arg(long)]
        pub to: Option<String>,
        /// Days back when --from is omitted.
        #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u32).range(1..))]
        pub days: u32,
        /// Schwab transaction type.
        #[arg(
            long = "transaction-type",
            default_value = "TRADE",
            long_help = "Schwab transaction type. Known values: TRADE, RECEIVE_AND_DELIVER, DIVIDEND_OR_INTEREST, ACH_RECEIPT, ACH_DISBURSEMENT, CASH_RECEIPT, CASH_DISBURSEMENT, ELECTRONIC_FUND, WIRE_OUT, WIRE_IN, JOURNAL, MEMORANDUM, MARGIN_CALL, MONEY_MARKET, SMA_ADJUSTMENT."
        )]
        pub r#type: String,
        /// Fetch one transaction by ID. Omitted account uses primary.
        #[arg(long = "transaction-id", conflicts_with_all = ["from", "to", "days", "symbol"])]
        pub transaction_id: Option<i64>,
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn handle(args: &TransactionsArgs) -> Result<Value, AppError> {
    let provider = auth::provider()?;
    let token = provider.token().await?;
    let account_hash = match &args.account {
        Some(selector) => {
            account::resolve_account(&token, selector)
                .await?
                .account_hash
        }
        None => default_account_hash(&token).await?,
    };

    if let Some(id) = args.transaction_id {
        return crate::raw::fetch_transaction_by_id(&token, &account_hash, id).await;
    }

    let now = OffsetDateTime::now_utc();
    let from = match &args.from {
        Some(value) => parse_date_arg(value, DateBoundary::Start)?,
        None => format_rfc3339(now - Duration::days(args.days.into())),
    };
    let to = match &args.to {
        Some(value) => parse_date_arg(value, DateBoundary::End)?,
        None => format_rfc3339(now),
    };
    let mut params = vec![
        ("startDate", from.as_str()),
        ("endDate", to.as_str()),
        ("types", args.r#type.as_str()),
    ];
    if let Some(symbol) = args.symbol.as_deref() {
        params.push(("symbol", symbol));
    }

    let transactions = crate::raw::fetch_transaction_list(&token, &account_hash, &params).await?;
    Ok(sort_oldest_first(transactions))
}

fn sort_oldest_first(mut value: Value) -> Value {
    if let Some(transactions) = value.as_array_mut() {
        transactions.sort_by(|a, b| transaction_sort_key(a).cmp(&transaction_sort_key(b)));
    }
    value
}

fn transaction_sort_key(value: &Value) -> (bool, &str) {
    let timestamp = value
        .get("time")
        .or_else(|| value.get("tradeDate"))
        .or_else(|| value.get("settlementDate"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    (timestamp.is_empty(), timestamp)
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn default_account_hash(token: &str) -> Result<String, AppError> {
    let http = reqwest::Client::new();
    let (hashes, preferences) = tokio::try_join!(
        crate::raw::fetch_account_numbers_with_client(&http, token),
        crate::raw::fetch_user_preference_with_client(&http, token),
    )?;
    let prefs = account::preference_accounts(preferences);
    account::resolve_default_account_hash_from_data(&hashes, &prefs)
}

fn format_rfc3339(value: OffsetDateTime) -> String {
    value
        .format(&Rfc3339)
        .expect("RFC3339 formatting cannot fail")
}

#[derive(Clone, Copy)]
enum DateBoundary {
    Start,
    End,
}

fn parse_date_arg(value: &str, boundary: DateBoundary) -> Result<String, AppError> {
    if value.len() == 10 && value.as_bytes().get(4) == Some(&b'-') {
        let date = parse_date_only(value)?;
        let time = match boundary {
            DateBoundary::Start => Time::MIDNIGHT,
            DateBoundary::End => Time::from_hms(23, 59, 59).expect("valid time"),
        };
        return Ok(format_rfc3339(date.with_time(time).assume_utc()));
    }

    Ok(value.to_string())
}

fn parse_date_only(value: &str) -> Result<Date, AppError> {
    let format = time::format_description::parse_borrowed::<1>("[year]-[month]-[day]")
        .map_err(|_| invalid_date(value))?;
    Date::parse(value, &format).map_err(|_| invalid_date(value))
}

fn invalid_date(value: &str) -> AppError {
    AppError::AccountValidation(format!(
        "invalid transaction date '{value}'; use YYYY-MM-DD or RFC3339"
    ))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{DateBoundary, parse_date_arg, sort_oldest_first};

    #[test]
    fn sort_oldest_first_leaves_non_arrays_unchanged() {
        let value = json!({"transactionId": 1});
        assert_eq!(sort_oldest_first(value.clone()), value);
    }

    #[test]
    fn sorts_transactions_oldest_first() {
        let value = json!([
            {"time":"2026-01-03T00:00:00Z", "transactionId": 3},
            {"time":"2026-01-01T00:00:00Z", "transactionId": 1},
            {"tradeDate":"2026-01-02", "transactionId": 2},
            {"transactionId": 4}
        ]);

        let sorted = sort_oldest_first(value);
        let ids: Vec<_> = sorted
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["transactionId"].as_i64().unwrap())
            .collect();

        assert_eq!(ids, [1, 2, 3, 4]);
    }

    #[test]
    fn date_only_ranges_expand_to_inclusive_utc_day() {
        assert_eq!(
            parse_date_arg("2026-01-02", DateBoundary::Start).unwrap(),
            "2026-01-02T00:00:00Z"
        );
        assert_eq!(
            parse_date_arg("2026-01-02", DateBoundary::End).unwrap(),
            "2026-01-02T23:59:59Z"
        );
    }

    #[test]
    fn rfc3339_ranges_pass_through() {
        let value = "2026-01-02T12:34:56Z";
        assert_eq!(parse_date_arg(value, DateBoundary::Start).unwrap(), value);
    }

    #[test]
    fn invalid_date_only_is_rejected() {
        let error = parse_date_arg("2026-99-99", DateBoundary::Start).unwrap_err();
        assert!(error.to_string().contains("invalid transaction date"));
    }
}
