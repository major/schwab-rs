//! Order lifecycle commands: get, cancel, repeat.
//!
//! These commands let agents inspect and manage existing orders rather than
//! only creating new ones. Cancel and repeat commands include post-action
//! verification where Schwab returns a newly affected order to inspect.

use clap::Args;
use serde_json::Value;
use time::format_description::well_known::Rfc3339;
use time::{Date, Month, OffsetDateTime, Time};

use crate::auth;
use crate::cli::Cli;
use crate::error::AppError;
use crate::order::workflow;
use crate::raw;
use crate::verify;

/// Schwab order statuses treated as active/open by `order get` discovery mode.
const ACTIVE_ORDER_STATUSES: &[&str] = &[
    "AWAITING_PARENT_ORDER",
    "AWAITING_CONDITION",
    "AWAITING_STOP_CONDITION",
    "AWAITING_MANUAL_REVIEW",
    "AWAITING_UR_OUT",
    "AWAITING_RELEASE_TIME",
    "PENDING_ACTIVATION",
    "PENDING_CANCEL",
    "PENDING_REPLACE",
    "PENDING_ACKNOWLEDGEMENT",
    "PENDING_RECALL",
    "QUEUED",
    "WORKING",
    "NEW",
];

// ---------------------------------------------------------------------------
// CLI argument structs
// ---------------------------------------------------------------------------

/// Retrieve active or all orders, or a single order by ID.
///
/// `--account` accepts a raw account hash or a nickname (same resolution as
/// `account`). When omitted, active orders are queried across all linked
/// accounts. Add `--order-id` with `--account` to retrieve one exact order.
///
/// Discovery mode fetches Schwab's order list without a status filter and then
/// treats any returned status outside `ACTIVE_ORDER_STATUSES` as inactive. By
/// default, it searches active orders entered in the last 60 days. Use
/// `--symbol` to keep only orders whose legs include a matching instrument
/// symbol. Use `--include-inactive` to keep filled, canceled, rejected,
/// replaced, expired, and any other non-active statuses. Use `--recent` for the
/// last 24 hours, or `--from`/`--to` for a custom range.
#[derive(Debug, Args)]
pub struct OrderGetArgs {
    /// Account hash or nickname. Omit to query active orders across all accounts.
    #[arg(long)]
    pub account: Option<String>,

    /// Specific order ID to retrieve. Requires `--account`.
    #[arg(long = "order-id", alias = "order", requires = "account", value_parser = clap::value_parser!(i64).range(1..))]
    pub order_id: Option<i64>,

    /// Start of time range (YYYY-MM-DD or RFC3339, e.g. 2025-01-15).
    /// Defaults to 60 days ago.
    #[arg(long, conflicts_with = "order_id")]
    pub from: Option<String>,

    /// End of time range (YYYY-MM-DD or RFC3339, e.g. 2025-06-15).
    /// Defaults to now.
    #[arg(long, conflicts_with = "order_id")]
    pub to: Option<String>,

    /// Keep only orders with at least one leg for this instrument symbol.
    #[arg(long, conflicts_with = "order_id")]
    pub symbol: Option<String>,

    /// Get active orders from the last 24 hours. Overrides `--from`.
    #[arg(long, conflicts_with = "order_id")]
    pub recent: bool,

    /// Include filled, canceled, rejected, replaced, expired, and other inactive orders.
    #[arg(long, conflicts_with = "order_id")]
    pub include_inactive: bool,
}

/// Cancel an order by ID, with post-cancel verification.
///
/// After cancellation, a follow-up GET retrieves the order so the agent
/// can confirm the order reached CANCELED.
#[derive(Debug, Args)]
pub struct OrderCancelArgs {
    /// Account hash (required).
    #[arg(long)]
    pub account: String,

    /// Order ID to cancel. Positional compatibility form; prefer `--order-id`.
    #[arg(
        value_parser = clap::value_parser!(i64).range(1..),
        required_unless_present = "order_id_flag"
    )]
    pub order_id: Option<i64>,

    /// Order ID to cancel.
    #[arg(
        long = "order-id",
        value_name = "ORDER_ID",
        value_parser = clap::value_parser!(i64).range(1..)
    )]
    pub order_id_flag: Option<i64>,
}

/// Repeat an order by rebuilding its historical payload as a new order.
///
/// `--account` accepts a raw account hash or nickname and identifies both the
/// account used to fetch the source order and the account that receives the new
/// repeated order. Omit preview flags to place directly, use `--save-preview`
/// to store a tamper-evident digest, or use `--preview-first` to preview then
/// place automatically.
#[derive(Debug, Args)]
pub struct OrderRepeatArgs {
    /// Account hash or nickname for the source and target account.
    #[arg(short, long)]
    pub account: String,

    /// Order ID to repeat. Positional compatibility form; prefer `--order-id`.
    #[arg(
        value_parser = clap::value_parser!(i64).range(1..),
        required_unless_present = "order_id_flag"
    )]
    pub order_id: Option<i64>,

    /// Order ID to repeat.
    #[arg(
        long = "order-id",
        value_name = "ORDER_ID",
        value_parser = clap::value_parser!(i64).range(1..)
    )]
    pub order_id_flag: Option<i64>,

    /// Preview the rebuilt order and save a digest instead of placing it.
    #[arg(long, conflicts_with = "preview_first")]
    pub save_preview: bool,

    /// Preview the rebuilt order first, then place automatically if accepted.
    #[arg(long)]
    pub preview_first: bool,
}

impl OrderRepeatArgs {
    /// Returns the order ID supplied either positionally or via `--order-id`.
    pub fn order_id(&self) -> Result<i64, AppError> {
        resolve_order_id(self.order_id, self.order_id_flag, "repeat")
    }
}

