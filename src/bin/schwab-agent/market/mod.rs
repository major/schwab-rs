//! Market data command handlers for quotes and price history.

use schwab::{PriceHistoryOptions, QuoteOptions, QuoteResponseObject};
use serde::Serialize;
use serde_json::{Value, to_value};
use time::format_description::well_known::Rfc3339;
use time::{Date, Month, OffsetDateTime, Time};

use crate::auth;
use crate::cli::{Cli, HistoryArgs, MarketCommand, QuoteArgs};
use crate::error::AppError;

/// Routes market subcommands to their handlers and returns a JSON value.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn handle(cli: &Cli, command: &MarketCommand) -> Result<Value, AppError> {
    match command {
        MarketCommand::History(args) => history(cli, args).await,
        MarketCommand::Quote(args) => quote(cli, args).await,
    }
}

/// Fetches price history candles for a single symbol and returns them as JSON.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn history(_cli: &Cli, args: &HistoryArgs) -> Result<Value, AppError> {
    let selected_fields = if args.all_fields {
        None
    } else {
        Some(selected_history_fields(args.fields.as_deref())?)
    };

    let mut options = PriceHistoryOptions::new();
    if let Some(period_type) = &args.period_type {
        options = options.parameter("periodType", period_type);
    }
    if let Some(period) = args.period {
        options = options.integer_parameter("period", period);
    }
    if let Some(frequency_type) = &args.frequency_type {
        options = options.parameter("frequencyType", frequency_type);
    }
    if let Some(frequency) = args.frequency {
        options = options.integer_parameter("frequency", frequency);
    }
    if let Some(from) = &args.from {
        let from = parse_history_instant(from, HistoryRangeBoundary::Start)?;
        options = options.integer_parameter("startDate", from);
    }
    if let Some(to) = &args.to {
        let to = parse_history_instant(to, HistoryRangeBoundary::End)?;
        options = options.integer_parameter("endDate", to);
    }
    if args.extended_hours {
        options = options.bool_parameter("needExtendedHoursData", true);
    }
    let client = auth::provider()?.client().await?;
    let candle_list = client.get_price_history(&args.symbol, options).await?;
    let value = to_value(candle_list)?;
    if args.all_fields {
        return Ok(value);
    }

    let fields =
        selected_fields.expect("compact history fields are validated unless --all-fields is set");
    Ok(to_value(select_history_fields(&value, &fields))?)
}

/// Default token-optimized candle fields for compact history row output.
pub(crate) const DEFAULT_HISTORY_FIELDS: [&str; 6] = ["ts", "open", "high", "low", "close", "vol"];

/// Inclusive boundary used when converting date-only history values.
#[derive(Clone, Copy)]
enum HistoryRangeBoundary {
    /// Start of the UTC calendar day.
    Start,
    /// End of the UTC calendar day.
    End,
}

/// Parses a history date argument to epoch milliseconds for the Schwab API.
fn parse_history_instant(value: &str, boundary: HistoryRangeBoundary) -> Result<i64, AppError> {
    let value = value.trim();
    if let Ok(epoch_millis) = value.parse::<i64>() {
        return Ok(epoch_millis);
    }

    let instant = if is_date_only(value) {
        history_date_boundary(parse_history_date_only(value)?, boundary)
    } else {
        OffsetDateTime::parse(value, &Rfc3339).map_err(|e| AppError::MarketValidation {
            message: format!(
                "invalid market history date/time '{value}': expected YYYY-MM-DD, RFC3339, or epoch milliseconds ({e})"
            ),
        })?
    };

    Ok(epoch_millis(instant))
}

/// Returns true when a value matches the supported YYYY-MM-DD shape.
fn is_date_only(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..].iter().all(u8::is_ascii_digit)
}

