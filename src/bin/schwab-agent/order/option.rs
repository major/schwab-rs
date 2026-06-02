//! Option order builder using [`schwab::OrderBuilder`].
//!
//! Builds single-leg option orders for four actions (buy-to-open, sell-to-open,
//! buy-to-close, sell-to-close). Each action hardcodes the Schwab
//! [`schwab::Instruction`] to prevent accidental trade reversal.
//!
//! Order type is inferred from the presence of the `price` argument:
//! - `None` : market order
//! - `Some` : limit order

use crate::auth;
use crate::cli::OptionOrderArgs;
use crate::error::AppError;
use crate::order::workflow;
use crate::shared::{DurationChoice, SessionChoice, to_number};

// ---------------------------------------------------------------------------
// Action enum (local, independent of CLI layer)
// ---------------------------------------------------------------------------

/// Option order action.
///
/// Each variant hardcodes the Schwab [`schwab::Instruction`] to prevent
/// accidental trade reversal.
#[derive(Debug, Clone, Copy)]
pub enum OptionActionKind {
    /// Buy to open (new long position).
    BuyToOpen,
    /// Sell to open (new short position).
    SellToOpen,
    /// Buy to close (close short position).
    BuyToClose,
    /// Sell to close (close long position).
    SellToClose,
}

// ---------------------------------------------------------------------------
// Order builder
// ---------------------------------------------------------------------------

