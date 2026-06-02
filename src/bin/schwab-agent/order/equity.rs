//! Equity order builder using [`schwab::OrderBuilder`].
//!
//! Builds single-leg equity orders for four actions (buy, sell, sell-short,
//! buy-to-cover). Order type is inferred from the presence of `price` and
//! `stop` arguments rather than an explicit enum.

use crate::auth;
use crate::cli::EquityOrderArgs;
use crate::error::AppError;
use crate::order::workflow;
use crate::shared::{DurationChoice, SessionChoice, to_number};

// ---------------------------------------------------------------------------
// Action enum (local, independent of CLI layer)
// ---------------------------------------------------------------------------

/// Equity order action.
///
/// Each variant hardcodes the Schwab [`schwab::Instruction`] to prevent
/// accidental trade reversal.
#[derive(Debug, Clone, Copy)]
pub enum EquityActionKind {
    /// Buy shares (long entry).
    Buy,
    /// Sell shares (long exit).
    Sell,
    /// Sell short (short entry).
    SellShort,
    /// Buy to cover (short exit).
    BuyToCover,
}

// ---------------------------------------------------------------------------
// Order builder
// ---------------------------------------------------------------------------

/// Builds a [`schwab::OrderBuilder`] for an equity order.
///
/// Order type is inferred from flag presence:
/// - Both `None` : market
/// - `price` only : limit
/// - `stop` only  : stop
/// - Both `Some`  : stop-limit
///
/// Buy and Sell use high-level `OrderBuilder` helpers. SellShort and
/// BuyToCover use lower-level methods with an explicit instruction.
///
/// # Errors
///
/// Returns [`AppError::OrderValidation`] when quantity or prices are
/// non-positive, or when number conversion fails.
#[must_use = "returns the built order, does not place it"]
pub fn build_equity_order(
    action: &EquityActionKind,
    symbol: &str,
    qty: f64,
    price: Option<f64>,
    stop: Option<f64>,
    session: SessionChoice,
    duration: DurationChoice,
) -> Result<schwab::OrderBuilder, AppError> {
    // --- validation ---
    if qty <= 0.0 {
        return Err(AppError::OrderValidation(
            "quantity must be positive".to_string(),
        ));
    }
    if price.is_some_and(|p| p <= 0.0) {
        return Err(AppError::OrderValidation(
            "price must be positive".to_string(),
        ));
    }
    if stop.is_some_and(|s| s <= 0.0) {
        return Err(AppError::OrderValidation(
            "stop price must be positive".to_string(),
        ));
    }

    // --- number conversion ---
    let qty_num = to_number(qty)?;
    let price_num = price.map(to_number).transpose()?;
    let stop_num = stop.map(to_number).transpose()?;

    // --- build order ---
    let base = match action {
        EquityActionKind::Buy => match (price_num, stop_num) {
            (None, None) => schwab::OrderBuilder::market_buy(symbol, qty_num),
            (Some(p), None) => schwab::OrderBuilder::limit_buy(symbol, qty_num, p),
            (None, Some(s)) => schwab::OrderBuilder::stop_buy(symbol, qty_num, s),
            (Some(p), Some(s)) => schwab::OrderBuilder::stop_limit_buy(symbol, qty_num, p, s),
        },
        EquityActionKind::Sell => match (price_num, stop_num) {
            (None, None) => schwab::OrderBuilder::market_sell(symbol, qty_num),
            (Some(p), None) => schwab::OrderBuilder::limit_sell(symbol, qty_num, p),
            (None, Some(s)) => schwab::OrderBuilder::stop_sell(symbol, qty_num, s),
            (Some(p), Some(s)) => schwab::OrderBuilder::stop_limit_sell(symbol, qty_num, p, s),
        },
        EquityActionKind::SellShort => {
            let inst = schwab::Instruction::SellShort;
            match (price_num, stop_num) {
                (None, None) => schwab::OrderBuilder::equity_market(symbol, inst, qty_num),
                (Some(p), None) => schwab::OrderBuilder::equity_limit(symbol, inst, qty_num, p),
                (None, Some(s)) => schwab::OrderBuilder::equity_stop(symbol, inst, qty_num, s),
                (Some(p), Some(s)) => {
                    schwab::OrderBuilder::equity_stop_limit(symbol, inst, qty_num, p, s)
                }
            }
        }
        EquityActionKind::BuyToCover => {
            let inst = schwab::Instruction::BuyToCover;
            match (price_num, stop_num) {
                (None, None) => schwab::OrderBuilder::equity_market(symbol, inst, qty_num),
                (Some(p), None) => schwab::OrderBuilder::equity_limit(symbol, inst, qty_num, p),
                (None, Some(s)) => schwab::OrderBuilder::equity_stop(symbol, inst, qty_num, s),
                (Some(p), Some(s)) => {
                    schwab::OrderBuilder::equity_stop_limit(symbol, inst, qty_num, p, s)
                }
            }
        }
    };

    Ok(base.session(session.into()).duration(duration.into()))
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Executes an equity order workflow.
///
/// Builds the order, determines the workflow mode from shared order flags, and
/// dispatches dry-run, preview, saved preview, or placement handling.
///
/// # Errors
///
pub async fn execute_equity(
    action: EquityActionKind,
    args: EquityOrderArgs,
) -> Result<serde_json::Value, AppError> {
    let order = build_equity_order(
        &action,
        &args.symbol,
        args.quantity,
        args.price,
        args.stop,
        args.common.session,
        args.common.duration,
    )?;
    let mode = workflow::determine_mode(
        args.common.account,
        args.common.dry_run,
        args.common.preview,
        args.common.save_preview,
        args.common.preview_first,
    )?;
    if let workflow::OrderMode::DryRun = &mode {
        return Ok(serde_json::to_value(&order)?);
    }

    let client = auth::provider()?.client().await?;
    workflow::execute_order(&client, &order, mode, "order equity").await
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{DurationChoice, SessionChoice};

    /// Helper: build and serialize to JSON for assertion.
    fn build_json(
        action: &EquityActionKind,
        symbol: &str,
        qty: f64,
        price: Option<f64>,
        stop: Option<f64>,
    ) -> serde_json::Value {
        let order = build_equity_order(
            action,
            symbol,
            qty,
            price,
            stop,
            SessionChoice::Normal,
            DurationChoice::Day,
        )
        .expect("build should succeed");
        serde_json::to_value(&order).expect("serialize should succeed")
    }

    #[test]
    fn buy_market_produces_correct_order_type() {
        let json = build_json(&EquityActionKind::Buy, "AAPL", 10.0, None, None);
        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY");
        assert_eq!(
            json["orderLegCollection"][0]["instrument"]["symbol"],
            "AAPL"
        );
    }

    #[test]
    fn buy_limit_sets_price() {
        let json = build_json(&EquityActionKind::Buy, "MSFT", 5.0, Some(150.0), None);
        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY");
        assert!(json["price"].as_f64().is_some() || json["price"].as_str().is_some());
    }

    #[test]
    fn sell_stop_limit_sets_both_prices() {
        let json = build_json(
            &EquityActionKind::Sell,
            "GOOG",
            3.0,
            Some(100.0),
            Some(95.0),
        );
        assert_eq!(json["orderType"], "STOP_LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL");
    }

    #[test]
    fn sell_short_uses_sell_short_instruction() {
        let json = build_json(&EquityActionKind::SellShort, "AAPL", 5.0, Some(200.0), None);
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL_SHORT");
        assert_eq!(json["orderType"], "LIMIT");
    }

    #[test]
    fn buy_to_cover_uses_correct_instruction() {
        let json = build_json(
            &EquityActionKind::BuyToCover,
            "TSLA",
            2.0,
            None,
            Some(180.0),
        );
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_COVER");
        assert_eq!(json["orderType"], "STOP");
    }

    #[test]
    fn zero_quantity_rejected() {
        let err = build_equity_order(
            &EquityActionKind::Buy,
            "AAPL",
            0.0,
            None,
            None,
            SessionChoice::Normal,
            DurationChoice::Day,
        )
        .unwrap_err();
        assert!(err.to_string().contains("quantity must be positive"));
    }

    #[test]
    fn negative_price_rejected() {
        let err = build_equity_order(
            &EquityActionKind::Sell,
            "AAPL",
            10.0,
            Some(-5.0),
            None,
            SessionChoice::Normal,
            DurationChoice::Day,
        )
        .unwrap_err();
        assert!(err.to_string().contains("price must be positive"));
    }

    #[test]
    fn negative_stop_rejected() {
        let err = build_equity_order(
            &EquityActionKind::Buy,
            "SPY",
            1.0,
            None,
            Some(-10.0),
            SessionChoice::Normal,
            DurationChoice::Day,
        )
        .unwrap_err();
        assert!(err.to_string().contains("stop price must be positive"));
    }

    #[test]
    fn session_and_duration_applied() {
        let order = build_equity_order(
            &EquityActionKind::Buy,
            "AAPL",
            10.0,
            Some(150.0),
            None,
            SessionChoice::Am,
            DurationChoice::GoodTillCancel,
        )
        .expect("build should succeed");
        let json = serde_json::to_value(&order).expect("serialize");
        assert_eq!(json["session"], "AM");
        assert_eq!(json["duration"], "GOOD_TILL_CANCEL");
    }

    #[test]
    fn buy_stop_produces_stop_order() {
        let json = build_json(&EquityActionKind::Buy, "SPY", 10.0, None, Some(400.0));
        assert_eq!(json["orderType"], "STOP");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY");
    }

    #[test]
    fn buy_stop_limit_produces_stop_limit_order() {
        let json = build_json(
            &EquityActionKind::Buy,
            "SPY",
            10.0,
            Some(405.0),
            Some(400.0),
        );
        assert_eq!(json["orderType"], "STOP_LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY");
    }

    #[test]
    fn sell_market_produces_market_order() {
        let json = build_json(&EquityActionKind::Sell, "AAPL", 5.0, None, None);
        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL");
    }

    #[test]
    fn sell_limit_sets_price() {
        let json = build_json(&EquityActionKind::Sell, "MSFT", 3.0, Some(100.0), None);
        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL");
    }

    #[test]
    fn sell_stop_sets_stop_price() {
        let json = build_json(&EquityActionKind::Sell, "GOOG", 2.0, None, Some(150.0));
        assert_eq!(json["orderType"], "STOP");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL");
    }

    #[test]
    fn sell_short_market_produces_market_order() {
        let json = build_json(&EquityActionKind::SellShort, "TSLA", 10.0, None, None);
        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL_SHORT");
    }

    #[test]
    fn sell_short_stop_sets_stop_price() {
        let json = build_json(&EquityActionKind::SellShort, "SPY", 5.0, None, Some(380.0));
        assert_eq!(json["orderType"], "STOP");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL_SHORT");
    }

    #[test]
    fn sell_short_stop_limit_sets_both_prices() {
        let json = build_json(
            &EquityActionKind::SellShort,
            "QQQ",
            3.0,
            Some(350.0),
            Some(345.0),
        );
        assert_eq!(json["orderType"], "STOP_LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL_SHORT");
    }

    #[test]
    fn buy_to_cover_market_produces_market_order() {
        let json = build_json(&EquityActionKind::BuyToCover, "AAPL", 8.0, None, None);
        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_COVER");
    }

    #[test]
    fn buy_to_cover_limit_sets_price() {
        let json = build_json(
            &EquityActionKind::BuyToCover,
            "GOOG",
            4.0,
            Some(120.0),
            None,
        );
        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_COVER");
    }

    #[test]
    fn buy_to_cover_stop_limit_sets_both_prices() {
        let json = build_json(
            &EquityActionKind::BuyToCover,
            "MSFT",
            6.0,
            Some(200.0),
            Some(195.0),
        );
        assert_eq!(json["orderType"], "STOP_LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_COVER");
    }

    #[tokio::test]
    async fn execute_equity_dry_run_returns_json() {
        use crate::cli::{CommonOrderArgs, EquityOrderArgs};

        let result = execute_equity(
            EquityActionKind::Buy,
            EquityOrderArgs {
                symbol: "AAPL".to_string(),
                quantity: 10.0,
                price: Some(150.0),
                stop: None,
                common: CommonOrderArgs {
                    account: None,
                    session: SessionChoice::Normal,
                    duration: DurationChoice::Day,
                    dry_run: false,
                    preview: false,
                    save_preview: false,
                    preview_first: false,
                },
            },
        )
        .await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY");
    }
}