/// Parses a YYYY-MM-DD history date without local timezone inference.
fn parse_history_date_only(value: &str) -> Result<Date, AppError> {
    let year = value[0..4]
        .parse::<i32>()
        .map_err(|e| invalid_history_date(value, e))?;
    let month_number = value[5..7]
        .parse::<u8>()
        .map_err(|e| invalid_history_date(value, e))?;
    let day = value[8..10]
        .parse::<u8>()
        .map_err(|e| invalid_history_date(value, e))?;
    let month = Month::try_from(month_number).map_err(|e| invalid_history_date(value, e))?;

    Date::from_calendar_date(year, month, day).map_err(|e| invalid_history_date(value, e))
}

/// Converts a calendar date to the requested inclusive UTC history boundary.
fn history_date_boundary(date: Date, boundary: HistoryRangeBoundary) -> OffsetDateTime {
    let time = match boundary {
        HistoryRangeBoundary::Start => Time::MIDNIGHT,
        HistoryRangeBoundary::End => {
            Time::from_hms_milli(23, 59, 59, 999).expect("23:59:59.999 is a valid time")
        }
    };

    date.with_time(time).assume_utc()
}

/// Converts a timestamp to epoch milliseconds.
fn epoch_millis(value: OffsetDateTime) -> i64 {
    i64::try_from(value.unix_timestamp_nanos() / 1_000_000)
        .expect("time crate timestamp range fits in i64 epoch milliseconds")
}

/// Builds a consistent validation error for invalid YYYY-MM-DD history dates.
fn invalid_history_date<E: std::fmt::Display>(value: &str, error: E) -> AppError {
    AppError::MarketValidation {
        message: format!("invalid market history date '{value}': {error}"),
    }
}

/// Parses the optional comma-separated history field list, or returns the compact default.
fn selected_history_fields(requested: Option<&str>) -> Result<Vec<&'static str>, AppError> {
    let Some(requested) = requested else {
        return Ok(DEFAULT_HISTORY_FIELDS.to_vec());
    };
    let fields = requested
        .split(',')
        .map(str::trim)
        .filter(|field| !field.is_empty())
        .collect::<Vec<_>>();
    if fields.is_empty() {
        return Err(AppError::MarketValidation {
            message: format!(
                "history --fields cannot be empty; hint: available fields: {}",
                available_history_fields().join(", ")
            ),
        });
    }
    validate_history_fields(&fields)?;
    Ok(fields
        .iter()
        .map(|field| canonical_history_field(field))
        .collect())
}

/// Validates user-requested history candle field names and aliases.
fn validate_history_fields(requested: &[&str]) -> Result<(), AppError> {
    let unknown = requested
        .iter()
        .filter(|field| canonical_history_field(field).is_empty())
        .copied()
        .collect::<Vec<_>>();

    if unknown.is_empty() {
        return Ok(());
    }

    Err(AppError::MarketValidation {
        message: format!(
            "unknown history field(s): {}; hint: available fields: {}",
            unknown.join(", "),
            available_history_fields().join(", ")
        ),
    })
}

/// Projects Schwab price-history candles into a compact table-shaped response.
#[must_use]
fn select_history_fields(history: &Value, fields: &[&str]) -> HistoryRowsOutput {
    let rows = history
        .get("candles")
        .and_then(Value::as_array)
        .map(|candles| {
            candles
                .iter()
                .map(|candle| {
                    fields
                        .iter()
                        .map(|field| selected_history_field_value(candle, field))
                        .collect()
                })
                .collect::<Vec<Vec<Value>>>()
        })
        .unwrap_or_default();

    HistoryRowsOutput {
        symbol: history
            .get("symbol")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        columns: fields.iter().map(|field| (*field).to_string()).collect(),
        row_count: rows.len(),
        rows,
    }
}

/// Maps accepted history field aliases to their emitted compact column name.
fn canonical_history_field(field: &str) -> &'static str {
    match field {
        "timestamp" | "datetime" | "time" | "ts" => "ts",
        "datetime_iso8601" | "datetimeISO8601" | "iso8601" | "iso" => "iso",
        "open" | "o" => "open",
        "high" | "h" => "high",
        "low" | "l" => "low",
        "close" | "c" => "close",
        "volume" | "vol" | "v" => "vol",
        _ => "",
    }
}

