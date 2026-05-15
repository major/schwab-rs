use serde::Serialize;

use crate::models::Number;
use crate::models::enums::{
    Duration, Instruction, InstrumentAssetType, OrderStrategyType, OrderTypeRequest, Session,
};

/// Instrument description for order submission.
///
/// Contains only the fields the Schwab API requires when placing orders.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LegInstrument {
    symbol: String,
    asset_type: InstrumentAssetType,
}

/// A single leg in an order.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Leg {
    instruction: Instruction,
    quantity: Number,
    instrument: LegInstrument,
}

/// Convenience builder for constructing Schwab order payloads.
///
/// Produces a [`Serialize`]-able value that [`crate::Client::place_order`],
/// [`crate::Client::replace_order`], and [`crate::Client::preview_order`] accept
/// directly.
///
/// Each constructor sets sensible defaults (`NORMAL` session, `DAY`
/// duration, `SINGLE` strategy). Override them with the fluent setters.
///
/// # Examples
///
/// ```
/// use schwab::{Instruction, Number, OrderBuilder};
///
/// // Market buy 10 shares of AAPL
/// let quantity: Number = "10".parse().unwrap();
/// let order = OrderBuilder::equity_market("AAPL", Instruction::Buy, quantity);
///
/// // Limit buy 5 shares of MSFT at $400, good-til-cancel
/// let quantity: Number = "5".parse().unwrap();
/// let price: Number = "400".parse().unwrap();
/// let order = OrderBuilder::equity_limit("MSFT", Instruction::Buy, quantity, price)
///     .duration(schwab::Duration::GoodTillCancel);
/// ```
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBuilder {
    order_type: OrderTypeRequest,
    session: Session,
    duration: Duration,
    order_strategy_type: OrderStrategyType,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<Number>,
    order_leg_collection: Vec<Leg>,
}

impl OrderBuilder {
    /// Build a `MARKET` order for a single equity leg.
    pub fn equity_market(
        symbol: impl Into<String>,
        instruction: Instruction,
        quantity: Number,
    ) -> Self {
        Self::new(
            OrderTypeRequest::Market,
            symbol,
            instruction,
            quantity,
            None,
            None,
        )
    }

    /// Build a `LIMIT` order for a single equity leg.
    pub fn equity_limit(
        symbol: impl Into<String>,
        instruction: Instruction,
        quantity: Number,
        price: Number,
    ) -> Self {
        Self::new(
            OrderTypeRequest::Limit,
            symbol,
            instruction,
            quantity,
            Some(price),
            None,
        )
    }

    /// Build a `STOP` order for a single equity leg.
    pub fn equity_stop(
        symbol: impl Into<String>,
        instruction: Instruction,
        quantity: Number,
        stop_price: Number,
    ) -> Self {
        Self::new(
            OrderTypeRequest::Stop,
            symbol,
            instruction,
            quantity,
            None,
            Some(stop_price),
        )
    }

    /// Build a `STOP_LIMIT` order for a single equity leg.
    pub fn equity_stop_limit(
        symbol: impl Into<String>,
        instruction: Instruction,
        quantity: Number,
        price: Number,
        stop_price: Number,
    ) -> Self {
        Self::new(
            OrderTypeRequest::StopLimit,
            symbol,
            instruction,
            quantity,
            Some(price),
            Some(stop_price),
        )
    }

    /// Override the session (default: [`Session::Normal`]).
    pub fn session(mut self, session: Session) -> Self {
        self.session = session;
        self
    }

    /// Override the duration (default: [`Duration::Day`]).
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Override the order strategy type (default: [`OrderStrategyType::Single`]).
    pub fn order_strategy_type(mut self, strategy: OrderStrategyType) -> Self {
        self.order_strategy_type = strategy;
        self
    }