impl OrderCancelArgs {
    /// Returns the order ID supplied either positionally or via `--order-id`.
    pub fn order_id(&self) -> Result<i64, AppError> {
        resolve_order_id(self.order_id, self.order_id_flag, "cancel")
    }
}

/// Resolves positional and named order ID inputs while preserving compatibility.
fn resolve_order_id(
    positional: Option<i64>,
    flag: Option<i64>,
    command: &str,
) -> Result<i64, AppError> {
    match (positional, flag) {
        (Some(positional), Some(flag)) if positional != flag => {
            Err(AppError::OrderValidation(format!(
                "order {command} received different positional ORDER_ID ({positional}) and --order-id ({flag}); use --order-id ORDER_ID"
            )))
        }
        (Some(order_id), _) | (_, Some(order_id)) => Ok(order_id),
        (None, None) => Err(AppError::OrderValidation(format!(
            "order {command} requires --order-id ORDER_ID"
        ))),
    }
}

/// Which side of a date range is being normalized.
#[derive(Clone, Copy, Debug)]
enum RangeBoundary {
    /// Inclusive start of a calendar day.
    Start,
    /// Inclusive end of a calendar day.
    End,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Retrieves active/all orders or a single order by account and order ID.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn handle_get(_cli: &Cli, args: &OrderGetArgs) -> Result<Value, AppError> {
    let provider = auth::provider()?;
    let token = provider.token().await?;

    if let Some(order_id) = args.order_id {
        let account = args
            .account
            .as_deref()
            .expect("clap requires account when order is present");
        let account_hash = crate::account::resolve_account(&token, account)
            .await?
            .account_hash;
        let client = provider.client().await?;
        let order = client.get_order(&account_hash, order_id).await?;
        return Ok(raw::sanitize_order(serde_json::to_value(&order)?));
    }

    handle_get_orders(&token, args).await
}

/// Retrieves discovery-mode orders across all accounts or a selected account.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn handle_get_orders(bearer_token: &str, args: &OrderGetArgs) -> Result<Value, AppError> {
    let (from_time, to_time) = normalize_get_range(args, OffsetDateTime::now_utc())?;
    let account_hash = match &args.account {
        Some(selector) => Some(
            crate::account::resolve_account(bearer_token, selector)
                .await?
                .account_hash,
        ),
        None => None,
    };

    let query = raw::OrderListQuery {
        from_entered_time: &from_time,
        to_entered_time: &to_time,
        max_results: None,
        status: None,
    };
    let raw_orders = raw::fetch_order_list(bearer_token, account_hash.as_deref(), &query).await?;
    let normalized = raw::normalize_order_list_response(raw_orders);

    render_order_discovery_response(normalized, args.include_inactive, args.symbol.as_deref())
}

/// Expands TRIGGER order children into the order list so active child stops
/// can be surfaced alongside regular SINGLE orders.
///
/// Schwab one-triggers-other (OTO) orders have `orderStrategyType: "TRIGGER"`
/// with working children inside `childOrderStrategies[]`. When the parent
/// fills, the child stop activates (status `WORKING`) but remains nested.
/// This function promotes those children so they can pass the active-order
/// and symbol filters independently.
#[must_use]
fn expand_trigger_child_orders(orders: Vec<Value>) -> Vec<Value> {
    let mut result = Vec::new();
    for order in orders {
        let is_trigger = order
            .get("orderStrategyType")
            .and_then(Value::as_str)
            .is_some_and(|ty| ty.eq_ignore_ascii_case("TRIGGER"));

        if is_trigger
            && let Some(children) = order.get("childOrderStrategies").and_then(Value::as_array)
        {
            for child in children {
                result.push(child.clone());
            }
        }

        result.push(order);
    }
    result
}

/// Renders discovery-mode order list output from a normalized Schwab response.
///
/// Non-array payloads are returned unchanged after sanitization. That preserves
/// unexpected response shapes instead of silently filtering them into an empty
/// order list because they do not have an order `status` field.
fn render_order_discovery_response(
    normalized: Value,
    include_inactive: bool,
    symbol: Option<&str>,
) -> Result<Value, AppError> {
    let Value::Array(mut orders) = normalized else {
        return Ok(raw::sanitize_order(normalized));
    };

    orders = expand_trigger_child_orders(orders);

    if !include_inactive {
        orders.retain(is_active_order);
    }

    if let Some(symbol) = symbol {
        orders.retain(|order| order_matches_symbol(order, symbol));
    }

    let order_value = raw::sanitize_order(Value::Array(orders));
    let count = order_value.as_array().map_or(0, Vec::len);
    let warnings = raw::order_activity_warnings(&order_value);

    let mut output = serde_json::json!({
        "orders": order_value,
        "count": count,
        "include_inactive": include_inactive,
        "active_statuses": ACTIVE_ORDER_STATUSES,
    });

    if !warnings.is_empty() {
        output["warnings"] = serde_json::to_value(warnings)?;
    }

    Ok(output)
}

/// Returns whether a raw Schwab order has an active/open status.
#[must_use]
fn is_active_order(order: &Value) -> bool {
    order
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|status| ACTIVE_ORDER_STATUSES.contains(&status))
}

/// Returns whether any order leg instrument symbol matches the requested symbol.
#[must_use]
fn order_matches_symbol(order: &Value, symbol: &str) -> bool {
    match order {
        Value::Object(fields) => fields.iter().any(|(key, value)| {
            if key == "orderLegCollection" {
                order_leg_collection_matches_symbol(value, symbol)
            } else {
                order_matches_symbol(value, symbol)
            }
        }),
        Value::Array(values) => values
            .iter()
            .any(|value| order_matches_symbol(value, symbol)),
        _ => false,
    }
}