/// Builds a [`schwab::OrderBuilder`] for a single-leg option order.
///
/// The OCC symbol is passed verbatim without parsing or validation.
/// Order type is inferred from `price`: `None` for market, `Some` for limit.
///
/// # Errors
///
/// Returns [`AppError::OrderValidation`] when quantity or price are
/// non-positive, or when number conversion fails.
#[must_use = "returns the built order, does not place it"]
pub fn build_option_order(
    action: &OptionActionKind,
    occ_symbol: &str,
    qty: f64,
    price: Option<f64>,
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

    // --- number conversion ---
    let qty_num = to_number(qty)?;
    let price_num = price.map(to_number).transpose()?;

    // --- build order ---
    let order = match (action, price_num) {
        (OptionActionKind::BuyToOpen, None) => {
            schwab::OrderBuilder::option_buy_to_open_market(occ_symbol, qty_num)
        }
        (OptionActionKind::BuyToOpen, Some(p)) => {
            schwab::OrderBuilder::option_buy_to_open_limit(occ_symbol, qty_num, p)
        }
        (OptionActionKind::SellToOpen, None) => {
            schwab::OrderBuilder::option_sell_to_open_market(occ_symbol, qty_num)
        }
        (OptionActionKind::SellToOpen, Some(p)) => {
            schwab::OrderBuilder::option_sell_to_open_limit(occ_symbol, qty_num, p)
        }
        (OptionActionKind::BuyToClose, None) => {
            schwab::OrderBuilder::option_buy_to_close_market(occ_symbol, qty_num)
        }
        (OptionActionKind::BuyToClose, Some(p)) => {
            schwab::OrderBuilder::option_buy_to_close_limit(occ_symbol, qty_num, p)
        }
        (OptionActionKind::SellToClose, None) => {
            schwab::OrderBuilder::option_sell_to_close_market(occ_symbol, qty_num)
        }
        (OptionActionKind::SellToClose, Some(p)) => {
            schwab::OrderBuilder::option_sell_to_close_limit(occ_symbol, qty_num, p)
        }
    };

    Ok(order.session(session.into()).duration(duration.into()))
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

/// Executes an option order workflow.
///
/// Builds the order, determines the workflow mode from shared order flags, and
/// dispatches dry-run, preview, saved preview, or placement handling.
///
/// # Errors
///
pub async fn execute_option(
    action: OptionActionKind,
    args: OptionOrderArgs,
) -> Result<serde_json::Value, AppError> {
    let order = build_option_order(
        &action,
        &args.symbol,
        args.quantity,
        args.price,
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
    workflow::execute_order(&client, &order, mode, "order option").await
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
        action: &OptionActionKind,
        occ_symbol: &str,
        qty: f64,
        price: Option<f64>,
    ) -> serde_json::Value {
        let order = build_option_order(
            action,
            occ_symbol,
            qty,
            price,
            SessionChoice::Normal,
            DurationChoice::Day,
        )
        .expect("build should succeed");
        serde_json::to_value(&order).expect("serialize should succeed")
    }

    #[test]
    fn buy_to_open_market() {
        let json = build_json(
            &OptionActionKind::BuyToOpen,
            "AAPL  250117C00150000",
            1.0,
            None,
        );
        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_OPEN");
    }

    #[test]
    fn buy_to_open_limit() {
        let json = build_json(
            &OptionActionKind::BuyToOpen,
            "AAPL  250117C00150000",
            2.0,
            Some(5.50),
        );
        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_OPEN");
        assert!(json["price"].as_f64().is_some() || json["price"].as_str().is_some());
    }

    #[test]
    fn sell_to_open_hardcodes_instruction() {
        let json = build_json(
            &OptionActionKind::SellToOpen,
            "SPY   250620P00400000",
            3.0,
            Some(2.00),
        );
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL_TO_OPEN");
        assert_eq!(json["orderType"], "LIMIT");
    }

    #[test]
    fn sell_to_close_hardcodes_instruction() {
        let json = build_json(
            &OptionActionKind::SellToClose,
            "TSLA  250117P00200000",
            1.0,
            None,
        );
        assert_eq!(
            json["orderLegCollection"][0]["instruction"],
            "SELL_TO_CLOSE"
        );
        assert_eq!(json["orderType"], "MARKET");
    }

    #[test]
    fn buy_to_close_hardcodes_instruction() {
        let json = build_json(
            &OptionActionKind::BuyToClose,
            "MSFT  250620C00400000",
            5.0,
            Some(1.25),
        );
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_CLOSE");
        assert_eq!(json["orderType"], "LIMIT");
    }

    #[test]
    fn occ_symbol_passed_verbatim() {
        let json = build_json(&OptionActionKind::BuyToOpen, "WEIRD_OCC_SYMBOL", 1.0, None);
        assert_eq!(
            json["orderLegCollection"][0]["instrument"]["symbol"],
            "WEIRD_OCC_SYMBOL"
        );
    }

    #[test]
    fn zero_quantity_rejected() {
        let result = build_option_order(
            &OptionActionKind::BuyToOpen,
            "AAPL  250117C00150000",
            0.0,
            None,
            SessionChoice::Normal,
            DurationChoice::Day,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("quantity must be positive")
        );
    }

    #[test]
    fn negative_quantity_rejected() {
        let result = build_option_order(
            &OptionActionKind::SellToOpen,
            "SPY   250620P00400000",
            -5.0,
            None,
            SessionChoice::Normal,
            DurationChoice::Day,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("quantity must be positive")
        );
    }

    #[test]
    fn negative_price_rejected() {
        let result = build_option_order(
            &OptionActionKind::BuyToOpen,
            "AAPL  250117C00150000",
            1.0,
            Some(-1.0),
            SessionChoice::Normal,
            DurationChoice::Day,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("price must be positive")
        );
    }

    #[test]
    fn zero_price_rejected() {
        let result = build_option_order(
            &OptionActionKind::SellToClose,
            "AAPL  250117C00150000",
            1.0,
            Some(0.0),
            SessionChoice::Normal,
            DurationChoice::Day,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("price must be positive")
        );
    }

    #[test]
    fn session_and_duration_applied() {
        let order = build_option_order(
            &OptionActionKind::BuyToOpen,
            "AAPL  250117C00150000",
            1.0,
            Some(5.50),
            SessionChoice::Am,
            DurationChoice::GoodTillCancel,
        )
        .expect("build should succeed");
        let json = serde_json::to_value(&order).expect("serialize");
        assert_eq!(json["session"], "AM");
        assert_eq!(json["duration"], "GOOD_TILL_CANCEL");
    }

    #[tokio::test]
    async fn execute_option_returns_json() {
        use crate::cli::{CommonOrderArgs, OptionOrderArgs};

        let result = execute_option(
            OptionActionKind::BuyToOpen,
            OptionOrderArgs {
                symbol: "AAPL  250117C00150000".to_string(),
                quantity: 1.0,
                price: Some(5.50),
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
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_OPEN");
    }

    #[test]
    fn sell_to_open_market_produces_market_order() {
        let json = build_json(
            &OptionActionKind::SellToOpen,
            "SPY   250620P00400000",
            1.0,
            None,
        );
        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL_TO_OPEN");
    }

    #[test]
    fn buy_to_close_market_produces_market_order() {
        let json = build_json(
            &OptionActionKind::BuyToClose,
            "MSFT  250620C00400000",
            2.0,
            None,
        );
        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_CLOSE");
    }

    #[test]
    fn sell_to_close_limit_sets_price() {
        let json = build_json(
            &OptionActionKind::SellToClose,
            "TSLA  250117P00200000",
            1.0,
            Some(8.75),
        );
        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(
            json["orderLegCollection"][0]["instruction"],
            "SELL_TO_CLOSE"
        );
        assert!(json["price"].as_f64().is_some() || json["price"].as_str().is_some());
    }
}