    /// Shared constructor for single-leg equity orders.
    fn new(
        order_type: OrderTypeRequest,
        symbol: impl Into<String>,
        instruction: Instruction,
        quantity: Number,
        price: Option<Number>,
        stop_price: Option<Number>,
    ) -> Self {
        Self {
            order_type,
            session: Session::Normal,
            duration: Duration::Day,
            order_strategy_type: OrderStrategyType::Single,
            price,
            stop_price,
            order_leg_collection: vec![Leg {
                instruction,
                quantity,
                instrument: LegInstrument {
                    symbol: symbol.into(),
                    asset_type: InstrumentAssetType::Equity,
                },
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::n;

    cfg_select! {
        feature = "decimal" => {
            fn expected_number(value: f64) -> serde_json::Value {
                serde_json::json!(n(value).to_string())
            }
        }
        _ => {
            fn expected_number(value: f64) -> serde_json::Value {
                serde_json::json!(value)
            }
        }
    }

    /// Market order serializes with required fields and no price.
    #[test]
    fn market_order_json() {
        let order = OrderBuilder::equity_market("AAPL", Instruction::Buy, n(10.0));
        let json: serde_json::Value = serde_json::to_value(&order).unwrap();

        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["session"], "NORMAL");
        assert_eq!(json["duration"], "DAY");
        assert_eq!(json["orderStrategyType"], "SINGLE");
        assert!(json.get("price").is_none());
        assert!(json.get("stopPrice").is_none());

        let legs = json["orderLegCollection"].as_array().unwrap();
        assert_eq!(legs.len(), 1);
        assert_eq!(legs[0]["instruction"], "BUY");
        assert_eq!(legs[0]["quantity"], expected_number(10.0));
        assert_eq!(legs[0]["instrument"]["symbol"], "AAPL");
        assert_eq!(legs[0]["instrument"]["assetType"], "EQUITY");
    }

    /// Limit order includes price and omits stopPrice.
    #[test]
    fn limit_order_json() {
        let order = OrderBuilder::equity_limit("MSFT", Instruction::Sell, n(5.0), n(400.50));
        let json: serde_json::Value = serde_json::to_value(&order).unwrap();

        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(json["price"], expected_number(400.50));
        assert!(json.get("stopPrice").is_none());
        assert_eq!(json["orderLegCollection"][0]["instruction"], "SELL");
        assert_eq!(
            json["orderLegCollection"][0]["quantity"],
            expected_number(5.0)
        );
    }

    /// Stop order includes stopPrice and omits price.
    #[test]
    fn stop_order_json() {
        let order = OrderBuilder::equity_stop("GOOG", Instruction::Sell, n(3.0), n(150.0));
        let json: serde_json::Value = serde_json::to_value(&order).unwrap();

        assert_eq!(json["orderType"], "STOP");
        assert_eq!(json["stopPrice"], expected_number(150.0));
        assert!(json.get("price").is_none());
    }

    /// Stop-limit order includes both price and stopPrice.
    #[test]
    fn stop_limit_order_json() {
        let order =
            OrderBuilder::equity_stop_limit("TSLA", Instruction::Buy, n(2.0), n(200.0), n(195.0));
        let json: serde_json::Value = serde_json::to_value(&order).unwrap();

        assert_eq!(json["orderType"], "STOP_LIMIT");
        assert_eq!(json["price"], expected_number(200.0));
        assert_eq!(json["stopPrice"], expected_number(195.0));
    }

    /// Fluent setters override defaults.
    #[test]
    fn fluent_setters() {
        let order = OrderBuilder::equity_market("SPY", Instruction::Buy, n(1.0))
            .session(Session::Am)
            .duration(Duration::GoodTillCancel)
            .order_strategy_type(OrderStrategyType::Trigger);

        let json: serde_json::Value = serde_json::to_value(&order).unwrap();
        assert_eq!(json["session"], "AM");
        assert_eq!(json["duration"], "GOOD_TILL_CANCEL");
        assert_eq!(json["orderStrategyType"], "TRIGGER");
    }
}