/// Returns whether an `orderLegCollection` array includes the requested symbol.
#[must_use]
fn order_leg_collection_matches_symbol(order_legs: &Value, symbol: &str) -> bool {
    order_legs.as_array().is_some_and(|legs| {
        legs.iter().any(|leg| {
            leg.get("instrument")
                .and_then(|instrument| instrument.get("symbol"))
                .and_then(Value::as_str)
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(symbol))
        })
    })
}

/// Cancels an order and verifies the cancellation via a follow-up GET.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn handle_cancel(_cli: &Cli, args: &OrderCancelArgs) -> Result<Value, AppError> {
    crate::config::require_mutable_enabled()?;
    let provider = auth::provider()?;
    let token = provider.token().await?;
    let account_hash = crate::account::resolve_account(&token, &args.account)
        .await?
        .account_hash;
    let client = provider.client().await?;
    let order_id = args.order_id()?;
    client.cancel_order(&account_hash, order_id).await?;

    let result =
        verify::verify_order(&client, &account_hash, Some(order_id), "cancel", None, None).await;

    verify::action_value(result)
}

/// Rebuilds an existing order and routes it through the standard order workflow.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn handle_repeat(args: &OrderRepeatArgs) -> Result<Value, AppError> {
    let mode = workflow::determine_mode(
        Some(args.account.clone()),
        false,
        false,
        args.save_preview,
        args.preview_first,
    )?;
    if repeat_mode_places_order(&mode) {
        crate::config::require_mutable_enabled()?;
    }

    let provider = auth::provider()?;
    let token = provider.token().await?;
    let account_hash = crate::account::resolve_account(&token, &args.account)
        .await?
        .account_hash;
    let client = provider.client().await?;
    let order_id = args.order_id()?;
    let source_order = client.get_order(&account_hash, order_id).await?;
    let order = repeat_order_builder(&source_order, order_id)?;

    workflow::execute_order_with_account_hash(&client, &order, mode, &account_hash, "order repeat")
        .await
}

/// Returns whether a repeat mode can place an order.
#[must_use]
fn repeat_mode_places_order(mode: &workflow::OrderMode) -> bool {
    matches!(
        mode,
        workflow::OrderMode::Place { .. } | workflow::OrderMode::PreviewFirst { .. }
    )
}

/// Converts a Schwab historical order into a repeatable order builder.
fn repeat_order_builder(
    order: &schwab::Order,
    order_id: i64,
) -> Result<schwab::OrderBuilder, AppError> {
    schwab::OrderBuilder::try_from_order(order).map_err(|error| match error {
        schwab::Error::OrderConversion(message) => {
            AppError::OrderValidation(format!("cannot repeat order {order_id}: {message}"))
        }
        other => AppError::Schwab(other),
    })
}

/// Normalizes active-order date arguments to RFC3339 instants.
fn normalize_get_range(
    args: &OrderGetArgs,
    now: OffsetDateTime,
) -> Result<(String, String), AppError> {
    let to_time = match &args.to {
        Some(value) => parse_range_instant(value, RangeBoundary::End)?,
        None => now,
    };

    let from_time = if args.recent {
        now - time::Duration::hours(24)
    } else {
        match &args.from {
            Some(value) => parse_range_instant(value, RangeBoundary::Start)?,
            None => now - time::Duration::days(60),
        }
    };

    if from_time > to_time {
        return Err(AppError::OrderValidation(
            "order get --from must be before or equal to --to".to_string(),
        ));
    }

    Ok((format_rfc3339(from_time), format_rfc3339(to_time)))
}