/// Returns every accepted history field name and alias for validation hints.
pub(crate) fn available_history_fields() -> Vec<&'static str> {
    let mut fields = [
        "timestamp",
        "datetime",
        "time",
        "ts",
        "datetime_iso8601",
        "datetimeISO8601",
        "iso8601",
        "iso",
        "open",
        "o",
        "high",
        "h",
        "low",
        "l",
        "close",
        "c",
        "volume",
        "vol",
        "v",
    ]
    .to_vec();
    fields.sort_unstable();
    fields
}

/// Extracts a single selected history candle field as a JSON value.
fn selected_history_field_value(candle: &Value, field: &str) -> Value {
    match field {
        "ts" => candle.get("datetime").cloned().unwrap_or(Value::Null),
        "iso" => candle
            .get("datetimeISO8601")
            .cloned()
            .unwrap_or(Value::Null),
        "open" => candle.get("open").cloned().unwrap_or(Value::Null),
        "high" => candle.get("high").cloned().unwrap_or(Value::Null),
        "low" => candle.get("low").cloned().unwrap_or(Value::Null),
        "close" => candle.get("close").cloned().unwrap_or(Value::Null),
        "vol" => candle.get("volume").cloned().unwrap_or(Value::Null),
        _ => Value::Null,
    }
}

/// Fetches quotes for the requested symbols from the Schwab API and returns either
/// compact row output or the full flattened [`QuoteSummary`] object list.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn quote(_cli: &Cli, args: &QuoteArgs) -> Result<Value, AppError> {
    let selected_fields = if args.all_fields {
        None
    } else {
        Some(selected_quote_fields(args.fields.as_deref())?)
    };

    let client = auth::provider()?.client().await?;
    let quotes = if let Some(fields) = &args.api_fields {
        client
            .get_quotes_with_options(&args.symbols, QuoteOptions::new().fields(fields))
            .await?
    } else {
        client.get_quotes(&args.symbols).await?
    };
    let summaries = quotes
        .into_iter()
        .map(|(requested_symbol, quote)| summarize_quote(requested_symbol, quote))
        .collect::<Vec<_>>();
    if args.all_fields {
        return Ok(to_value(QuoteOutput {
            symbols: args.symbols.clone(),
            quotes: sort_quote_summaries(summaries),
        })?);
    }

    let summaries = normalize_quote_summaries(summaries, &args.symbols);
    let fields =
        selected_fields.expect("compact quote fields are validated unless --all-fields is set");
    Ok(to_value(select_quote_fields(&summaries, &fields))?)
}

/// Sorts quote summaries by requested symbol for stable detailed output.
fn sort_quote_summaries(mut summaries: Vec<QuoteSummary>) -> Vec<QuoteSummary> {
    summaries.sort_by(|left, right| left.requested_symbol.cmp(&right.requested_symbol));
    summaries
}

/// Keeps one useful row per requested symbol, even when Schwab returns a generic `errors` entry.
fn normalize_quote_summaries(
    mut summaries: Vec<QuoteSummary>,
    requested_symbols: &[String],
) -> Vec<QuoteSummary> {
    let generic_errors = summaries
        .iter()
        .filter(|summary| {
            !requested_symbols
                .iter()
                .any(|requested| quote_summary_matches_request(summary, requested))
        })
        .filter_map(|summary| summary.error.as_ref().map(clone_quote_error))
        .collect::<Vec<_>>();

    summaries.retain(|summary| {
        requested_symbols
            .iter()
            .any(|requested| quote_summary_matches_request(summary, requested))
    });

    for requested_symbol in requested_symbols {
        if summaries
            .iter()
            .any(|summary| quote_summary_matches_request(summary, requested_symbol))
        {
            continue;
        }

        let error = generic_quote_error(&generic_errors, requested_symbol, requested_symbols.len())
            .unwrap_or_else(|| missing_quote_error(requested_symbol.clone()));
        summaries.push(missing_quote_summary(requested_symbol.clone(), error));
    }

    sort_quote_summaries(summaries)
}

