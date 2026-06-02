//! Order replacement handler.
//!
//! Replaces an existing order by building a new typed order payload from a
//! [`ReplaceOrderSpec`] and submitting it via the Schwab replace endpoint.
//! Includes post-replace verification via GET.

use serde_json::Value;

use crate::cli::{EquityArgs, OptionArgs, ReplaceArgs, ReplaceOrderSpec};
use crate::error::AppError;
use crate::order::equity::{self, EquityActionKind};
use crate::order::option::{self, OptionActionKind};
use crate::{account, auth, config, verify};

/// Executes `order replace`.
///
/// Replaces an existing order by building a new typed order payload from the
/// [`ReplaceOrderSpec`] and submitting it via the Schwab replace endpoint.
/// The mutable-operations guard is checked first, before any auth or API
/// calls.
///
/// # Errors
///
/// Returns `AppError` on mutable-guard failure, validation errors, auth
/// issues, account resolution failures, or Schwab API errors.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn execute_replace(args: &ReplaceArgs) -> Result<Value, AppError> {
    config::require_mutable_enabled()?;

    let provider = auth::provider()?;
    let token = provider.token().await?;
    let account_data = account::resolve_account(&token, &args.account).await?;
    let client = provider.client().await?;

    let order = build_replacement_order(&args.order_spec)?;

    let response = client
        .replace_order(&account_data.account_hash, args.order_id, &order)
        .await?;

    let order_json = serde_json::to_value(&order)?;
    let result = verify::verify_order(
        &client,
        &account_data.account_hash,
        response.order_id,
        "replace",
        response.location,
        Some(order_json),
    )
    .await;

    verify::action_value(result)
}