/// Parses either a date-only value or a full RFC3339 instant.
fn parse_range_instant(value: &str, boundary: RangeBoundary) -> Result<OffsetDateTime, AppError> {
    if is_date_only(value) {
        return parse_date_only(value).and_then(|date| date_boundary(date, boundary));
    }

    OffsetDateTime::parse(value, &Rfc3339).map_err(|e| {
        AppError::OrderValidation(format!(
            "invalid order get date/time '{value}': expected YYYY-MM-DD or RFC3339 ({e})"
        ))
    })
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

/// Parses a YYYY-MM-DD date without enabling free-form local timezone inference.
fn parse_date_only(value: &str) -> Result<Date, AppError> {
    let year = value[0..4]
        .parse::<i32>()
        .map_err(|e| invalid_date(value, e))?;
    let month_number = value[5..7]
        .parse::<u8>()
        .map_err(|e| invalid_date(value, e))?;
    let day = value[8..10]
        .parse::<u8>()
        .map_err(|e| invalid_date(value, e))?;
    let month = Month::try_from(month_number).map_err(|e| invalid_date(value, e))?;

    Date::from_calendar_date(year, month, day).map_err(|e| invalid_date(value, e))
}

/// Converts a calendar date to the requested inclusive UTC boundary.
fn date_boundary(date: Date, boundary: RangeBoundary) -> Result<OffsetDateTime, AppError> {
    let time = match boundary {
        RangeBoundary::Start => Time::MIDNIGHT,
        RangeBoundary::End => Time::from_hms_nano(23, 59, 59, 999_999_999).map_err(|e| {
            AppError::OrderValidation(format!("failed to build end-of-day timestamp: {e}"))
        })?,
    };

    Ok(date.with_time(time).assume_utc())
}

/// Formats an instant as RFC3339, which the Schwab API expects.
fn format_rfc3339(value: OffsetDateTime) -> String {
    value.format(&Rfc3339).expect("RFC3339 format")
}

/// Builds a consistent validation error for invalid YYYY-MM-DD dates.
fn invalid_date<E: std::fmt::Display>(value: &str, error: E) -> AppError {
    AppError::OrderValidation(format!("invalid order get date '{value}': {error}"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use clap::Parser;
    use time::{Duration, OffsetDateTime};

    use super::{
        ACTIVE_ORDER_STATUSES, OrderGetArgs, expand_trigger_child_orders, is_active_order,
        normalize_get_range, parse_range_instant, render_order_discovery_response,
        repeat_mode_places_order, repeat_order_builder, resolve_order_id,
    };
    use crate::cli::{Cli, Command, OrderCommand};
    use crate::order::workflow;
    use crate::shared::to_number;

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn expect_order_get(command: Command) -> OrderGetArgs {
        match command {
            Command::Order(OrderCommand::Get(args)) => args,
            _ => panic!("expected order get command"),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn expect_order_cancel(command: Command) -> super::OrderCancelArgs {
        match command {
            Command::Order(OrderCommand::Cancel(args)) => args,
            _ => panic!("expected order cancel command"),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn expect_order_repeat(command: Command) -> super::OrderRepeatArgs {
        match command {
            Command::Order(OrderCommand::Repeat(args)) => args,
            _ => panic!("expected order repeat command"),
        }
    }

    #[test]
    fn parse_order_get_no_args_means_all_active_orders() {
        let cli = Cli::parse_from(["schwab-agent", "order", "get"]);
        let Command::Order(OrderCommand::Get(args)) = cli.command else {
            panic!("expected order get command");
        };
        assert!(args.account.is_none());
        assert!(args.order_id.is_none());
        assert!(args.from.is_none());
        assert!(args.to.is_none());
        assert!(args.symbol.is_none());
        assert!(!args.recent);
        assert!(!args.include_inactive);
    }

    #[test]
    fn parse_order_get_with_account_means_account_active_orders() {
        let cli = Cli::parse_from(["schwab-agent", "order", "get", "--account", "HASH123"]);
        let Command::Order(OrderCommand::Get(args)) = cli.command else {
            panic!("expected order get command");
        };
        assert_eq!(args.account.as_deref(), Some("HASH123"));
        assert!(args.order_id.is_none());
    }

    #[test]
    fn parse_order_get_recent() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "get",
            "--account",
            "HASH123",
            "--recent",
        ]);
        let Command::Order(OrderCommand::Get(args)) = cli.command else {
            panic!("expected order get command");
        };
        assert_eq!(args.account.as_deref(), Some("HASH123"));
        assert!(args.recent);
    }

    #[test]
    fn parse_order_get_with_symbol() {
        let cli = Cli::parse_from(["schwab-agent", "order", "get", "--symbol", "IBM"]);
        let Command::Order(OrderCommand::Get(args)) = cli.command else {
            panic!("expected order get command");
        };
        assert_eq!(args.symbol.as_deref(), Some("IBM"));
    }

    #[test]
    fn parse_order_get_include_inactive() {
        let cli = Cli::parse_from(["schwab-agent", "order", "get", "--include-inactive"]);
        let Command::Order(OrderCommand::Get(args)) = cli.command else {
            panic!("expected order get command");
        };
        assert!(args.account.is_none());
        assert!(args.order_id.is_none());
        assert!(args.include_inactive);
    }

    #[test]
    fn parse_order_get_with_time_range() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "get",
            "--from",
            "2025-01-01",
            "--to",
            "2025-06-01T12:00:00Z",
        ]);
        let Command::Order(OrderCommand::Get(args)) = cli.command else {
            panic!("expected order get command");
        };
        assert_eq!(args.from.as_deref(), Some("2025-01-01"));
        assert_eq!(args.to.as_deref(), Some("2025-06-01T12:00:00Z"));
    }

    #[test]
    fn parse_order_get_specific_order() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "get",
            "--account",
            "HASH123",
            "--order-id",
            "12345",
        ]);
        let args = expect_order_get(cli.command);
        assert_eq!(args.account.as_deref(), Some("HASH123"));
        assert_eq!(args.order_id, Some(12345));
    }

    #[test]
    fn parse_order_get_accepts_deprecated_order_alias() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "get",
            "--account",
            "HASH123",
            "--order",
            "12345",
        ]);
        let Command::Order(OrderCommand::Get(args)) = cli.command else {
            panic!("expected order get command");
        };
        assert_eq!(args.account.as_deref(), Some("HASH123"));
        assert_eq!(args.order_id, Some(12345));
    }

    #[test]
    fn parse_order_get_rejects_order_without_account() {
        assert!(
            Cli::try_parse_from(["schwab-agent", "order", "get", "--order-id", "12345"]).is_err()
        );
    }

    #[test]
    fn parse_order_get_rejects_discovery_flags_with_specific_order() {
        assert!(
            Cli::try_parse_from([
                "schwab-agent",
                "order",
                "get",
                "--account",
                "HASH123",
                "--order-id",
                "12345",
                "--include-inactive"
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "schwab-agent",
                "order",
                "get",
                "--account",
                "HASH123",
                "--order-id",
                "12345",
                "--recent"
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "schwab-agent",
                "order",
                "get",
                "--account",
                "HASH123",
                "--order-id",
                "12345",
                "--symbol",
                "IBM"
            ])
            .is_err()
        );
    }

    #[test]
    fn parse_order_list_is_removed() {
        assert!(Cli::try_parse_from(["schwab-agent", "order", "list"]).is_err());
    }

    #[test]
    fn active_order_statuses_include_requested_patterns() {
        assert!(
            ACTIVE_ORDER_STATUSES
                .iter()
                .any(|status| status.starts_with("AWAITING_"))
        );
        assert!(
            ACTIVE_ORDER_STATUSES
                .iter()
                .any(|status| status.starts_with("PENDING_"))
        );
        assert!(ACTIVE_ORDER_STATUSES.contains(&"PENDING_ACTIVATION"));
        assert!(ACTIVE_ORDER_STATUSES.contains(&"QUEUED"));
        assert!(ACTIVE_ORDER_STATUSES.contains(&"WORKING"));
        assert!(ACTIVE_ORDER_STATUSES.contains(&"NEW"));
    }

    #[test]
    fn is_active_order_uses_active_status_allowlist() {
        let active = serde_json::json!({ "status": "WORKING" });
        let inactive = serde_json::json!({ "status": "FILLED" });
        let unknown = serde_json::json!({ "status": "SOME_NEW_STATUS" });
        let missing = serde_json::json!({ "orderId": 12345 });

        assert!(is_active_order(&active));
        assert!(!is_active_order(&inactive));
        assert!(!is_active_order(&unknown));
        assert!(!is_active_order(&missing));
    }

    #[test]
    fn render_order_discovery_filters_array_orders() {
        let output = render_order_discovery_response(
            serde_json::json!([
                { "orderId": 1, "status": "WORKING" },
                { "orderId": 2, "status": "FILLED" },
                { "orderId": 3 }
            ]),
            false,
            None,
        )
        .unwrap();

        assert_eq!(output["count"], 1);
        assert_eq!(output["include_inactive"], false);
        assert_eq!(output["orders"][0]["orderId"], 1);
    }

    #[test]
    fn render_order_discovery_filters_by_symbol_case_insensitive() {
        let output = render_order_discovery_response(
            serde_json::json!([
                {
                    "orderId": 1,
                    "status": "WORKING",
                    "orderLegCollection": [{
                        "instruction": "SELL",
                        "instrument": { "symbol": "IBM" }
                    }]
                },
                {
                    "orderId": 2,
                    "status": "WORKING",
                    "orderLegCollection": [{
                        "instruction": "SELL",
                        "instrument": { "symbol": "AAPL" }
                    }]
                }
            ]),
            false,
            Some("ibm"),
        )
        .unwrap();

        assert_eq!(output["count"], 1);
        assert_eq!(output["orders"][0]["orderId"], 1);
        assert_eq!(
            output["orders"][0]["orderLegCollection"][0]["instrument"]["symbol"],
            "IBM"
        );
    }

    #[test]
    fn render_order_discovery_keeps_multi_leg_symbol_match() {
        let output = render_order_discovery_response(
            serde_json::json!([
                {
                    "orderId": 1,
                    "status": "WORKING",
                    "orderLegCollection": [
                        { "instrument": { "symbol": "MSFT" } },
                        { "instrument": { "symbol": "IBM" } }
                    ]
                }
            ]),
            false,
            Some("IBM"),
        )
        .unwrap();

        assert_eq!(output["count"], 1);
        assert_eq!(output["orders"][0]["orderId"], 1);
    }

    #[test]
    fn render_order_discovery_returns_empty_for_symbol_without_matches() {
        let output = render_order_discovery_response(
            serde_json::json!([
                {
                    "orderId": 1,
                    "status": "WORKING",
                    "orderLegCollection": [{ "instrument": { "symbol": "AAPL" } }]
                }
            ]),
            false,
            Some("IBM"),
        )
        .unwrap();

        assert_eq!(output["count"], 0);
        assert_eq!(output["orders"].as_array().unwrap().len(), 0);
        assert_eq!(output["include_inactive"], false);
        assert!(output.get("active_statuses").is_some());
    }

    #[test]
    fn render_order_discovery_preserves_non_array_payload() {
        let payload = serde_json::json!({
            "error": "unexpected shape",
            "status": "SOME_ENVELOPE_STATUS"
        });

        let output = render_order_discovery_response(payload.clone(), false, Some("IBM")).unwrap();

        assert_eq!(output, payload);
    }

    #[test]
    fn render_order_discovery_include_inactive_keeps_all_statuses() {
        let output = render_order_discovery_response(
            serde_json::json!([
                { "orderId": 1, "status": "WORKING" },
                { "orderId": 2, "status": "FILLED" },
                { "orderId": 3, "status": "CANCELED" }
            ]),
            true,
            None,
        )
        .unwrap();

        assert_eq!(output["count"], 3);
        assert_eq!(output["include_inactive"], true);
    }

    #[test]
    fn render_order_discovery_finds_symbol_in_nested_child_orders() {
        let output = render_order_discovery_response(
            serde_json::json!([
                {
                    "orderId": 1,
                    "status": "WORKING",
                    "childOrderStrategies": [{
                        "orderLegCollection": [{
                            "instrument": { "symbol": "TSLA" }
                        }]
                    }]
                },
                {
                    "orderId": 2,
                    "status": "WORKING",
                    "childOrderStrategies": [{
                        "orderLegCollection": [{
                            "instrument": { "symbol": "MSFT" }
                        }]
                    }]
                }
            ]),
            false,
            Some("tsla"),
        )
        .unwrap();

        assert_eq!(output["count"], 1);
        assert_eq!(output["orders"][0]["orderId"], 1);
    }

    #[test]
    fn render_order_discovery_ignores_malformed_order_leg_collections() {
        let output = render_order_discovery_response(
            serde_json::json!([
                {
                    "orderId": 1,
                    "status": "WORKING",
                    "orderLegCollection": { "instrument": { "symbol": "IBM" } }
                },
                {
                    "orderId": 2,
                    "status": "WORKING",
                    "orderLegCollection": [{ "instrument": { "cusip": "NO-SYMBOL" } }]
                }
            ]),
            false,
            Some("IBM"),
        )
        .unwrap();

        assert_eq!(output["count"], 0);
    }

    // --- expand_trigger_child_orders ---

    #[test]
    fn expand_trigger_child_orders_promotes_children_before_parent() {
        let orders = vec![serde_json::json!({
            "orderId": 1,
            "orderStrategyType": "TRIGGER",
            "status": "FILLED",
            "childOrderStrategies": [{
                "orderType": "STOP",
                "status": "WORKING",
                "stopPrice": 150.00,
                "orderLegCollection": [{
                    "instrument": { "symbol": "XYZ" },
                    "quantity": 100.0,
                    "instruction": "SELL"
                }]
            }]
        })];

        let expanded = expand_trigger_child_orders(orders);

        assert_eq!(expanded.len(), 2);
        // Child is promoted before the parent
        assert_eq!(expanded[0]["status"], "WORKING");
        assert_eq!(expanded[0]["orderType"], "STOP");
        assert_eq!(
            expanded[0]["orderLegCollection"][0]["instrument"]["symbol"],
            "XYZ"
        );
        // Parent follows
        assert_eq!(expanded[1]["orderId"], 1);
        assert_eq!(expanded[1]["status"], "FILLED");
    }

    #[test]
    fn expand_trigger_child_orders_handles_multiple_children() {
        let orders = vec![serde_json::json!({
            "orderId": 1,
            "orderStrategyType": "TRIGGER",
            "status": "FILLED",
            "childOrderStrategies": [
                { "status": "WORKING", "orderType": "STOP", "stopPrice": 150.00 },
                { "status": "CANCELED", "orderType": "LIMIT", "price": 145.00 }
            ]
        })];

        let expanded = expand_trigger_child_orders(orders);

        assert_eq!(expanded.len(), 3);
        assert_eq!(expanded[0]["orderType"], "STOP");
        assert_eq!(expanded[1]["orderType"], "LIMIT");
        assert_eq!(expanded[2]["orderId"], 1);
    }

    #[test]
    fn expand_trigger_child_orders_keeps_non_trigger_unchanged() {
        let orders = vec![
            serde_json::json!({
                "orderId": 1,
                "orderStrategyType": "SINGLE",
                "status": "WORKING"
            }),
            serde_json::json!({
                "orderId": 2,
                "orderStrategyType": "OCO",
                "status": "WORKING"
            }),
        ];

        let expanded = expand_trigger_child_orders(orders);

        assert_eq!(expanded.len(), 2);
        assert_eq!(expanded[0]["orderId"], 1);
        assert_eq!(expanded[1]["orderId"], 2);
    }

    #[test]
    fn expand_trigger_child_orders_ignores_missing_child_order_strategies() {
        let orders = vec![serde_json::json!({
            "orderId": 1,
            "orderStrategyType": "TRIGGER",
            "status": "WORKING"
        })];

        let expanded = expand_trigger_child_orders(orders);

        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0]["orderId"], 1);
    }

    #[test]
    fn expand_trigger_child_orders_case_insensitive_strategy_type() {
        let orders = vec![serde_json::json!({
            "orderId": 1,
            "orderStrategyType": "trigger",
            "status": "FILLED",
            "childOrderStrategies": [{
                "status": "WORKING",
                "orderType": "STOP"
            }]
        })];

        let expanded = expand_trigger_child_orders(orders);

        assert_eq!(expanded.len(), 2);
        assert_eq!(expanded[0]["status"], "WORKING");
    }

    // --- trigger child end-to-end via render_order_discovery_response ---

    #[test]
    fn render_order_discovery_surfaces_filled_trigger_with_working_child() {
        // This is the core fix: a TRIGGER order whose parent is FILLED
        // but has a WORKING child stop should surface the child.
        let output = render_order_discovery_response(
            serde_json::json!([{
                "orderId": 1,
                "orderStrategyType": "TRIGGER",
                "status": "FILLED",
                "orderLegCollection": [{
                    "instruction": "BUY",
                    "instrument": { "symbol": "XYZ" }
                }],
                "childOrderStrategies": [{
                    "status": "WORKING",
                    "orderType": "STOP",
                    "stopPrice": 150.00,
                    "orderLegCollection": [{
                        "instruction": "SELL",
                        "instrument": { "symbol": "XYZ" }
                    }]
                }]
            }]),
            false,
            None,
        )
        .unwrap();

        // The WORKING child is surfaced because expand_trigger_child_orders
        // promoted it before is_active_order filtered the FILLED parent.
        assert_eq!(output["count"], 1);
        assert_eq!(output["orders"][0]["status"], "WORKING");
        assert_eq!(output["orders"][0]["orderType"], "STOP");
    }

    #[test]
    fn render_order_discovery_keeps_active_trigger_parent_and_working_child() {
        // When a TRIGGER parent is still active (e.g. AWAITING_CONDITION),
        // both parent and working child should appear.
        let output = render_order_discovery_response(
            serde_json::json!([{
                "orderId": 1,
                "orderStrategyType": "TRIGGER",
                "status": "AWAITING_CONDITION",
                "childOrderStrategies": [{
                    "status": "WORKING",
                    "orderType": "STOP"
                }]
            }]),
            false,
            None,
        )
        .unwrap();

        assert_eq!(output["count"], 2);
        assert_eq!(output["orders"][0]["status"], "WORKING");
        assert_eq!(output["orders"][1]["status"], "AWAITING_CONDITION");
    }

    #[test]
    fn render_order_discovery_filters_trigger_child_by_symbol() {
        // Symbol filtering works on expanded trigger children.
        let output = render_order_discovery_response(
            serde_json::json!([{
                "orderId": 1,
                "orderStrategyType": "TRIGGER",
                "status": "FILLED",
                "childOrderStrategies": [{
                    "status": "WORKING",
                    "orderType": "STOP",
                    "orderLegCollection": [{
                        "instrument": { "symbol": "IBM" }
                    }]
                }]
            }]),
            false,
            Some("IBM"),
        )
        .unwrap();

        assert_eq!(output["count"], 1);
        assert_eq!(output["orders"][0]["orderType"], "STOP");
    }

    #[test]
    fn render_order_discovery_excludes_non_matching_trigger_child_by_symbol() {
        // When the child's symbol doesn't match, it's excluded.
        let output = render_order_discovery_response(
            serde_json::json!([{
                "orderId": 1,
                "orderStrategyType": "TRIGGER",
                "status": "FILLED",
                "childOrderStrategies": [{
                    "status": "WORKING",
                    "orderType": "STOP",
                    "orderLegCollection": [{
                        "instrument": { "symbol": "AAPL" }
                    }]
                }]
            }]),
            false,
            Some("IBM"),
        )
        .unwrap();

        assert_eq!(output["count"], 0);
    }

    #[test]
    fn parse_order_cancel() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "cancel",
            "--account",
            "HASH123",
            "67890",
        ]);
        let Command::Order(OrderCommand::Cancel(args)) = cli.command else {
            panic!("expected order cancel command");
        };
        assert_eq!(args.account, "HASH123");
        assert_eq!(args.order_id().unwrap(), 67890);
    }

    #[test]
    fn parse_order_cancel_with_order_id_flag() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "cancel",
            "--account",
            "HASH123",
            "--order-id",
            "67890",
        ]);
        let Command::Order(OrderCommand::Cancel(args)) = cli.command else {
            panic!("expected order cancel command");
        };
        assert_eq!(args.account, "HASH123");
        assert_eq!(args.order_id().unwrap(), 67890);
    }

    #[test]
    fn parse_order_cancel_rejects_missing_order_id() {
        assert!(
            Cli::try_parse_from(["schwab-agent", "order", "cancel", "--account", "HASH123"])
                .is_err()
        );
    }

    #[test]
    fn parse_order_cancel_rejects_mismatched_duplicate_order_ids() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "cancel",
            "--account",
            "HASH123",
            "67890",
            "--order-id",
            "12345",
        ]);
        let args = expect_order_cancel(cli.command);
        let err = args.order_id().unwrap_err();
        assert!(err.to_string().contains("different positional ORDER_ID"));
        assert!(err.to_string().contains("--order-id"));
    }

    #[test]
    fn parse_order_cancel_accepts_matching_duplicate_order_ids() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "cancel",
            "--account",
            "HASH123",
            "67890",
            "--order-id",
            "67890",
        ]);
        let args = expect_order_cancel(cli.command);
        assert_eq!(args.order_id().unwrap(), 67890);
    }

    #[test]
    fn resolve_order_id_rejects_missing_values() {
        let err = resolve_order_id(None, None, "cancel").unwrap_err();
        assert_eq!(err.to_string(), "order cancel requires --order-id ORDER_ID");
    }

    #[test]
    fn parse_order_repeat() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "repeat",
            "--account",
            "HASH123",
            "67890",
        ]);
        let Command::Order(OrderCommand::Repeat(args)) = cli.command else {
            panic!("expected order repeat command");
        };
        assert_eq!(args.account, "HASH123");
        assert_eq!(args.order_id().unwrap(), 67890);
        assert!(!args.save_preview);
        assert!(!args.preview_first);
    }

    #[test]
    fn parse_order_repeat_with_order_id_flag_and_preview() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "repeat",
            "--account",
            "Trading",
            "--order-id",
            "67890",
            "--save-preview",
        ]);
        let Command::Order(OrderCommand::Repeat(args)) = cli.command else {
            panic!("expected order repeat command");
        };
        assert_eq!(args.account, "Trading");
        assert_eq!(args.order_id().unwrap(), 67890);
        assert!(args.save_preview);
        assert!(!args.preview_first);
    }

    #[test]
    fn parse_order_repeat_rejects_missing_order_id() {
        assert!(
            Cli::try_parse_from(["schwab-agent", "order", "repeat", "--account", "HASH123"])
                .is_err()
        );
    }

    #[test]
    fn parse_order_repeat_rejects_mismatched_duplicate_order_ids() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "repeat",
            "--account",
            "HASH123",
            "67890",
            "--order-id",
            "12345",
        ]);
        let args = expect_order_repeat(cli.command);
        let err = args.order_id().unwrap_err();
        assert!(err.to_string().contains("different positional ORDER_ID"));
        assert!(err.to_string().contains("--order-id"));
    }

    #[test]
    fn parse_order_repeat_accepts_matching_duplicate_order_ids() {
        let cli = Cli::parse_from([
            "schwab-agent",
            "order",
            "repeat",
            "--account",
            "HASH123",
            "67890",
            "--order-id",
            "67890",
        ]);
        let args = expect_order_repeat(cli.command);
        assert_eq!(args.order_id().unwrap(), 67890);
    }

    #[test]
    fn parse_order_repeat_rejects_conflicting_preview_modes() {
        assert!(
            Cli::try_parse_from([
                "schwab-agent",
                "order",
                "repeat",
                "--account",
                "HASH123",
                "67890",
                "--save-preview",
                "--preview-first",
            ])
            .is_err()
        );
    }

    #[test]
    fn parse_order_get_and_cancel_reject_non_positive_order_ids() {
        assert!(
            Cli::try_parse_from([
                "schwab-agent",
                "order",
                "get",
                "--account",
                "HASH123",
                "--order-id",
                "0"
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "schwab-agent",
                "order",
                "cancel",
                "--account",
                "HASH123",
                "-1"
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "schwab-agent",
                "order",
                "cancel",
                "--account",
                "HASH123",
                "--order-id",
                "0"
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "schwab-agent",
                "order",
                "repeat",
                "--account",
                "HASH123",
                "--order-id",
                "0"
            ])
            .is_err()
        );
    }

    #[test]
    fn repeat_mode_places_only_for_mutable_modes() {
        assert!(!repeat_mode_places_order(&workflow::OrderMode::DryRun));
        assert!(!repeat_mode_places_order(
            &workflow::OrderMode::SavePreview {
                account: "HASH123".to_string(),
            }
        ));
        assert!(repeat_mode_places_order(&workflow::OrderMode::Place {
            account: "HASH123".to_string(),
        }));
        assert!(repeat_mode_places_order(
            &workflow::OrderMode::PreviewFirst {
                account: "HASH123".to_string(),
            }
        ));
    }

    #[test]
    fn repeat_order_builder_converts_supported_equity_order() {
        let source_order: schwab::Order = serde_json::from_value(serde_json::json!({
            "orderId": 67890,
            "orderType": "LIMIT",
            "session": "NORMAL",
            "duration": "DAY",
            "price": 150.25,
            "orderStrategyType": "SINGLE",
            "orderLegCollection": [{
                "instruction": "BUY",
                "quantity": 10,
                "instrument": {
                    "symbol": "AAPL",
                    "assetType": "EQUITY"
                }
            }]
        }))
        .unwrap();

        let order = repeat_order_builder(&source_order, 67890).unwrap();
        let output = serde_json::to_value(order).unwrap();

        assert_eq!(output["orderType"], "LIMIT");
        assert_eq!(
            output["price"],
            serde_json::to_value(to_number(150.25).unwrap()).unwrap()
        );
        assert_eq!(output["orderLegCollection"][0]["instruction"], "BUY");
        assert_eq!(
            output["orderLegCollection"][0]["instrument"]["symbol"],
            "AAPL"
        );
        assert!(output.get("orderId").is_none());
    }

    #[test]
    fn repeat_order_builder_maps_unsupported_order_conversion_to_validation() {
        let source_order: schwab::Order = serde_json::from_value(serde_json::json!({
            "orderId": 67890,
            "orderType": "LIMIT",
            "session": "NORMAL",
            "duration": "DAY",
            "price": 150.25,
            "orderStrategyType": "SINGLE"
        }))
        .unwrap();

        let error = repeat_order_builder(&source_order, 67890).unwrap_err();

        assert_eq!(error.code(), "order.validation_failed");
        assert!(error.to_string().contains("cannot repeat order 67890"));
    }

    #[test]
    fn normalize_get_range_expands_date_only_boundaries() {
        let now = OffsetDateTime::parse(
            "2026-06-15T12:00:00Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        let args = OrderGetArgs {
            account: None,
            order_id: None,
            from: Some("2026-05-28".to_string()),
            to: Some("2026-05-31".to_string()),
            symbol: None,
            recent: false,
            include_inactive: false,
        };

        let (from, to) = normalize_get_range(&args, now).unwrap();

        assert_eq!(from, "2026-05-28T00:00:00Z");
        assert_eq!(to, "2026-05-31T23:59:59.999999999Z");
    }

    #[test]
    fn normalize_get_range_allows_mixed_date_and_rfc3339() {
        let now = OffsetDateTime::parse(
            "2026-06-15T12:00:00Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        let args = OrderGetArgs {
            account: None,
            order_id: None,
            from: Some("2026-05-28".to_string()),
            to: Some("2026-05-31T12:30:00Z".to_string()),
            symbol: None,
            recent: false,
            include_inactive: false,
        };

        let (from, to) = normalize_get_range(&args, now).unwrap();

        assert_eq!(from, "2026-05-28T00:00:00Z");
        assert_eq!(to, "2026-05-31T12:30:00Z");
    }

    #[test]
    fn normalize_get_range_recent_overrides_from() {
        let now = OffsetDateTime::parse(
            "2026-06-15T12:00:00Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        let args = OrderGetArgs {
            account: None,
            order_id: None,
            from: Some("2026-05-28".to_string()),
            to: None,
            symbol: None,
            recent: true,
            include_inactive: false,
        };

        let (from, to) = normalize_get_range(&args, now).unwrap();

        assert_eq!(
            from,
            (now - Duration::hours(24))
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap()
        );
        assert_eq!(
            to,
            now.format(&time::format_description::well_known::Rfc3339)
                .unwrap()
        );
    }

    #[test]
    fn normalize_get_range_rejects_reversed_ranges() {
        let now = OffsetDateTime::parse(
            "2026-06-15T12:00:00Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        let args = OrderGetArgs {
            account: None,
            order_id: None,
            from: Some("2026-06-01".to_string()),
            to: Some("2026-05-31".to_string()),
            symbol: None,
            recent: false,
            include_inactive: false,
        };

        let error = normalize_get_range(&args, now).unwrap_err();
        assert!(error.to_string().contains("--from must be before"));
    }

    #[test]
    fn normalize_get_range_defaults_to_last_sixty_days() {
        let now = OffsetDateTime::parse(
            "2026-06-15T12:00:00Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        let args = OrderGetArgs {
            account: None,
            order_id: None,
            from: None,
            to: None,
            symbol: None,
            recent: false,
            include_inactive: false,
        };

        let (from, to) = normalize_get_range(&args, now).unwrap();

        assert_eq!(
            from,
            (now - Duration::days(60))
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap()
        );
        assert_eq!(
            to,
            now.format(&time::format_description::well_known::Rfc3339)
                .unwrap()
        );
    }

    #[test]
    fn parse_range_instant_rejects_invalid_date_and_timestamp() {
        let date_error = parse_range_instant("2026-02-30", super::RangeBoundary::Start)
            .unwrap_err()
            .to_string();
        let timestamp_error = parse_range_instant("not-a-date", super::RangeBoundary::End)
            .unwrap_err()
            .to_string();

        assert!(date_error.contains("invalid order get date"));
        assert!(timestamp_error.contains("invalid order get date/time"));
    }
}