/// Finds API-provided error details that apply to a missing requested symbol.
fn generic_quote_error(
    generic_errors: &[QuoteErrorSummary],
    requested_symbol: &str,
    requested_count: usize,
) -> Option<QuoteErrorSummary> {
    for error in generic_errors {
        if error.invalid_symbols.as_ref().is_some_and(|invalid| {
            invalid
                .iter()
                .any(|symbol| symbol.eq_ignore_ascii_case(requested_symbol))
        }) {
            return Some(clone_quote_error(error));
        }
    }

    generic_errors
        .iter()
        .find(|error| requested_count == 1 || error.invalid_symbols.is_none())
        .map(clone_quote_error)
}

/// Checks whether a normalized quote row corresponds to a user-requested symbol.
fn quote_summary_matches_request(summary: &QuoteSummary, requested_symbol: &str) -> bool {
    summary
        .requested_symbol
        .eq_ignore_ascii_case(requested_symbol)
        || summary
            .symbol
            .as_deref()
            .is_some_and(|symbol| symbol.eq_ignore_ascii_case(requested_symbol))
}

/// Builds a compact error row for requested symbols omitted from Schwab's quote map.
fn missing_quote_summary(requested_symbol: String, error: QuoteErrorSummary) -> QuoteSummary {
    QuoteSummary {
        requested_symbol,
        error: Some(error),
        ..QuoteSummary::default()
    }
}

/// Builds the fallback error used when Schwab omits a requested symbol without details.
fn missing_quote_error(requested_symbol: String) -> QuoteErrorSummary {
    QuoteErrorSummary {
        invalid_symbols: Some(vec![requested_symbol]),
        invalid_cusips: None,
        invalid_ssids: None,
    }
}

/// Clones API error details without making the private output type broadly cloneable.
fn clone_quote_error(error: &QuoteErrorSummary) -> QuoteErrorSummary {
    QuoteErrorSummary {
        invalid_symbols: error.invalid_symbols.clone(),
        invalid_cusips: error.invalid_cusips.clone(),
        invalid_ssids: error.invalid_ssids.clone(),
    }
}

/// Default token-optimized quote fields for compact row output.
pub(crate) const DEFAULT_QUOTE_FIELDS: [&str; 10] = [
    "req", "sym", "bid", "ask", "last", "mark", "chg", "pct", "vol", "err",
];

/// Parses the optional comma-separated quote field list, or returns the compact default.
fn selected_quote_fields(requested: Option<&str>) -> Result<Vec<&'static str>, AppError> {
    let Some(requested) = requested else {
        return Ok(DEFAULT_QUOTE_FIELDS.to_vec());
    };
    let fields = requested
        .split(',')
        .map(str::trim)
        .filter(|field| !field.is_empty())
        .collect::<Vec<_>>();
    if fields.is_empty() {
        return Err(AppError::MarketValidation {
            message: format!(
                "quote --fields cannot be empty; hint: available fields: {}",
                available_quote_fields().join(", ")
            ),
        });
    }
    validate_quote_fields(&fields)?;
    Ok(fields
        .iter()
        .map(|field| canonical_quote_field(field))
        .collect())
}

/// Validates user-requested quote field names and aliases.
fn validate_quote_fields(requested: &[&str]) -> Result<(), AppError> {
    let unknown = requested
        .iter()
        .filter(|field| canonical_quote_field(field).is_empty())
        .copied()
        .collect::<Vec<_>>();

    if unknown.is_empty() {
        return Ok(());
    }

    Err(AppError::MarketValidation {
        message: format!(
            "unknown quote field(s): {}; hint: available fields: {}",
            unknown.join(", "),
            available_quote_fields().join(", ")
        ),
    })
}

/// Projects quote summaries into a compact table-shaped response.
#[must_use]
fn select_quote_fields(summaries: &[QuoteSummary], fields: &[&str]) -> QuoteRowsOutput {
    let columns = fields.iter().map(|field| (*field).to_string()).collect();
    let rows = summaries
        .iter()
        .map(|summary| {
            fields
                .iter()
                .map(|field| selected_quote_field_value(summary, field))
                .collect()
        })
        .collect::<Vec<Vec<Value>>>();
    QuoteRowsOutput {
        columns,
        row_count: rows.len(),
        rows,
    }
}

