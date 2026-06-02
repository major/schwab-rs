//! Unified order command dispatch.
//!
//! The CLI owns order argument definitions. This module wires those arguments
//! to the equity, option, raw, preview, replace, and lifecycle workflows.

pub(crate) mod equity;
pub(crate) mod lifecycle;
pub(crate) mod option;
pub(crate) mod preview;
pub(crate) mod replace;
pub(crate) mod workflow;

use serde_json::Value;

use crate::cli::{
    Cli, CommonOrderArgs, EquityArgs, EquityOrderArgs, OptionArgs, OptionOrderArgs, OrderCommand,
    PlaceFromPreviewArgs, PlaceRawArgs, PreviewRawArgs,
};
use crate::error::AppError;

use equity::EquityActionKind;
use option::OptionActionKind;

/// Handles all unified order subcommands.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn handle(cli: &Cli, command: &OrderCommand) -> Result<Value, AppError> {
    match command {
        OrderCommand::Equity(args) => dispatch_equity(args).await,
        OrderCommand::Option(args) => dispatch_option(args).await,
        OrderCommand::Get(args) => lifecycle::handle_get(cli, args).await,
        OrderCommand::Cancel(args) => lifecycle::handle_cancel(cli, args).await,
        OrderCommand::Replace(args) => replace::execute_replace(args).await,
        OrderCommand::Repeat(args) => lifecycle::handle_repeat(args).await,
        OrderCommand::PlaceFromPreview(args) => dispatch_place_from_preview(args).await,
        OrderCommand::PreviewRaw(args) => dispatch_preview_raw(args).await,
        OrderCommand::PlaceRaw(args) => dispatch_place_raw(args).await,
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn dispatch_equity(command: &EquityArgs) -> Result<Value, AppError> {
    let (action, args) = match command {
        EquityArgs::Buy(args) => (EquityActionKind::Buy, clone_equity_args(args)),
        EquityArgs::Sell(args) => (EquityActionKind::Sell, clone_equity_args(args)),
        EquityArgs::SellShort(args) => (EquityActionKind::SellShort, clone_equity_args(args)),
        EquityArgs::BuyToCover(args) => (EquityActionKind::BuyToCover, clone_equity_args(args)),
    };

    equity::execute_equity(action, args).await
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn dispatch_option(command: &OptionArgs) -> Result<Value, AppError> {
    let (action, args) = match command {
        OptionArgs::BuyToOpen(args) => (OptionActionKind::BuyToOpen, clone_option_args(args)),
        OptionArgs::SellToOpen(args) => (OptionActionKind::SellToOpen, clone_option_args(args)),
        OptionArgs::BuyToClose(args) => (OptionActionKind::BuyToClose, clone_option_args(args)),
        OptionArgs::SellToClose(args) => (OptionActionKind::SellToClose, clone_option_args(args)),
    };

    option::execute_option(action, args).await
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn dispatch_place_from_preview(args: &PlaceFromPreviewArgs) -> Result<Value, AppError> {
    let client = crate::auth::provider()?.client().await?;
    workflow::place_from_saved_preview(&client, &args.account, &args.digest).await
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn dispatch_preview_raw(args: &PreviewRawArgs) -> Result<Value, AppError> {
    workflow::execute_raw_preview(
        &args.account,
        &args.json,
        args.save_preview,
        "order preview-raw",
    )
    .await
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn dispatch_place_raw(args: &PlaceRawArgs) -> Result<Value, AppError> {
    let client = crate::auth::provider()?.client().await?;
    workflow::execute_raw_place(&client, &args.account, &args.json).await
}

fn clone_equity_args(args: &EquityOrderArgs) -> EquityOrderArgs {
    EquityOrderArgs {
        symbol: args.symbol.clone(),
        quantity: args.quantity,
        price: args.price,
        stop: args.stop,
        common: clone_common_args(&args.common),
    }
}

fn clone_option_args(args: &OptionOrderArgs) -> OptionOrderArgs {
    OptionOrderArgs {
        symbol: args.symbol.clone(),
        quantity: args.quantity,
        price: args.price,
        common: clone_common_args(&args.common),
    }
}

fn clone_common_args(args: &CommonOrderArgs) -> CommonOrderArgs {
    CommonOrderArgs {
        account: args.account.clone(),
        session: args.session,
        duration: args.duration,
        dry_run: args.dry_run,
        preview: args.preview,
        save_preview: args.save_preview,
        preview_first: args.preview_first,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{DurationChoice, SessionChoice};

    fn sample_common() -> CommonOrderArgs {
        CommonOrderArgs {
            account: Some("HASH".to_string()),
            session: SessionChoice::Normal,
            duration: DurationChoice::Day,
            dry_run: true,
            preview: false,
            save_preview: true,
            preview_first: false,
        }
    }

    #[test]
    fn clone_common_args_preserves_all_fields() {
        let orig = sample_common();
        let cloned = clone_common_args(&orig);
        assert_eq!(cloned.account, Some("HASH".to_string()));
        assert!(cloned.dry_run);
        assert!(!cloned.preview);
        assert!(cloned.save_preview);
        assert!(!cloned.preview_first);
    }

    #[test]
    fn clone_equity_args_preserves_all_fields() {
        let orig = EquityOrderArgs {
            symbol: "AAPL".to_string(),
            quantity: 10.0,
            price: Some(150.0),
            stop: Some(140.0),
            common: sample_common(),
        };
        let cloned = clone_equity_args(&orig);
        assert_eq!(cloned.symbol, "AAPL");
        assert_eq!(cloned.quantity, 10.0);
        assert_eq!(cloned.price, Some(150.0));
        assert_eq!(cloned.stop, Some(140.0));
        assert_eq!(cloned.common.account, Some("HASH".to_string()));
    }

    #[test]
    fn clone_option_args_preserves_all_fields() {
        let orig = OptionOrderArgs {
            symbol: "AAPL  250117C00150000".to_string(),
            quantity: 2.0,
            price: Some(3.50),
            common: sample_common(),
        };
        let cloned = clone_option_args(&orig);
        assert_eq!(cloned.symbol, "AAPL  250117C00150000");
        assert_eq!(cloned.quantity, 2.0);
        assert_eq!(cloned.price, Some(3.50));
        assert!(cloned.common.save_preview);
    }
}

#[cfg(test)]
mod builder {
    use crate::error::AppError;
    use crate::shared::to_number;
    use serde_json::Value;

    pub(crate) struct OptionLegSpec<'a> {
        pub(crate) underlying: &'a str,
        pub(crate) expiration: &'a str,
        pub(crate) strike: f64,
        pub(crate) quantity: u32,
        pub(crate) price: Option<f64>,
        pub(crate) put_call: schwab::PutCall,
    }

    pub(crate) struct OrderTiming {
        pub(crate) session: schwab::Session,
        pub(crate) duration: schwab::Duration,
    }

    pub(crate) fn build_single_leg(
        leg: OptionLegSpec<'_>,
        timing: OrderTiming,
        instruction: schwab::Instruction,
    ) -> Result<schwab::OrderBuilder, AppError> {
        let symbol = occ_symbol(leg.underlying, leg.expiration, leg.strike, leg.put_call)?;
        let quantity = to_number(f64::from(leg.quantity))?;
        let price = leg.price.map(to_number).transpose()?;
        let order = match (instruction, price) {
            (schwab::Instruction::BuyToOpen, None) => {
                schwab::OrderBuilder::option_buy_to_open_market(&symbol, quantity)
            }
            (schwab::Instruction::BuyToOpen, Some(price)) => {
                schwab::OrderBuilder::option_buy_to_open_limit(&symbol, quantity, price)
            }
            (schwab::Instruction::SellToOpen, None) => {
                schwab::OrderBuilder::option_sell_to_open_market(&symbol, quantity)
            }
            (schwab::Instruction::SellToOpen, Some(price)) => {
                schwab::OrderBuilder::option_sell_to_open_limit(&symbol, quantity, price)
            }
            (schwab::Instruction::BuyToClose, None) => {
                schwab::OrderBuilder::option_buy_to_close_market(&symbol, quantity)
            }
            (schwab::Instruction::BuyToClose, Some(price)) => {
                schwab::OrderBuilder::option_buy_to_close_limit(&symbol, quantity, price)
            }
            (schwab::Instruction::SellToClose, None) => {
                schwab::OrderBuilder::option_sell_to_close_market(&symbol, quantity)
            }
            (schwab::Instruction::SellToClose, Some(price)) => {
                schwab::OrderBuilder::option_sell_to_close_limit(&symbol, quantity, price)
            }
            _ => {
                return Err(AppError::OrderValidation(
                    "unsupported option preview test instruction".to_string(),
                ));
            }
        };

        Ok(order.session(timing.session).duration(timing.duration))
    }

    fn occ_symbol(
        underlying: &str,
        expiration: &str,
        strike: f64,
        put_call: schwab::PutCall,
    ) -> Result<String, AppError> {
        let compact_date = expiration
            .strip_prefix("20")
            .and_then(|rest| rest.get(0..8))
            .map(|value| value.replace('-', ""))
            .ok_or_else(|| {
                AppError::OrderValidation(format!(
                    "invalid expiration '{expiration}': expected YYYY-MM-DD"
                ))
            })?;
        let side = match put_call {
            schwab::PutCall::Call => 'C',
            schwab::PutCall::Put => 'P',
            _ => {
                return Err(AppError::OrderValidation(
                    "unsupported option preview test contract type".to_string(),
                ));
            }
        };
        let strike = (strike * 1000.0).round() as u64;

        Ok(format!("{underlying:<6}{compact_date}{side}{strike:08}"))
    }

    fn order_json(order: &schwab::OrderBuilder) -> Value {
        serde_json::to_value(order).unwrap()
    }

    #[test]
    fn build_single_leg_creates_market_call_order() {
        let order = build_single_leg(
            OptionLegSpec {
                underlying: "AAPL",
                expiration: "2025-01-17",
                strike: 150.0,
                quantity: 2,
                price: None,
                put_call: schwab::PutCall::Call,
            },
            OrderTiming {
                session: schwab::Session::Normal,
                duration: schwab::Duration::Day,
            },
            schwab::Instruction::BuyToOpen,
        )
        .unwrap();
        let value = order_json(&order);

        assert_eq!(value["orderType"], "MARKET");
        assert_eq!(value["session"], "NORMAL");
        assert_eq!(value["duration"], "DAY");
        assert_eq!(value["orderLegCollection"][0]["instruction"], "BUY_TO_OPEN");
        assert_eq!(
            value["orderLegCollection"][0]["instrument"]["symbol"],
            "AAPL  250117C00150000"
        );
    }

    #[test]
    fn build_single_leg_creates_limit_put_order() {
        let order = build_single_leg(
            OptionLegSpec {
                underlying: "MSFT",
                expiration: "2026-06-19",
                strike: 42.5,
                quantity: 3,
                price: Some(1.25),
                put_call: schwab::PutCall::Put,
            },
            OrderTiming {
                session: schwab::Session::Am,
                duration: schwab::Duration::GoodTillCancel,
            },
            schwab::Instruction::SellToClose,
        )
        .unwrap();
        let value = order_json(&order);

        assert_eq!(value["orderType"], "LIMIT");
        assert_eq!(value["session"], "AM");
        assert_eq!(value["duration"], "GOOD_TILL_CANCEL");
        assert_eq!(
            value["orderLegCollection"][0]["instruction"],
            "SELL_TO_CLOSE"
        );
        assert_eq!(
            value["orderLegCollection"][0]["instrument"]["symbol"],
            "MSFT  260619P00042500"
        );
        assert_eq!(
            value["price"],
            serde_json::to_value(to_number(1.25).unwrap()).unwrap()
        );
    }

    #[test]
    fn build_single_leg_rejects_unsupported_instruction() {
        let err = build_single_leg(
            OptionLegSpec {
                underlying: "AAPL",
                expiration: "2025-01-17",
                strike: 150.0,
                quantity: 1,
                price: None,
                put_call: schwab::PutCall::Call,
            },
            OrderTiming {
                session: schwab::Session::Normal,
                duration: schwab::Duration::Day,
            },
            schwab::Instruction::Buy,
        )
        .unwrap_err();

        assert!(
            err.to_string()
                .contains("unsupported option preview test instruction")
        );
    }

    #[test]
    fn build_single_leg_rejects_invalid_expiration() {
        let err = build_single_leg(
            OptionLegSpec {
                underlying: "AAPL",
                expiration: "25-01-17",
                strike: 150.0,
                quantity: 1,
                price: None,
                put_call: schwab::PutCall::Call,
            },
            OrderTiming {
                session: schwab::Session::Normal,
                duration: schwab::Duration::Day,
            },
            schwab::Instruction::BuyToOpen,
        )
        .unwrap_err();

        assert!(err.to_string().contains("expected YYYY-MM-DD"));
    }
}