/// Builds a [`schwab::OrderBuilder`] from a [`ReplaceOrderSpec`].
///
/// Dispatches to the appropriate equity or option builder based on the
/// spec variant and action.
fn build_replacement_order(spec: &ReplaceOrderSpec) -> Result<schwab::OrderBuilder, AppError> {
    match spec {
        ReplaceOrderSpec::Equity(eq) => {
            let (action, args) = match eq {
                EquityArgs::Buy(a) => (EquityActionKind::Buy, a),
                EquityArgs::Sell(a) => (EquityActionKind::Sell, a),
                EquityArgs::SellShort(a) => (EquityActionKind::SellShort, a),
                EquityArgs::BuyToCover(a) => (EquityActionKind::BuyToCover, a),
            };
            equity::build_equity_order(
                &action,
                &args.symbol,
                args.quantity,
                args.price,
                args.stop,
                args.common.session,
                args.common.duration,
            )
        }
        ReplaceOrderSpec::Option(opt) => {
            let (action, args) = match opt {
                OptionArgs::BuyToOpen(a) => (OptionActionKind::BuyToOpen, a),
                OptionArgs::SellToOpen(a) => (OptionActionKind::SellToOpen, a),
                OptionArgs::BuyToClose(a) => (OptionActionKind::BuyToClose, a),
                OptionArgs::SellToClose(a) => (OptionActionKind::SellToClose, a),
            };
            option::build_option_order(
                &action,
                &args.symbol,
                args.quantity,
                args.price,
                args.common.session,
                args.common.duration,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{CommonOrderArgs, EquityOrderArgs, OptionOrderArgs};
    use crate::shared::{DurationChoice, SessionChoice};

    /// Returns default shared order args for test construction.
    fn test_common() -> CommonOrderArgs {
        CommonOrderArgs {
            account: None,
            session: SessionChoice::Normal,
            duration: DurationChoice::Day,
            dry_run: false,
            preview: false,
            save_preview: false,
            preview_first: false,
        }
    }

    #[test]
    fn builds_equity_buy_replacement() {
        let spec = ReplaceOrderSpec::Equity(EquityArgs::Buy(EquityOrderArgs {
            symbol: "AAPL".to_string(),
            quantity: 10.0,
            price: Some(150.0),
            stop: None,
            common: test_common(),
        }));

        let order = build_replacement_order(&spec).unwrap();
        let json = serde_json::to_value(&order).unwrap();
        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY");
        assert_eq!(
            json["orderLegCollection"][0]["instrument"]["symbol"],
            "AAPL"
        );
    }

    #[test]
    fn builds_equity_sell_short_replacement() {
        let spec = ReplaceOrderSpec::Equity(EquityArgs::SellShort(EquityOrderArgs {
            symbol: "TSLA".to_string(),
            quantity: 5.0,
            price: Some(200.0),
            stop: None,
            common: test_common(),
        }));

        let order = build_replacement_order(&spec).unwrap();
        let json = serde_json::to_value(&order).unwrap();
        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL_SHORT");
    }

    #[test]
    fn builds_option_sell_to_close_replacement() {
        let spec = ReplaceOrderSpec::Option(OptionArgs::SellToClose(OptionOrderArgs {
            symbol: "AAPL  250117C00150000".to_string(),
            quantity: 2.0,
            price: Some(3.50),
            common: test_common(),
        }));

        let order = build_replacement_order(&spec).unwrap();
        let json = serde_json::to_value(&order).unwrap();
        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(
            json["orderLegCollection"][0]["instruction"],
            "SELL_TO_CLOSE"
        );
    }

    #[test]
    fn builds_option_buy_to_open_market_replacement() {
        let spec = ReplaceOrderSpec::Option(OptionArgs::BuyToOpen(OptionOrderArgs {
            symbol: "SPY   250620P00400000".to_string(),
            quantity: 1.0,
            price: None,
            common: test_common(),
        }));

        let order = build_replacement_order(&spec).unwrap();
        let json = serde_json::to_value(&order).unwrap();
        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_OPEN");
    }

    #[test]
    fn equity_replacement_rejects_invalid_quantity() {
        let spec = ReplaceOrderSpec::Equity(EquityArgs::Sell(EquityOrderArgs {
            symbol: "SPY".to_string(),
            quantity: 0.0,
            price: None,
            stop: None,
            common: test_common(),
        }));

        let err = build_replacement_order(&spec).unwrap_err();
        assert!(err.to_string().contains("quantity must be positive"));
    }

    #[test]
    fn option_replacement_rejects_negative_price() {
        let spec = ReplaceOrderSpec::Option(OptionArgs::BuyToOpen(OptionOrderArgs {
            symbol: "SPY   250620P00400000".to_string(),
            quantity: 1.0,
            price: Some(-5.0),
            common: test_common(),
        }));

        let err = build_replacement_order(&spec).unwrap_err();
        assert!(err.to_string().contains("price must be positive"));
    }

    #[test]
    fn builds_equity_sell_replacement() {
        let spec = ReplaceOrderSpec::Equity(EquityArgs::Sell(EquityOrderArgs {
            symbol: "GOOG".to_string(),
            quantity: 15.0,
            price: None,
            stop: None,
            common: test_common(),
        }));

        let order = build_replacement_order(&spec).unwrap();
        let json = serde_json::to_value(&order).unwrap();
        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL");
    }

    #[test]
    fn builds_equity_buy_to_cover_replacement() {
        let spec = ReplaceOrderSpec::Equity(EquityArgs::BuyToCover(EquityOrderArgs {
            symbol: "MSFT".to_string(),
            quantity: 7.0,
            price: Some(300.0),
            stop: None,
            common: test_common(),
        }));

        let order = build_replacement_order(&spec).unwrap();
        let json = serde_json::to_value(&order).unwrap();
        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_COVER");
    }

    #[test]
    fn builds_option_sell_to_open_replacement() {
        let spec = ReplaceOrderSpec::Option(OptionArgs::SellToOpen(OptionOrderArgs {
            symbol: "AAPL  250117C00150000".to_string(),
            quantity: 3.0,
            price: Some(4.25),
            common: test_common(),
        }));

        let order = build_replacement_order(&spec).unwrap();
        let json = serde_json::to_value(&order).unwrap();
        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL_TO_OPEN");
    }

    #[test]
    fn builds_option_buy_to_close_replacement() {
        let spec = ReplaceOrderSpec::Option(OptionArgs::BuyToClose(OptionOrderArgs {
            symbol: "SPY   250620P00400000".to_string(),
            quantity: 2.0,
            price: None,
            common: test_common(),
        }));

        let order = build_replacement_order(&spec).unwrap();
        let json = serde_json::to_value(&order).unwrap();
        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_CLOSE");
    }
}