/// Maps accepted quote field aliases to their emitted compact column name.
fn canonical_quote_field(field: &str) -> &'static str {
    match field {
        "requested_symbol" | "requested" | "req" => "req",
        "symbol" | "sym" => "sym",
        "asset_type" | "asset" | "type" => "asset",
        "description" | "desc" => "desc",
        "exchange" | "exch" => "exch",
        "bid" => "bid",
        "ask" => "ask",
        "last" => "last",
        "mark" => "mark",
        "net_change" | "change" | "chg" => "chg",
        "net_percent_change" | "percent_change" | "pct" | "chg_pct" => "pct",
        "volume" | "vol" => "vol",
        "quote_time" | "qt" => "qt",
        "trade_time" | "tt" => "tt",
        "security_status" | "status" => "status",
        "realtime" | "rt" => "rt",
        "underlying" | "und" => "und",
        "put_call" | "cp" => "cp",
        "strike_price" | "strike" => "strike",
        "days_to_expiration" | "dte" => "dte",
        "error" | "err" => "err",
        _ => "",
    }
}

/// Returns every accepted quote field name and alias for validation hints.
pub(crate) fn available_quote_fields() -> Vec<&'static str> {
    let mut fields = [
        "requested_symbol",
        "requested",
        "req",
        "symbol",
        "sym",
        "asset_type",
        "asset",
        "type",
        "description",
        "desc",
        "exchange",
        "exch",
        "bid",
        "ask",
        "last",
        "mark",
        "net_change",
        "change",
        "chg",
        "net_percent_change",
        "percent_change",
        "pct",
        "chg_pct",
        "volume",
        "vol",
        "quote_time",
        "qt",
        "trade_time",
        "tt",
        "security_status",
        "status",
        "realtime",
        "rt",
        "underlying",
        "und",
        "put_call",
        "cp",
        "strike_price",
        "strike",
        "days_to_expiration",
        "dte",
        "error",
        "err",
    ]
    .to_vec();
    fields.sort_unstable();
    fields
}

/// Extracts a single selected quote field as a JSON value.
fn selected_quote_field_value(summary: &QuoteSummary, field: &str) -> Value {
    match field {
        "req" => Value::String(summary.requested_symbol.clone()),
        "sym" => option_string(&summary.symbol),
        "asset" => option_string(&summary.asset_type),
        "desc" => option_string(&summary.description),
        "exch" => option_string(&summary.exchange),
        "bid" => option_number(&summary.bid),
        "ask" => option_number(&summary.ask),
        "last" => option_number(&summary.last),
        "mark" => option_number(&summary.mark),
        "chg" => option_number(&summary.net_change),
        "pct" => option_number(&summary.net_percent_change),
        "vol" => option_i64(summary.volume),
        "qt" => option_i64(summary.quote_time),
        "tt" => option_i64(summary.trade_time),
        "status" => option_string(&summary.security_status),
        "rt" => summary.realtime.map_or(Value::Null, Value::Bool),
        "und" => option_string(&summary.underlying),
        "cp" => option_string(&summary.put_call),
        "strike" => option_number(&summary.strike_price),
        "dte" => summary.days_to_expiration.map_or(Value::Null, Value::from),
        "err" => summary
            .error
            .as_ref()
            .map_or(Value::Null, |error| to_value(error).unwrap_or(Value::Null)),
        _ => Value::Null,
    }
}

/// Converts an optional string into a JSON scalar or null.
fn option_string(value: &Option<String>) -> Value {
    value.clone().map_or(Value::Null, Value::String)
}

/// Converts an optional Schwab number into a JSON scalar or null.
fn option_number(value: &Option<schwab::Number>) -> Value {
    value.as_ref().map_or(Value::Null, |number| {
        to_value(number).unwrap_or(Value::Null)
    })
}

/// Converts an optional i64 into a JSON scalar or null.
fn option_i64(value: Option<i64>) -> Value {
    value.map_or(Value::Null, Value::from)
}

/// Extracts a field from an `Option<T>` by reference, returning `None` if the
/// outer option is `None`. Use `clone` for non-`Copy` fields like `String`.
macro_rules! opt_field {
    ($opt:expr, $field:ident) => {
        $opt.as_ref().and_then(|v| v.$field)
    };
    ($opt:expr, clone $field:ident) => {
        $opt.as_ref().and_then(|v| v.$field.clone())
    };
}

/// Normalizes all eight [`QuoteResponseObject`] variants (Equity, Option, MutualFund,
/// Forex, Future, FutureOption, Index, Error) into a single flat [`QuoteSummary`].
/// Fields that don't apply to a given asset type are left at their `Default` value (`None`).
pub(crate) fn summarize_quote(
    requested_symbol: String,
    quote: QuoteResponseObject,
) -> QuoteSummary {
    match quote {
        QuoteResponseObject::Equity(response) => {
            let quote = response.quote;
            let reference = response.reference;
            QuoteSummary {
                requested_symbol,
                symbol: response.symbol,
                asset_type: response.asset_main_type.map(|v| format!("{v:?}")),
                description: opt_field!(reference, clone description),
                exchange: opt_field!(reference, clone exchange_name),
                bid: opt_field!(quote, bid_price),
                ask: opt_field!(quote, ask_price),
                last: opt_field!(quote, last_price),
                mark: opt_field!(quote, mark),
                net_change: opt_field!(quote, net_change),
                net_percent_change: opt_field!(quote, net_percent_change),
                volume: opt_field!(quote, total_volume),
                quote_time: opt_field!(quote, quote_time),
                trade_time: opt_field!(quote, trade_time),
                security_status: opt_field!(quote, clone security_status),
                realtime: response.realtime,
                ..QuoteSummary::default()
            }
        }
        QuoteResponseObject::Option(response) => {
            let quote = response.quote;
            let reference = response.reference;
            QuoteSummary {
                requested_symbol,
                symbol: response.symbol,
                asset_type: response.asset_main_type.map(|v| format!("{v:?}")),
                description: opt_field!(reference, clone description),
                exchange: opt_field!(reference, clone exchange_name),
                bid: opt_field!(quote, bid_price),
                ask: opt_field!(quote, ask_price),
                last: opt_field!(quote, last_price),
                mark: opt_field!(quote, mark),
                net_change: opt_field!(quote, net_change),
                net_percent_change: opt_field!(quote, net_percent_change),
                volume: opt_field!(quote, total_volume),
                quote_time: opt_field!(quote, quote_time),
                trade_time: opt_field!(quote, trade_time),
                security_status: opt_field!(quote, clone security_status),
                realtime: response.realtime,
                underlying: opt_field!(reference, clone underlying),
                put_call: reference
                    .as_ref()
                    .and_then(|v| v.contract_type.as_ref())
                    .map(|v| format!("{v:?}")),
                strike_price: opt_field!(reference, strike_price),
                days_to_expiration: opt_field!(reference, days_to_expiration),
                ..QuoteSummary::default()
            }
        }
        QuoteResponseObject::MutualFund(response) => {
            let quote = response.quote;
            let reference = response.reference;
            QuoteSummary {
                requested_symbol,
                symbol: response.symbol,
                asset_type: response.asset_main_type.map(|v| format!("{v:?}")),
                description: opt_field!(reference, clone description),
                exchange: opt_field!(reference, clone exchange_name),
                last: opt_field!(quote, nav),
                mark: opt_field!(quote, nav),
                net_change: opt_field!(quote, net_change),
                net_percent_change: opt_field!(quote, net_percent_change),
                volume: opt_field!(quote, total_volume),
                trade_time: opt_field!(quote, trade_time),
                security_status: opt_field!(quote, clone security_status),
                realtime: response.realtime,
                ..QuoteSummary::default()
            }
        }
        QuoteResponseObject::Forex(response) => {
            let quote = response.quote;
            QuoteSummary {
                requested_symbol,
                symbol: response.symbol,
                asset_type: response.asset_main_type.map(|v| format!("{v:?}")),
                description: response.reference.and_then(|v| v.description),
                bid: opt_field!(quote, bid_price),
                ask: opt_field!(quote, ask_price),
                last: opt_field!(quote, last_price),
                mark: opt_field!(quote, mark),
                net_change: opt_field!(quote, net_change),
                net_percent_change: opt_field!(quote, net_percent_change),
                volume: opt_field!(quote, total_volume),
                quote_time: opt_field!(quote, quote_time),
                trade_time: opt_field!(quote, trade_time),
                security_status: opt_field!(quote, clone security_status),
                realtime: response.realtime,
                ..QuoteSummary::default()
            }
        }
        QuoteResponseObject::Future(response) => {
            let quote = response.quote;
            QuoteSummary {
                requested_symbol,
                symbol: response.symbol,
                asset_type: response.asset_main_type.map(|v| format!("{v:?}")),
                description: response.reference.and_then(|v| v.description),
                bid: opt_field!(quote, bid_price),
                ask: opt_field!(quote, ask_price),
                last: opt_field!(quote, last_price),
                mark: opt_field!(quote, mark),
                net_change: opt_field!(quote, net_change),
                net_percent_change: opt_field!(quote, future_percent_change),
                volume: opt_field!(quote, total_volume),
                quote_time: opt_field!(quote, quote_time),
                trade_time: opt_field!(quote, trade_time),
                security_status: opt_field!(quote, clone security_status),
                realtime: response.realtime,
                ..QuoteSummary::default()
            }
        }
        QuoteResponseObject::FutureOption(response) => {
            let quote = response.quote;
            QuoteSummary {
                requested_symbol,
                symbol: response.symbol,
                asset_type: response.asset_main_type.map(|v| format!("{v:?}")),
                description: response.reference.and_then(|v| v.description),
                bid: opt_field!(quote, bid_price),
                ask: opt_field!(quote, ask_price),
                last: opt_field!(quote, last_price),
                mark: opt_field!(quote, mark),
                net_change: opt_field!(quote, net_change),
                net_percent_change: opt_field!(quote, net_percent_change),
                volume: opt_field!(quote, total_volume),
                quote_time: opt_field!(quote, quote_time),
                trade_time: opt_field!(quote, trade_time),
                security_status: opt_field!(quote, clone security_status),
                realtime: response.realtime,
                ..QuoteSummary::default()
            }
        }
        QuoteResponseObject::Index(response) => {
            let quote = response.quote;
            QuoteSummary {
                requested_symbol,
                symbol: response.symbol,
                asset_type: response.asset_main_type.map(|v| format!("{v:?}")),
                description: response.reference.and_then(|v| v.description),
                last: opt_field!(quote, last_price),
                mark: opt_field!(quote, last_price),
                net_change: opt_field!(quote, net_change),
                net_percent_change: opt_field!(quote, net_percent_change),
                volume: opt_field!(quote, total_volume),
                trade_time: opt_field!(quote, trade_time),
                security_status: opt_field!(quote, clone security_status),
                realtime: response.realtime,
                ..QuoteSummary::default()
            }
        }
        QuoteResponseObject::Error(error) => QuoteSummary {
            requested_symbol,
            error: Some(QuoteErrorSummary {
                invalid_symbols: error.invalid_symbols,
                invalid_cusips: error.invalid_cusips,
                invalid_ssids: error.invalid_ssids,
            }),
            ..QuoteSummary::default()
        },
    }
}

/// Top-level JSON envelope returned by the `market quote` command.
#[derive(Debug, Serialize)]
struct QuoteOutput {
    /// The symbols that were requested, in the order the user supplied them.
    symbols: Vec<String>,
    /// Normalized quote data for each symbol, sorted alphabetically by `requested_symbol`.
    quotes: Vec<QuoteSummary>,
}

/// Token-optimized table-shaped JSON envelope returned by default for `market quote`.
#[derive(Debug, Serialize)]
struct QuoteRowsOutput {
    /// Ordered column names for every row value.
    columns: Vec<String>,
    /// Compact quote rows aligned with [`Self::columns`].
    rows: Vec<Vec<Value>>,
    /// Number of quote rows returned.
    #[serde(rename = "rowCount")]
    row_count: usize,
}

/// Token-optimized table-shaped JSON envelope returned by default for `market history`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
struct HistoryRowsOutput {
    /// Symbol returned by Schwab for this candle series.
    symbol: Option<String>,
    /// Ordered column names for every row value.
    columns: Vec<String>,
    /// Compact candle rows aligned with [`Self::columns`].
    rows: Vec<Vec<Value>>,
    /// Number of candle rows returned.
    #[serde(rename = "rowCount")]
    row_count: usize,
}

/// Flattened, agent-friendly view of any Schwab quote type.
///
/// All eight `QuoteResponseObject` variants collapse into this single struct.
/// Fields that don't apply to a given asset type are omitted from JSON output.
#[serde_with::skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub(crate) struct QuoteSummary {
    /// The symbol string the caller originally requested.
    pub(crate) requested_symbol: String,
    /// The canonical symbol returned by the API, which may differ from the requested symbol.
    symbol: Option<String>,
    /// Asset class as a debug-formatted string (e.g. `"Equity"`, `"Option"`, `"Future"`).
    asset_type: Option<String>,
    /// Human-readable name or description of the instrument.
    description: Option<String>,
    /// Exchange name (e.g. `"NASDAQ"`, `"CBOE"`). Not set for Forex, Future, FutureOption, or Index.
    exchange: Option<String>,
    /// Current best bid price. `None` for MutualFund and Index.
    bid: Option<schwab::Number>,
    /// Current best ask price. `None` for MutualFund and Index.
    ask: Option<schwab::Number>,
    /// Last traded price. For MutualFund this is the NAV.
    last: Option<schwab::Number>,
    /// Mark price. For Index this equals `last`; for MutualFund this equals NAV.
    mark: Option<schwab::Number>,
    /// Dollar change from the previous close.
    net_change: Option<schwab::Number>,
    /// Percent change from the previous close. For Future, sourced from `future_percent_change`.
    net_percent_change: Option<schwab::Number>,
    /// Total trading volume for the session.
    volume: Option<i64>,
    /// Timestamp of the most recent quote, in milliseconds since epoch. `None` for MutualFund and Index.
    quote_time: Option<i64>,
    /// Timestamp of the most recent trade, in milliseconds since epoch.
    trade_time: Option<i64>,
    /// Market session status string (e.g. `"Normal"`, `"Unknown"`).
    security_status: Option<String>,
    /// Whether the quote is real-time (`true`) or delayed/end-of-day (`false`).
    realtime: Option<bool>,
    /// Underlying symbol for options (e.g. `"AAPL"` for an AAPL option). Option variant only.
    underlying: Option<String>,
    /// Contract type as a debug-formatted string (`"Call"` or `"Put"`). Option variant only.
    put_call: Option<String>,
    /// Strike price of the option contract. Option variant only.
    strike_price: Option<schwab::Number>,
    /// Calendar days remaining until option expiration. Option variant only.
    days_to_expiration: Option<i32>,
    /// Populated when the API returns an error for this symbol instead of a valid quote.
    error: Option<QuoteErrorSummary>,
}

/// Error detail returned by the API when one or more requested symbols are unrecognized.
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
struct QuoteErrorSummary {
    /// Ticker symbols the API did not recognize.
    invalid_symbols: Option<Vec<String>>,
    /// CUSIP identifiers the API did not recognize.
    invalid_cusips: Option<Vec<String>>,
    /// SSID identifiers the API did not recognize.
    invalid_ssids: Option<Vec<i64>>,
}

#[cfg(test)]
mod tests;
