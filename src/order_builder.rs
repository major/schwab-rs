use serde::Serialize;

use crate::models::Number;
use crate::models::enums::{
    ComplexOrderStrategyType, Duration, Instruction, InstrumentAssetType, OrderStrategyType,
    OrderType, OrderTypeRequest, PriceLinkBasis, PriceLinkType, Session, SpecialInstruction,
    StopPriceLinkBasis, StopPriceLinkType, StopType,
};
use crate::models::{AccountsInstrument, Order, OrderLegCollection};
use crate::{Error, Result};

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
/// The buy/sell constructors cover common equity orders, and the option
/// constructors cover common buy-to-open, sell-to-open, buy-to-close, and
/// sell-to-close orders without requiring callers to choose raw instruction
/// values. The `equity_*` constructors stay available for advanced equity
/// instructions such as short sales. Single orders set sensible defaults
/// (`NORMAL` session, `DAY` duration, `SINGLE` strategy). Override them with
/// the fluent setters, or compose builders with [`Self::one_cancels_other`]
/// and [`Self::first_triggers_second`].
///
/// Public constructor docs use consistent `Arguments`, `Defaults`, and
/// `Payload` sections so downstream tools can generate command help without
/// reverse-engineering the serialized JSON shape.
///
/// # Examples
///
/// ```
/// use schwab::{Instruction, Number, OrderBuilder};
///
/// // Market buy 10 shares of AAPL
/// let quantity: Number = "10".parse().unwrap();
/// let order = OrderBuilder::market_buy("AAPL", quantity);
///
/// // Limit buy 5 shares of MSFT at $400, good-til-cancel
/// let quantity: Number = "5".parse().unwrap();
/// let price: Number = "400".parse().unwrap();
/// let order = OrderBuilder::limit_buy("MSFT", quantity, price)
///     .duration(schwab::Duration::GoodTillCancel);
///
/// // Advanced instructions are still available when needed.
/// let quantity: Number = "2".parse().unwrap();
/// let order = OrderBuilder::equity_market("TSLA", Instruction::SellShort, quantity);
///
/// // Buy to open one option contract at market.
/// let quantity: Number = "1".parse().unwrap();
/// let order = OrderBuilder::option_buy_to_open_market("AAPL  260116C00150000", quantity);
///
/// // Compose two already-built orders into an OCO order.
/// let quantity: Number = "1".parse().unwrap();
/// let limit_price: Number = "140".parse().unwrap();
/// let stop_price: Number = "120".parse().unwrap();
/// let order = OrderBuilder::one_cancels_other(
///     OrderBuilder::limit_sell("AAPL", quantity, limit_price),
///     OrderBuilder::stop_sell("AAPL", quantity, stop_price),
/// );
///
/// // Buy shares, then place a bracket exit with profit target and stop loss.
/// let quantity: Number = "1".parse().unwrap();
/// let limit_price: Number = "160".parse().unwrap();
/// let stop_price: Number = "140".parse().unwrap();
/// let order = OrderBuilder::first_triggers_second(
///     OrderBuilder::market_buy("AAPL", quantity),
///     OrderBuilder::one_cancels_other(
///         OrderBuilder::limit_sell("AAPL", quantity, limit_price),
///         OrderBuilder::stop_sell("AAPL", quantity, stop_price),
///     ),
/// );
/// ```
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBuilder {
    #[serde(skip_serializing_if = "Option::is_none")]
    order_type: Option<OrderTypeRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session: Option<Session>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<Duration>,
    order_strategy_type: OrderStrategyType,
    #[serde(skip_serializing_if = "Option::is_none")]
    complex_order_strategy_type: Option<ComplexOrderStrategyType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price_link_basis: Option<PriceLinkBasis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price_link_type: Option<PriceLinkType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price_link_basis: Option<StopPriceLinkBasis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price_link_type: Option<StopPriceLinkType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price_offset: Option<Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_type: Option<StopType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    activation_price: Option<Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    special_instruction: Option<SpecialInstruction>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    order_leg_collection: Vec<Leg>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    child_order_strategies: Vec<OrderBuilder>,
}

impl OrderBuilder {
    /// Build a market buy order for a single equity leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Equity ticker symbol copied exactly as provided.
    /// - `quantity` - Number of shares to buy.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=MARKET`, `instruction=BUY`, and `assetType=EQUITY`.
    /// No `price` or `stopPrice` field is included.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "10".parse().unwrap();
    /// let order = OrderBuilder::market_buy("AAPL", quantity);
    /// ```
    pub fn market_buy(symbol: impl Into<String>, quantity: Number) -> Self {
        Self::equity_market(symbol, Instruction::Buy, quantity)
    }

    /// Build a market sell order for a single equity leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Equity ticker symbol copied exactly as provided.
    /// - `quantity` - Number of shares to sell.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=MARKET`, `instruction=SELL`, and `assetType=EQUITY`.
    /// No `price` or `stopPrice` field is included.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "10".parse().unwrap();
    /// let order = OrderBuilder::market_sell("AAPL", quantity);
    /// ```
    pub fn market_sell(symbol: impl Into<String>, quantity: Number) -> Self {
        Self::equity_market(symbol, Instruction::Sell, quantity)
    }

    /// Build a limit buy order for a single equity leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Equity ticker symbol copied exactly as provided.
    /// - `quantity` - Number of shares to buy.
    /// - `price` - Limit price for the buy order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=LIMIT`, `instruction=BUY`, `assetType=EQUITY`, and
    /// `price`. No `stopPrice` field is included.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "5".parse().unwrap();
    /// let price: Number = "150.00".parse().unwrap();
    /// let order = OrderBuilder::limit_buy("AAPL", quantity, price);
    /// ```
    pub fn limit_buy(symbol: impl Into<String>, quantity: Number, price: Number) -> Self {
        Self::equity_limit(symbol, Instruction::Buy, quantity, price)
    }

    /// Build a limit sell order for a single equity leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Equity ticker symbol copied exactly as provided.
    /// - `quantity` - Number of shares to sell.
    /// - `price` - Limit price for the sell order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=LIMIT`, `instruction=SELL`, `assetType=EQUITY`, and
    /// `price`. No `stopPrice` field is included.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "5".parse().unwrap();
    /// let price: Number = "150.00".parse().unwrap();
    /// let order = OrderBuilder::limit_sell("AAPL", quantity, price);
    /// ```
    pub fn limit_sell(symbol: impl Into<String>, quantity: Number, price: Number) -> Self {
        Self::equity_limit(symbol, Instruction::Sell, quantity, price)
    }

    /// Build a stop buy order for a single equity leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Equity ticker symbol copied exactly as provided.
    /// - `quantity` - Number of shares to buy.
    /// - `stop_price` - Stop price that activates the market buy order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=STOP`, `instruction=BUY`, `assetType=EQUITY`, and
    /// `stopPrice`. No `price` field is included.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "5".parse().unwrap();
    /// let stop_price: Number = "155.00".parse().unwrap();
    /// let order = OrderBuilder::stop_buy("AAPL", quantity, stop_price);
    /// ```
    pub fn stop_buy(symbol: impl Into<String>, quantity: Number, stop_price: Number) -> Self {
        Self::equity_stop(symbol, Instruction::Buy, quantity, stop_price)
    }

    /// Build a stop sell order for a single equity leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Equity ticker symbol copied exactly as provided.
    /// - `quantity` - Number of shares to sell.
    /// - `stop_price` - Stop price that activates the market sell order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=STOP`, `instruction=SELL`, `assetType=EQUITY`, and
    /// `stopPrice`. No `price` field is included.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "5".parse().unwrap();
    /// let stop_price: Number = "145.00".parse().unwrap();
    /// let order = OrderBuilder::stop_sell("AAPL", quantity, stop_price);
    /// ```
    pub fn stop_sell(symbol: impl Into<String>, quantity: Number, stop_price: Number) -> Self {
        Self::equity_stop(symbol, Instruction::Sell, quantity, stop_price)
    }

    /// Build a stop-limit buy order for a single equity leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Equity ticker symbol copied exactly as provided.
    /// - `quantity` - Number of shares to buy.
    /// - `price` - Limit price used after the stop activates.
    /// - `stop_price` - Stop price that activates the limit buy order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=STOP_LIMIT`, `instruction=BUY`, `assetType=EQUITY`,
    /// `price`, and `stopPrice`.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "5".parse().unwrap();
    /// let price: Number = "156.00".parse().unwrap();
    /// let stop_price: Number = "155.00".parse().unwrap();
    /// let order = OrderBuilder::stop_limit_buy("AAPL", quantity, price, stop_price);
    /// ```
    pub fn stop_limit_buy(
        symbol: impl Into<String>,
        quantity: Number,
        price: Number,
        stop_price: Number,
    ) -> Self {
        Self::equity_stop_limit(symbol, Instruction::Buy, quantity, price, stop_price)
    }

    /// Build a stop-limit sell order for a single equity leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Equity ticker symbol copied exactly as provided.
    /// - `quantity` - Number of shares to sell.
    /// - `price` - Limit price used after the stop activates.
    /// - `stop_price` - Stop price that activates the limit sell order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=STOP_LIMIT`, `instruction=SELL`, `assetType=EQUITY`,
    /// `price`, and `stopPrice`.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "5".parse().unwrap();
    /// let price: Number = "144.00".parse().unwrap();
    /// let stop_price: Number = "145.00".parse().unwrap();
    /// let order = OrderBuilder::stop_limit_sell("AAPL", quantity, price, stop_price);
    /// ```
    pub fn stop_limit_sell(
        symbol: impl Into<String>,
        quantity: Number,
        price: Number,
        stop_price: Number,
    ) -> Self {
        Self::equity_stop_limit(symbol, Instruction::Sell, quantity, price, stop_price)
    }

    /// Build a market buy-to-open order for a single option leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Schwab option symbol copied exactly as provided.
    /// - `quantity` - Number of option contracts to buy to open.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=MARKET`, `instruction=BUY_TO_OPEN`, and
    /// `assetType=OPTION`. No `price` or `stopPrice` field is included.
    /// The option symbol is not parsed, formatted, trimmed, or normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "1".parse().unwrap();
    /// let order = OrderBuilder::option_buy_to_open_market(
    ///     "AAPL  260116C00150000",
    ///     quantity,
    /// );
    /// ```
    pub fn option_buy_to_open_market(symbol: impl Into<String>, quantity: Number) -> Self {
        Self::option_market(symbol, Instruction::BuyToOpen, quantity)
    }

    /// Build a limit buy-to-open order for a single option leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Schwab option symbol copied exactly as provided.
    /// - `quantity` - Number of option contracts to buy to open.
    /// - `price` - Limit price for the option order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=LIMIT`, `instruction=BUY_TO_OPEN`,
    /// `assetType=OPTION`, and `price`. No `stopPrice` field is included.
    /// The option symbol is not parsed, formatted, trimmed, or normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "1".parse().unwrap();
    /// let price: Number = "2.50".parse().unwrap();
    /// let order = OrderBuilder::option_buy_to_open_limit(
    ///     "AAPL  260116C00150000",
    ///     quantity,
    ///     price,
    /// );
    /// ```
    pub fn option_buy_to_open_limit(
        symbol: impl Into<String>,
        quantity: Number,
        price: Number,
    ) -> Self {
        Self::option_limit(symbol, Instruction::BuyToOpen, quantity, price)
    }

    /// Build a market sell-to-open order for a single option leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Schwab option symbol copied exactly as provided.
    /// - `quantity` - Number of option contracts to sell to open.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=MARKET`, `instruction=SELL_TO_OPEN`, and
    /// `assetType=OPTION`. No `price` or `stopPrice` field is included.
    /// The option symbol is not parsed, formatted, trimmed, or normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "1".parse().unwrap();
    /// let order = OrderBuilder::option_sell_to_open_market(
    ///     "AAPL  260116C00150000",
    ///     quantity,
    /// );
    /// ```
    pub fn option_sell_to_open_market(symbol: impl Into<String>, quantity: Number) -> Self {
        Self::option_market(symbol, Instruction::SellToOpen, quantity)
    }

    /// Build a limit sell-to-open order for a single option leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Schwab option symbol copied exactly as provided.
    /// - `quantity` - Number of option contracts to sell to open.
    /// - `price` - Limit price for the option order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=LIMIT`, `instruction=SELL_TO_OPEN`,
    /// `assetType=OPTION`, and `price`. No `stopPrice` field is included.
    /// The option symbol is not parsed, formatted, trimmed, or normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "1".parse().unwrap();
    /// let price: Number = "2.50".parse().unwrap();
    /// let order = OrderBuilder::option_sell_to_open_limit(
    ///     "AAPL  260116C00150000",
    ///     quantity,
    ///     price,
    /// );
    /// ```
    pub fn option_sell_to_open_limit(
        symbol: impl Into<String>,
        quantity: Number,
        price: Number,
    ) -> Self {
        Self::option_limit(symbol, Instruction::SellToOpen, quantity, price)
    }

    /// Build a market buy-to-close order for a single option leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Schwab option symbol copied exactly as provided.
    /// - `quantity` - Number of option contracts to buy to close.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=MARKET`, `instruction=BUY_TO_CLOSE`, and
    /// `assetType=OPTION`. No `price` or `stopPrice` field is included.
    /// The option symbol is not parsed, formatted, trimmed, or normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "1".parse().unwrap();
    /// let order = OrderBuilder::option_buy_to_close_market(
    ///     "AAPL  260116C00150000",
    ///     quantity,
    /// );
    /// ```
    pub fn option_buy_to_close_market(symbol: impl Into<String>, quantity: Number) -> Self {
        Self::option_market(symbol, Instruction::BuyToClose, quantity)
    }

    /// Build a limit buy-to-close order for a single option leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Schwab option symbol copied exactly as provided.
    /// - `quantity` - Number of option contracts to buy to close.
    /// - `price` - Limit price for the option order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=LIMIT`, `instruction=BUY_TO_CLOSE`,
    /// `assetType=OPTION`, and `price`. No `stopPrice` field is included.
    /// The option symbol is not parsed, formatted, trimmed, or normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "1".parse().unwrap();
    /// let price: Number = "2.50".parse().unwrap();
    /// let order = OrderBuilder::option_buy_to_close_limit(
    ///     "AAPL  260116C00150000",
    ///     quantity,
    ///     price,
    /// );
    /// ```
    pub fn option_buy_to_close_limit(
        symbol: impl Into<String>,
        quantity: Number,
        price: Number,
    ) -> Self {
        Self::option_limit(symbol, Instruction::BuyToClose, quantity, price)
    }

    /// Build a market sell-to-close order for a single option leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Schwab option symbol copied exactly as provided.
    /// - `quantity` - Number of option contracts to sell to close.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=MARKET`, `instruction=SELL_TO_CLOSE`, and
    /// `assetType=OPTION`. No `price` or `stopPrice` field is included.
    /// The option symbol is not parsed, formatted, trimmed, or normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "1".parse().unwrap();
    /// let order = OrderBuilder::option_sell_to_close_market(
    ///     "AAPL  260116C00150000",
    ///     quantity,
    /// );
    /// ```
    pub fn option_sell_to_close_market(symbol: impl Into<String>, quantity: Number) -> Self {
        Self::option_market(symbol, Instruction::SellToClose, quantity)
    }

    /// Build a limit sell-to-close order for a single option leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Schwab option symbol copied exactly as provided.
    /// - `quantity` - Number of option contracts to sell to close.
    /// - `price` - Limit price for the option order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=LIMIT`, `instruction=SELL_TO_CLOSE`,
    /// `assetType=OPTION`, and `price`. No `stopPrice` field is included.
    /// The option symbol is not parsed, formatted, trimmed, or normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "1".parse().unwrap();
    /// let price: Number = "2.50".parse().unwrap();
    /// let order = OrderBuilder::option_sell_to_close_limit(
    ///     "AAPL  260116C00150000",
    ///     quantity,
    ///     price,
    /// );
    /// ```
    pub fn option_sell_to_close_limit(
        symbol: impl Into<String>,
        quantity: Number,
        price: Number,
    ) -> Self {
        Self::option_limit(symbol, Instruction::SellToClose, quantity, price)
    }

    /// Compose two orders into a one-cancels-other (`OCO`) strategy.
    ///
    /// # Arguments
    ///
    /// - `first_order` - First child order in the OCO group.
    /// - `second_order` - Second child order in the OCO group.
    ///
    /// # Defaults
    ///
    /// The parent strategy is [`OrderStrategyType::Oco`]. Child orders keep
    /// their own sessions, durations, order types, legs, and prices.
    ///
    /// # Payload
    ///
    /// Emits a parent with `orderStrategyType=OCO` and a
    /// `childOrderStrategies` array containing the two provided orders. The
    /// parent omits `orderType`, `session`, `duration`, `price`, `stopPrice`,
    /// and `orderLegCollection` so it does not invent simple-order fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "1".parse().unwrap();
    /// let limit_price: Number = "155.00".parse().unwrap();
    /// let stop_price: Number = "145.00".parse().unwrap();
    /// let order = OrderBuilder::one_cancels_other(
    ///     OrderBuilder::limit_sell("AAPL", quantity, limit_price),
    ///     OrderBuilder::stop_sell("AAPL", quantity, stop_price),
    /// );
    /// ```
    pub fn one_cancels_other(first_order: Self, second_order: Self) -> Self {
        Self {
            order_type: None,
            session: None,
            duration: None,
            order_strategy_type: OrderStrategyType::Oco,
            complex_order_strategy_type: None,
            price: None,
            price_link_basis: None,
            price_link_type: None,
            stop_price: None,
            stop_price_link_basis: None,
            stop_price_link_type: None,
            stop_price_offset: None,
            stop_type: None,
            activation_price: None,
            special_instruction: None,
            order_leg_collection: Vec::new(),
            child_order_strategies: vec![first_order, second_order],
        }
    }

    /// Compose an order that triggers a second order after the first fills.
    ///
    /// # Arguments
    ///
    /// - `first_order` - Parent order that must fill first.
    /// - `second_order` - Child order sent by Schwab after the first fills.
    ///
    /// # Defaults
    ///
    /// Changes the first order strategy to [`OrderStrategyType::Trigger`]. The
    /// second order keeps its own default or overridden fields.
    ///
    /// # Payload
    ///
    /// Emits the first order as the parent with `orderStrategyType=TRIGGER`
    /// and appends the second order to `childOrderStrategies`. The parent keeps
    /// its original `orderType`, `session`, `duration`, price fields, and legs.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder};
    ///
    /// let quantity: Number = "1".parse().unwrap();
    /// let stop_price: Number = "145.00".parse().unwrap();
    /// let order = OrderBuilder::first_triggers_second(
    ///     OrderBuilder::market_buy("AAPL", quantity),
    ///     OrderBuilder::stop_sell("AAPL", quantity, stop_price),
    /// );
    ///
    /// // A bracket order triggers an OCO exit after the entry order fills.
    /// let quantity: Number = "1".parse().unwrap();
    /// let limit_price: Number = "160.00".parse().unwrap();
    /// let stop_price: Number = "140.00".parse().unwrap();
    /// let bracket = OrderBuilder::first_triggers_second(
    ///     OrderBuilder::market_buy("AAPL", quantity),
    ///     OrderBuilder::one_cancels_other(
    ///         OrderBuilder::limit_sell("AAPL", quantity, limit_price),
    ///         OrderBuilder::stop_sell("AAPL", quantity, stop_price),
    ///     ),
    /// );
    /// ```
    pub fn first_triggers_second(mut first_order: Self, second_order: Self) -> Self {
        first_order.order_strategy_type = OrderStrategyType::Trigger;
        first_order.child_order_strategies.push(second_order);
        first_order
    }

    /// Build a `MARKET` order for a single equity leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Equity ticker symbol copied exactly as provided.
    /// - `instruction` - Equity instruction to place on the leg.
    /// - `quantity` - Number of shares for the leg.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=MARKET`, the provided `instruction`, and
    /// `assetType=EQUITY`. No `price` or `stopPrice` field is included.
    ///
    /// # Caution
    ///
    /// This lower-level constructor trusts the provided instruction. Prefer
    /// [`Self::market_buy`] or [`Self::market_sell`] for common buy/sell flows.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Instruction, Number, OrderBuilder};
    ///
    /// let quantity: Number = "2".parse().unwrap();
    /// let order = OrderBuilder::equity_market("TSLA", Instruction::SellShort, quantity);
    /// ```
    pub fn equity_market(
        symbol: impl Into<String>,
        instruction: Instruction,
        quantity: Number,
    ) -> Self {
        Self::single_leg(
            OrderTypeRequest::Market,
            symbol,
            instruction,
            InstrumentAssetType::Equity,
            quantity,
            None,
            None,
        )
    }

    /// Build a `LIMIT` order for a single equity leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Equity ticker symbol copied exactly as provided.
    /// - `instruction` - Equity instruction to place on the leg.
    /// - `quantity` - Number of shares for the leg.
    /// - `price` - Limit price for the order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=LIMIT`, the provided `instruction`,
    /// `assetType=EQUITY`, and `price`. No `stopPrice` field is included.
    ///
    /// # Caution
    ///
    /// This lower-level constructor trusts the provided instruction. Prefer
    /// [`Self::limit_buy`] or [`Self::limit_sell`] for common buy/sell flows.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Instruction, Number, OrderBuilder};
    ///
    /// let quantity: Number = "2".parse().unwrap();
    /// let price: Number = "250.00".parse().unwrap();
    /// let order = OrderBuilder::equity_limit(
    ///     "TSLA",
    ///     Instruction::SellShort,
    ///     quantity,
    ///     price,
    /// );
    /// ```
    pub fn equity_limit(
        symbol: impl Into<String>,
        instruction: Instruction,
        quantity: Number,
        price: Number,
    ) -> Self {
        Self::single_leg(
            OrderTypeRequest::Limit,
            symbol,
            instruction,
            InstrumentAssetType::Equity,
            quantity,
            Some(price),
            None,
        )
    }

    /// Build a `STOP` order for a single equity leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Equity ticker symbol copied exactly as provided.
    /// - `instruction` - Equity instruction to place on the leg.
    /// - `quantity` - Number of shares for the leg.
    /// - `stop_price` - Stop price that activates the market order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=STOP`, the provided `instruction`,
    /// `assetType=EQUITY`, and `stopPrice`. No `price` field is included.
    ///
    /// # Caution
    ///
    /// This lower-level constructor trusts the provided instruction. Prefer
    /// [`Self::stop_buy`] or [`Self::stop_sell`] for common buy/sell flows.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Instruction, Number, OrderBuilder};
    ///
    /// let quantity: Number = "2".parse().unwrap();
    /// let stop_price: Number = "245.00".parse().unwrap();
    /// let order = OrderBuilder::equity_stop(
    ///     "TSLA",
    ///     Instruction::SellShort,
    ///     quantity,
    ///     stop_price,
    /// );
    /// ```
    pub fn equity_stop(
        symbol: impl Into<String>,
        instruction: Instruction,
        quantity: Number,
        stop_price: Number,
    ) -> Self {
        Self::single_leg(
            OrderTypeRequest::Stop,
            symbol,
            instruction,
            InstrumentAssetType::Equity,
            quantity,
            None,
            Some(stop_price),
        )
    }

    /// Build a `STOP_LIMIT` order for a single equity leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Equity ticker symbol copied exactly as provided.
    /// - `instruction` - Equity instruction to place on the leg.
    /// - `quantity` - Number of shares for the leg.
    /// - `price` - Limit price used after the stop activates.
    /// - `stop_price` - Stop price that activates the limit order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=STOP_LIMIT`, the provided `instruction`,
    /// `assetType=EQUITY`, `price`, and `stopPrice`.
    ///
    /// # Caution
    ///
    /// This lower-level constructor trusts the provided instruction. Prefer
    /// [`Self::stop_limit_buy`] or [`Self::stop_limit_sell`] for common
    /// buy/sell flows.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Instruction, Number, OrderBuilder};
    ///
    /// let quantity: Number = "2".parse().unwrap();
    /// let price: Number = "244.00".parse().unwrap();
    /// let stop_price: Number = "245.00".parse().unwrap();
    /// let order = OrderBuilder::equity_stop_limit(
    ///     "TSLA",
    ///     Instruction::SellShort,
    ///     quantity,
    ///     price,
    ///     stop_price,
    /// );
    /// ```
    pub fn equity_stop_limit(
        symbol: impl Into<String>,
        instruction: Instruction,
        quantity: Number,
        price: Number,
        stop_price: Number,
    ) -> Self {
        Self::single_leg(
            OrderTypeRequest::StopLimit,
            symbol,
            instruction,
            InstrumentAssetType::Equity,
            quantity,
            Some(price),
            Some(stop_price),
        )
    }

    /// Build a `MARKET` order for a single option leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Schwab option symbol copied exactly as provided.
    /// - `instruction` - Option instruction to place on the leg.
    /// - `quantity` - Number of option contracts for the leg.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=MARKET`, the provided `instruction`, and
    /// `assetType=OPTION`. No `price` or `stopPrice` field is included. The
    /// option symbol is not parsed, formatted, trimmed, or normalized.
    ///
    /// # Caution
    ///
    /// This lower-level constructor trusts the provided instruction. Prefer
    /// the option open/close helpers for common flows.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Instruction, Number, OrderBuilder};
    ///
    /// let quantity: Number = "1".parse().unwrap();
    /// let order = OrderBuilder::option_market(
    ///     "AAPL  260116C00150000",
    ///     Instruction::BuyToOpen,
    ///     quantity,
    /// );
    /// ```
    pub fn option_market(
        symbol: impl Into<String>,
        instruction: Instruction,
        quantity: Number,
    ) -> Self {
        Self::single_leg(
            OrderTypeRequest::Market,
            symbol,
            instruction,
            InstrumentAssetType::Option,
            quantity,
            None,
            None,
        )
    }

    /// Build a `LIMIT` order for a single option leg.
    ///
    /// # Arguments
    ///
    /// - `symbol` - Schwab option symbol copied exactly as provided.
    /// - `instruction` - Option instruction to place on the leg.
    /// - `quantity` - Number of option contracts for the leg.
    /// - `price` - Limit price for the option order.
    ///
    /// # Defaults
    ///
    /// Sets [`Session::Normal`], [`Duration::Day`], and
    /// [`OrderStrategyType::Single`].
    ///
    /// # Payload
    ///
    /// Emits `orderType=LIMIT`, the provided `instruction`,
    /// `assetType=OPTION`, and `price`. No `stopPrice` field is included. The
    /// option symbol is not parsed, formatted, trimmed, or normalized.
    ///
    /// # Caution
    ///
    /// This lower-level constructor trusts the provided instruction. Prefer
    /// the option open/close helpers for common flows.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Instruction, Number, OrderBuilder};
    ///
    /// let quantity: Number = "1".parse().unwrap();
    /// let price: Number = "2.50".parse().unwrap();
    /// let order = OrderBuilder::option_limit(
    ///     "AAPL  260116C00150000",
    ///     Instruction::SellToClose,
    ///     quantity,
    ///     price,
    /// );
    /// ```
    pub fn option_limit(
        symbol: impl Into<String>,
        instruction: Instruction,
        quantity: Number,
        price: Number,
    ) -> Self {
        Self::single_leg(
            OrderTypeRequest::Limit,
            symbol,
            instruction,
            InstrumentAssetType::Option,
            quantity,
            Some(price),
            None,
        )
    }

    /// Override the session (default: [`Session::Normal`]).
    ///
    /// # Arguments
    ///
    /// - `session` - Session value to serialize on this order.
    ///
    /// # Payload
    ///
    /// Replaces the current `session` field. Single-leg constructors start
    /// with `NORMAL`; OCO parent orders intentionally omit `session` unless
    /// this setter is called on the composed parent.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder, Session};
    ///
    /// let quantity: Number = "10".parse().unwrap();
    /// let order = OrderBuilder::market_buy("AAPL", quantity)
    ///     .session(Session::Am);
    /// ```
    pub fn session(mut self, session: Session) -> Self {
        self.session = Some(session);
        self
    }

    /// Override the duration (default: [`Duration::Day`]).
    ///
    /// # Arguments
    ///
    /// - `duration` - Duration value to serialize on this order.
    ///
    /// # Payload
    ///
    /// Replaces the current `duration` field. Single-leg constructors start
    /// with `DAY`; OCO parent orders intentionally omit `duration` unless this
    /// setter is called on the composed parent.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Duration, Number, OrderBuilder};
    ///
    /// let quantity: Number = "10".parse().unwrap();
    /// let order = OrderBuilder::market_buy("AAPL", quantity)
    ///     .duration(Duration::GoodTillCancel);
    /// ```
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Override the order strategy type (default: [`OrderStrategyType::Single`]).
    ///
    /// # Arguments
    ///
    /// - `strategy` - Strategy type to serialize on this order.
    ///
    /// # Payload
    ///
    /// Replaces the current `orderStrategyType` field. Prefer
    /// [`Self::one_cancels_other`] or [`Self::first_triggers_second`] for OCO
    /// and trigger strategies because they also set up child orders.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Number, OrderBuilder, OrderStrategyType};
    ///
    /// let quantity: Number = "10".parse().unwrap();
    /// let order = OrderBuilder::market_buy("AAPL", quantity)
    ///     .order_strategy_type(OrderStrategyType::Single);
    /// ```
    pub fn order_strategy_type(mut self, strategy: OrderStrategyType) -> Self {
        self.order_strategy_type = strategy;
        self
    }

    /// Convert an existing Schwab order into a reusable builder payload.
    ///
    /// The conversion keeps only fields that can be submitted back to Schwab as
    /// an order request. Response metadata such as order IDs, status, fill
    /// history, timestamps, and account numbers is intentionally omitted.
    ///
    /// Supports `SINGLE`, `TRIGGER`, and `OCO` order strategies with equity or
    /// option legs. Malformed partial orders and unsupported strategy, type, or
    /// instrument values return [`crate::Error::OrderConversion`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::OrderConversion`] when required request fields are
    /// missing or the historical order uses a strategy, type, or leg this builder
    /// cannot safely recreate.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Order, OrderBuilder};
    ///
    /// fn rebuild(order: &Order) -> schwab::Result<OrderBuilder> {
    ///     OrderBuilder::try_from_order(order)
    /// }
    /// ```
    pub fn try_from_order(order: &Order) -> Result<Self> {
        Self::from_order(order)
    }

    /// Shared constructor for single-leg orders.
    fn single_leg(
        order_type: OrderTypeRequest,
        symbol: impl Into<String>,
        instruction: Instruction,
        asset_type: InstrumentAssetType,
        quantity: Number,
        price: Option<Number>,
        stop_price: Option<Number>,
    ) -> Self {
        Self {
            order_type: Some(order_type),
            session: Some(Session::Normal),
            duration: Some(Duration::Day),
            order_strategy_type: OrderStrategyType::Single,
            complex_order_strategy_type: None,
            price,
            price_link_basis: None,
            price_link_type: None,
            stop_price,
            stop_price_link_basis: None,
            stop_price_link_type: None,
            stop_price_offset: None,
            stop_type: None,
            activation_price: None,
            special_instruction: None,
            order_leg_collection: vec![Leg {
                instruction,
                quantity,
                instrument: LegInstrument {
                    symbol: symbol.into(),
                    asset_type,
                },
            }],
            child_order_strategies: Vec::new(),
        }
    }

    fn empty(order_strategy_type: OrderStrategyType) -> Self {
        Self {
            order_type: None,
            session: None,
            duration: None,
            order_strategy_type,
            complex_order_strategy_type: None,
            price: None,
            price_link_basis: None,
            price_link_type: None,
            stop_price: None,
            stop_price_link_basis: None,
            stop_price_link_type: None,
            stop_price_offset: None,
            stop_type: None,
            activation_price: None,
            special_instruction: None,
            order_leg_collection: Vec::new(),
            child_order_strategies: Vec::new(),
        }
    }

    fn from_order(order: &Order) -> Result<Self> {
        let strategy = required(order.order_strategy_type.clone(), "orderStrategyType")?;
        let mut builder = Self::empty(strategy.clone());
        builder.copy_optional_fields(order)?;

        match strategy {
            OrderStrategyType::Single => {
                if order
                    .child_order_strategies
                    .as_ref()
                    .is_some_and(|children| !children.is_empty())
                {
                    return Err(conversion_error(
                        "SINGLE order cannot include childOrderStrategies".to_string(),
                    ));
                }

                Ok(builder)
            }
            OrderStrategyType::Trigger => {
                let children = required_children(order, 1, "TRIGGER")?;
                builder.child_order_strategies = vec![Self::from_order(&children[0])?];
                Ok(builder)
            }
            OrderStrategyType::Oco => {
                let children = required_children(order, 2, "OCO")?;
                builder.child_order_strategies = vec![
                    Self::from_order(&children[0])?,
                    Self::from_order(&children[1])?,
                ];
                Ok(builder)
            }
            _ => Err(conversion_error(format!(
                "unsupported orderStrategyType {strategy:?}"
            ))),
        }
    }

    fn copy_optional_fields(&mut self, order: &Order) -> Result<()> {
        reject_unsupported_order_fields(order)?;

        self.order_type = order
            .order_type
            .clone()
            .map(request_order_type)
            .transpose()?;
        self.session.clone_from(&order.session);
        self.duration.clone_from(&order.duration);
        self.complex_order_strategy_type
            .clone_from(&order.complex_order_strategy_type);
        self.price = clone_number(&order.price);
        self.price_link_basis.clone_from(&order.price_link_basis);
        self.price_link_type.clone_from(&order.price_link_type);
        self.stop_price = clone_number(&order.stop_price);
        self.stop_price_link_basis
            .clone_from(&order.stop_price_link_basis);
        self.stop_price_link_type
            .clone_from(&order.stop_price_link_type);
        self.stop_price_offset = clone_number(&order.stop_price_offset);
        self.stop_type.clone_from(&order.stop_type);
        self.activation_price = clone_number(&order.activation_price);
        self.special_instruction
            .clone_from(&order.special_instruction);

        if let Some(legs) = &order.order_leg_collection {
            self.order_leg_collection = legs
                .iter()
                .enumerate()
                .map(|(index, leg)| convert_leg(index, leg))
                .collect::<Result<Vec<_>>>()?;
        }
        validate_order_quantity(order, self)?;

        if self.order_strategy_type != OrderStrategyType::Oco {
            require_submit_fields(self)?;
        }

        Ok(())
    }
}

impl TryFrom<&Order> for OrderBuilder {
    type Error = Error;

    fn try_from(order: &Order) -> Result<Self> {
        Self::try_from_order(order)
    }
}

impl TryFrom<Order> for OrderBuilder {
    type Error = Error;

    fn try_from(order: Order) -> Result<Self> {
        Self::try_from_order(&order)
    }
}

fn required<T>(value: Option<T>, field: impl Into<String>) -> Result<T> {
    value.ok_or_else(|| conversion_error(format!("missing {}", field.into())))
}

fn clone_number(value: &Option<Number>) -> Option<Number> {
    *value
}

fn conversion_error(message: String) -> Error {
    Error::OrderConversion(message)
}

fn required_children<'a>(order: &'a Order, count: usize, strategy: &str) -> Result<&'a [Order]> {
    let children = order.child_order_strategies.as_deref().ok_or_else(|| {
        conversion_error(format!("{strategy} order is missing childOrderStrategies"))
    })?;

    if children.len() != count {
        return Err(conversion_error(format!(
            "{strategy} order requires {count} childOrderStrategies, found {}",
            children.len()
        )));
    }

    Ok(children)
}

fn require_submit_fields(builder: &OrderBuilder) -> Result<()> {
    let order_type = builder
        .order_type
        .as_ref()
        .ok_or_else(|| conversion_error("missing orderType".to_string()))?;
    if builder.session.is_none() {
        return Err(conversion_error("missing session".to_string()));
    }
    if builder.duration.is_none() {
        return Err(conversion_error("missing duration".to_string()));
    }
    if builder.order_leg_collection.is_empty() {
        return Err(conversion_error("missing orderLegCollection".to_string()));
    }

    match order_type {
        OrderTypeRequest::Limit | OrderTypeRequest::LimitOnClose => {
            require_number(builder.price, "price")?;
        }
        OrderTypeRequest::Stop => {
            require_number(builder.stop_price, "stopPrice")?;
        }
        OrderTypeRequest::StopLimit => {
            require_number(builder.price, "price")?;
            require_number(builder.stop_price, "stopPrice")?;
        }
        _ => {}
    }

    Ok(())
}

fn require_number(value: Option<Number>, field: &'static str) -> Result<()> {
    value
        .map(|_| ())
        .ok_or_else(|| conversion_error(format!("missing {field}")))
}

fn reject_unsupported_order_fields(order: &Order) -> Result<()> {
    reject_present(order.tax_lot_method.is_some(), "taxLotMethod")?;

    Ok(())
}

fn validate_order_quantity(order: &Order, builder: &OrderBuilder) -> Result<()> {
    let Some(quantity) = order.quantity else {
        return Ok(());
    };

    if builder.order_leg_collection.len() != 1 {
        return Err(conversion_error(
            "unsupported request field quantity for orders without exactly one leg".to_string(),
        ));
    }

    let leg_quantity = builder.order_leg_collection[0].quantity;
    if leg_quantity != quantity {
        return Err(conversion_error(format!(
            "quantity {quantity:?} does not match orderLegCollection[0].quantity {leg_quantity:?}"
        )));
    }

    Ok(())
}

fn reject_present(present: bool, field: &str) -> Result<()> {
    if present {
        Err(conversion_error(format!(
            "unsupported request field {field}"
        )))
    } else {
        Ok(())
    }
}

fn request_order_type(order_type: OrderType) -> Result<OrderTypeRequest> {
    match order_type {
        OrderType::Market => Ok(OrderTypeRequest::Market),
        OrderType::Limit => Ok(OrderTypeRequest::Limit),
        OrderType::Stop => Ok(OrderTypeRequest::Stop),
        OrderType::StopLimit => Ok(OrderTypeRequest::StopLimit),
        OrderType::TrailingStop => Ok(OrderTypeRequest::TrailingStop),
        OrderType::Cabinet => Ok(OrderTypeRequest::Cabinet),
        OrderType::NonMarketable => Ok(OrderTypeRequest::NonMarketable),
        OrderType::MarketOnClose => Ok(OrderTypeRequest::MarketOnClose),
        OrderType::Exercise => Ok(OrderTypeRequest::Exercise),
        OrderType::TrailingStopLimit => Ok(OrderTypeRequest::TrailingStopLimit),
        OrderType::NetDebit => Ok(OrderTypeRequest::NetDebit),
        OrderType::NetCredit => Ok(OrderTypeRequest::NetCredit),
        OrderType::NetZero => Ok(OrderTypeRequest::NetZero),
        OrderType::LimitOnClose => Ok(OrderTypeRequest::LimitOnClose),
        OrderType::Unknown => Err(conversion_error(
            "unsupported orderType UNKNOWN".to_string(),
        )),
    }
}

fn convert_leg(index: usize, leg: &OrderLegCollection) -> Result<Leg> {
    reject_unsupported_leg_fields(index, leg)?;

    let instruction = required(
        leg.instruction.clone(),
        format!("orderLegCollection[{index}].instruction"),
    )?;
    let quantity = required(
        clone_number(&leg.quantity),
        format!("orderLegCollection[{index}].quantity"),
    )?;
    let instrument = required(
        leg.instrument.as_ref(),
        format!("orderLegCollection[{index}].instrument"),
    )?;
    let (symbol, instrument_asset_type) = instrument_symbol_and_asset(index, instrument)?;
    let asset_type = match leg.order_leg_type.clone().or(instrument_asset_type.clone()) {
        Some(InstrumentAssetType::Equity) => InstrumentAssetType::Equity,
        Some(InstrumentAssetType::Option) => InstrumentAssetType::Option,
        Some(asset_type) => {
            return Err(conversion_error(format!(
                "unsupported orderLegCollection[{index}].orderLegType {asset_type:?}"
            )));
        }
        None => {
            return Err(conversion_error(format!(
                "missing orderLegCollection[{index}].orderLegType or instrument.assetType"
            )));
        }
    };

    if let Some(instrument_asset_type) = instrument_asset_type
        && instrument_asset_type != asset_type
    {
        return Err(conversion_error(format!(
            "orderLegCollection[{index}].instrument assetType is {instrument_asset_type:?}, expected {asset_type:?}"
        )));
    }

    Ok(Leg {
        instruction,
        quantity,
        instrument: LegInstrument { symbol, asset_type },
    })
}

fn reject_unsupported_leg_fields(index: usize, leg: &OrderLegCollection) -> Result<()> {
    let fields = [
        (leg.quantity_type.is_some(), "quantityType"),
        (leg.position_effect.is_some(), "positionEffect"),
        (leg.div_cap_gains.is_some(), "divCapGains"),
        (leg.to_symbol.is_some(), "toSymbol"),
    ];

    for (present, field) in fields {
        if present {
            return Err(conversion_error(format!(
                "unsupported orderLegCollection[{index}].{field}"
            )));
        }
    }

    Ok(())
}

fn instrument_symbol_and_asset(
    index: usize,
    instrument: &AccountsInstrument,
) -> Result<(String, Option<InstrumentAssetType>)> {
    match instrument {
        AccountsInstrument::Equity(instrument) => {
            symbol_and_asset(index, &instrument.symbol, &instrument.asset_type)
        }
        AccountsInstrument::Option(instrument) => {
            symbol_and_asset(index, &instrument.symbol, &instrument.asset_type)
        }
        AccountsInstrument::FixedIncome(instrument) => {
            symbol_and_asset(index, &instrument.symbol, &instrument.asset_type)
        }
        AccountsInstrument::CashEquivalent(instrument) => {
            symbol_and_asset(index, &instrument.symbol, &instrument.asset_type)
        }
        AccountsInstrument::MutualFund(instrument) => {
            symbol_and_asset(index, &instrument.symbol, &instrument.asset_type)
        }
    }
}

fn symbol_and_asset(
    index: usize,
    symbol: &Option<String>,
    asset_type: &Option<InstrumentAssetType>,
) -> Result<(String, Option<InstrumentAssetType>)> {
    let symbol = required(
        symbol.clone(),
        format!("orderLegCollection[{index}].instrument.symbol"),
    )?;

    if symbol.trim().is_empty() {
        return Err(conversion_error(format!(
            "orderLegCollection[{index}].instrument symbol is empty"
        )));
    }

    Ok((symbol, asset_type.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
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
    /// Convenience constructors choose buy/sell instructions.
    #[test]
    fn buy_sell_convenience_constructors() {
        let cases = [
            (OrderBuilder::market_buy("AAPL", n(1.0)), "MARKET", "BUY"),
            (OrderBuilder::market_sell("AAPL", n(1.0)), "MARKET", "SELL"),
            (
                OrderBuilder::limit_buy("AAPL", n(1.0), n(100.0)),
                "LIMIT",
                "BUY",
            ),
            (
                OrderBuilder::limit_sell("AAPL", n(1.0), n(100.0)),
                "LIMIT",
                "SELL",
            ),
            (
                OrderBuilder::stop_buy("AAPL", n(1.0), n(90.0)),
                "STOP",
                "BUY",
            ),
            (
                OrderBuilder::stop_sell("AAPL", n(1.0), n(90.0)),
                "STOP",
                "SELL",
            ),
            (
                OrderBuilder::stop_limit_buy("AAPL", n(1.0), n(91.0), n(90.0)),
                "STOP_LIMIT",
                "BUY",
            ),
            (
                OrderBuilder::stop_limit_sell("AAPL", n(1.0), n(91.0), n(90.0)),
                "STOP_LIMIT",
                "SELL",
            ),
        ];

        for (order, expected_type, expected_instruction) in cases {
            let json: serde_json::Value = serde_json::to_value(&order).unwrap();
            assert_eq!(json["orderType"], expected_type);
            assert_eq!(
                json["orderLegCollection"][0]["instruction"],
                expected_instruction
            );
        }
    }

    /// Option convenience constructors choose option instructions and asset type.
    #[test]
    fn option_convenience_constructors() {
        let symbol = "AAPL  260116C00150000";
        let cases = [
            (
                OrderBuilder::option_buy_to_open_market(symbol, n(1.0)),
                "MARKET",
                "BUY_TO_OPEN",
                None,
            ),
            (
                OrderBuilder::option_buy_to_open_limit(symbol, n(1.0), n(2.5)),
                "LIMIT",
                "BUY_TO_OPEN",
                Some(expected_number(2.5)),
            ),
            (
                OrderBuilder::option_sell_to_open_market(symbol, n(1.0)),
                "MARKET",
                "SELL_TO_OPEN",
                None,
            ),
            (
                OrderBuilder::option_sell_to_open_limit(symbol, n(1.0), n(2.5)),
                "LIMIT",
                "SELL_TO_OPEN",
                Some(expected_number(2.5)),
            ),
            (
                OrderBuilder::option_buy_to_close_market(symbol, n(1.0)),
                "MARKET",
                "BUY_TO_CLOSE",
                None,
            ),
            (
                OrderBuilder::option_buy_to_close_limit(symbol, n(1.0), n(2.5)),
                "LIMIT",
                "BUY_TO_CLOSE",
                Some(expected_number(2.5)),
            ),
            (
                OrderBuilder::option_sell_to_close_market(symbol, n(1.0)),
                "MARKET",
                "SELL_TO_CLOSE",
                None,
            ),
            (
                OrderBuilder::option_sell_to_close_limit(symbol, n(1.0), n(2.5)),
                "LIMIT",
                "SELL_TO_CLOSE",
                Some(expected_number(2.5)),
            ),
        ];

        for (order, expected_type, expected_instruction, expected_price) in cases {
            let json: serde_json::Value = serde_json::to_value(&order).unwrap();
            assert_eq!(json["orderType"], expected_type);
            assert_eq!(json["session"], "NORMAL");
            assert_eq!(json["duration"], "DAY");
            assert_eq!(json["orderStrategyType"], "SINGLE");
            assert_eq!(
                json["orderLegCollection"][0]["instruction"],
                expected_instruction
            );
            assert_eq!(
                json["orderLegCollection"][0]["quantity"],
                expected_number(1.0)
            );
            assert_eq!(
                json["orderLegCollection"][0]["instrument"]["symbol"],
                symbol
            );
            assert_eq!(
                json["orderLegCollection"][0]["instrument"]["assetType"],
                "OPTION"
            );

            if let Some(price) = expected_price {
                assert_eq!(json["price"], price);
            } else {
                assert!(json.get("price").is_none());
            }
        }
    }

    /// Lower-level option constructors accept explicit option instructions.
    #[test]
    fn option_lower_level_constructors() {
        let order = OrderBuilder::option_limit(
            "MSFT  260116P00300000",
            Instruction::SellToClose,
            n(2.0),
            n(3.25),
        );
        let json: serde_json::Value = serde_json::to_value(&order).unwrap();

        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(json["price"], expected_number(3.25));
        assert_eq!(
            json["orderLegCollection"][0]["instruction"],
            "SELL_TO_CLOSE"
        );
        assert_eq!(
            json["orderLegCollection"][0]["quantity"],
            expected_number(2.0)
        );
        assert_eq!(
            json["orderLegCollection"][0]["instrument"]["assetType"],
            "OPTION"
        );
    }

    /// OCO composition nests two child orders without inventing parent order fields.
    #[test]
    fn one_cancels_other_json() {
        let order = OrderBuilder::one_cancels_other(
            OrderBuilder::limit_sell("AAPL", n(1.0), n(140.0)),
            OrderBuilder::stop_sell("AAPL", n(1.0), n(120.0)),
        );
        let json: serde_json::Value = serde_json::to_value(&order).unwrap();

        assert_eq!(json["orderStrategyType"], "OCO");
        assert!(json.get("orderType").is_none());
        assert!(json.get("session").is_none());
        assert!(json.get("duration").is_none());
        assert!(json.get("orderLegCollection").is_none());

        let children = json["childOrderStrategies"].as_array().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0]["orderType"], "LIMIT");
        assert_eq!(children[0]["orderLegCollection"][0]["instruction"], "SELL");
        assert_eq!(children[1]["orderType"], "STOP");
        assert_eq!(children[1]["orderLegCollection"][0]["instruction"], "SELL");
    }

    /// Trigger composition keeps the first order as the parent and nests the second order.
    #[test]
    fn first_triggers_second_json() {
        let order = OrderBuilder::first_triggers_second(
            OrderBuilder::market_buy("AAPL", n(1.0)),
            OrderBuilder::limit_sell("AAPL", n(1.0), n(140.0)),
        );
        let json: serde_json::Value = serde_json::to_value(&order).unwrap();

        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["orderStrategyType"], "TRIGGER");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY");

        let children = json["childOrderStrategies"].as_array().unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0]["orderType"], "LIMIT");
        assert_eq!(children[0]["orderStrategyType"], "SINGLE");
        assert_eq!(children[0]["orderLegCollection"][0]["instruction"], "SELL");
    }

    /// Bracket composition triggers an OCO exit after the entry order fills.
    #[test]
    fn bracket_order_json() {
        let order = OrderBuilder::first_triggers_second(
            OrderBuilder::market_buy("AAPL", n(1.0)),
            OrderBuilder::one_cancels_other(
                OrderBuilder::limit_sell("AAPL", n(1.0), n(160.0)),
                OrderBuilder::stop_sell("AAPL", n(1.0), n(140.0)),
            ),
        );
        let json: serde_json::Value = serde_json::to_value(&order).unwrap();

        assert_eq!(json["orderType"], "MARKET");
        assert_eq!(json["orderStrategyType"], "TRIGGER");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY");

        let trigger_children = json["childOrderStrategies"].as_array().unwrap();
        assert_eq!(trigger_children.len(), 1);
        assert_eq!(trigger_children[0]["orderStrategyType"], "OCO");
        assert!(trigger_children[0].get("orderType").is_none());
        assert!(trigger_children[0].get("orderLegCollection").is_none());

        let oco_children = trigger_children[0]["childOrderStrategies"]
            .as_array()
            .unwrap();
        assert_eq!(oco_children.len(), 2);
        assert_eq!(oco_children[0]["orderType"], "LIMIT");
        assert_eq!(oco_children[0]["price"], expected_number(160.0));
        assert_eq!(
            oco_children[0]["orderLegCollection"][0]["instruction"],
            "SELL"
        );
        assert_eq!(oco_children[1]["orderType"], "STOP");
        assert_eq!(oco_children[1]["stopPrice"], expected_number(140.0));
        assert_eq!(
            oco_children[1]["orderLegCollection"][0]["instruction"],
            "SELL"
        );
    }
    /// Historical single-leg limit orders convert into submit-ready payloads.
    #[test]
    fn converts_single_equity_order() {
        let order: Order = serde_json::from_value(serde_json::json!({
            "orderId": 123456,
            "status": "FILLED",
            "enteredTime": "2026-01-02T03:04:05+0000",
            "session": "NORMAL",
            "duration": "GOOD_TILL_CANCEL",
            "orderType": "LIMIT",
            "price": expected_number(400.50),
            "orderStrategyType": "SINGLE",
            "orderLegCollection": [{
                "orderLegType": "EQUITY",
                "instruction": "BUY",
                "quantity": expected_number(5.0),
                "instrument": {
                    "assetType": "EQUITY",
                    "symbol": "MSFT",
                    "description": "Microsoft Corp"
                }
            }]
        }))
        .unwrap();

        let builder = OrderBuilder::try_from_order(&order).unwrap();
        let json = serde_json::to_value(&builder).unwrap();

        assert_eq!(json["session"], "NORMAL");
        assert_eq!(json["duration"], "GOOD_TILL_CANCEL");
        assert_eq!(json["orderType"], "LIMIT");
        assert_eq!(json["price"], expected_number(400.50));
        assert_eq!(json["orderStrategyType"], "SINGLE");
        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY");
        assert_eq!(
            json["orderLegCollection"][0]["quantity"],
            expected_number(5.0)
        );
        assert_eq!(
            json["orderLegCollection"][0]["instrument"]["symbol"],
            "MSFT"
        );
        assert_eq!(
            json["orderLegCollection"][0]["instrument"]["assetType"],
            "EQUITY"
        );
        assert!(json.get("orderId").is_none());
        assert!(json.get("status").is_none());
        assert!(json.get("enteredTime").is_none());
    }

    /// Historical option legs keep option asset type and symbol.
    #[test]
    fn converts_option_leg() {
        let order: Order = serde_json::from_value(serde_json::json!({
            "session": "NORMAL",
            "duration": "DAY",
            "orderType": "LIMIT",
            "price": expected_number(1.25),
            "orderStrategyType": "SINGLE",
            "orderLegCollection": [{
                "orderLegType": "OPTION",
                "instruction": "BUY_TO_OPEN",
                "quantity": expected_number(1.0),
                "instrument": {
                    "assetType": "OPTION",
                    "symbol": "MSFT  260116C00400000",
                    "putCall": "CALL",
                    "type": "VANILLA"
                }
            }]
        }))
        .unwrap();

        let builder = OrderBuilder::try_from_order(&order).unwrap();
        let json = serde_json::to_value(&builder).unwrap();

        assert_eq!(json["orderLegCollection"][0]["instruction"], "BUY_TO_OPEN");
        assert_eq!(
            json["orderLegCollection"][0]["instrument"]["assetType"],
            "OPTION"
        );
        assert_eq!(
            json["orderLegCollection"][0]["instrument"]["symbol"],
            "MSFT  260116C00400000"
        );
    }

    /// Trigger orders recursively preserve the first child strategy.
    #[test]
    fn converts_trigger_order_with_child() {
        let order: Order = serde_json::from_value(serde_json::json!({
            "session": "NORMAL",
            "duration": "DAY",
            "orderType": "LIMIT",
            "price": expected_number(400.0),
            "orderStrategyType": "TRIGGER",
            "orderLegCollection": [{
                "orderLegType": "EQUITY",
                "instruction": "BUY",
                "quantity": expected_number(1.0),
                "instrument": { "assetType": "EQUITY", "symbol": "MSFT" }
            }],
            "childOrderStrategies": [{
                "session": "NORMAL",
                "duration": "DAY",
                "orderType": "STOP",
                "stopPrice": expected_number(390.0),
                "orderStrategyType": "SINGLE",
                "orderLegCollection": [{
                    "orderLegType": "EQUITY",
                    "instruction": "SELL",
                    "quantity": expected_number(1.0),
                    "instrument": { "assetType": "EQUITY", "symbol": "MSFT" }
                }]
            }]
        }))
        .unwrap();

        let builder = OrderBuilder::try_from_order(&order).unwrap();
        let json = serde_json::to_value(&builder).unwrap();

        assert_eq!(json["orderStrategyType"], "TRIGGER");
        assert_eq!(json["childOrderStrategies"].as_array().unwrap().len(), 1);
        assert_eq!(json["childOrderStrategies"][0]["orderType"], "STOP");
        assert_eq!(
            json["childOrderStrategies"][0]["stopPrice"],
            expected_number(390.0)
        );
    }

    /// OCO wrapper orders can convert without top-level order fields or legs.
    #[test]
    fn converts_oco_order_with_two_children() {
        let child = |order_type: &str, instruction: &str, stop_price: Option<serde_json::Value>| {
            let mut value = serde_json::json!({
                "session": "NORMAL",
                "duration": "DAY",
                "orderType": order_type,
                "price": expected_number(400.0),
                "orderStrategyType": "SINGLE",
                "orderLegCollection": [{
                    "orderLegType": "EQUITY",
                    "instruction": instruction,
                    "quantity": expected_number(1.0),
                    "instrument": { "assetType": "EQUITY", "symbol": "MSFT" }
                }]
            });

            if let Some(stop_price) = stop_price {
                value["stopPrice"] = stop_price;
            }

            value
        };
        let order: Order = serde_json::from_value(serde_json::json!({
            "orderStrategyType": "OCO",
            "childOrderStrategies": [
                child("LIMIT", "SELL", None),
                child("STOP_LIMIT", "SELL", Some(expected_number(390.0)))
            ]
        }))
        .unwrap();

        let builder = OrderBuilder::try_from_order(&order).unwrap();
        let json = serde_json::to_value(&builder).unwrap();

        assert_eq!(json["orderStrategyType"], "OCO");
        assert!(json.get("orderType").is_none());
        assert!(json.get("orderLegCollection").is_none());
        assert_eq!(json["childOrderStrategies"].as_array().unwrap().len(), 2);
        assert_eq!(json["childOrderStrategies"][1]["orderType"], "STOP_LIMIT");
    }

    /// Missing order-type-specific prices fail instead of emitting invalid payloads.
    #[test]
    fn rejects_missing_type_specific_prices() {
        let cases = [
            ("LIMIT", None, None, "price"),
            ("STOP", None, None, "stopPrice"),
            ("STOP_LIMIT", Some(expected_number(10.0)), None, "stopPrice"),
            ("STOP_LIMIT", None, Some(expected_number(9.0)), "price"),
        ];

        for (order_type, price, stop_price, missing_field) in cases {
            let mut value = serde_json::json!({
                "session": "NORMAL",
                "duration": "DAY",
                "orderType": order_type,
                "orderStrategyType": "SINGLE",
                "orderLegCollection": [{
                    "orderLegType": "EQUITY",
                    "instruction": "BUY",
                    "quantity": expected_number(1.0),
                    "instrument": { "assetType": "EQUITY", "symbol": "MSFT" }
                }]
            });

            if let Some(price) = price {
                value["price"] = price;
            }
            if let Some(stop_price) = stop_price {
                value["stopPrice"] = stop_price;
            }

            let order: Order = serde_json::from_value(value).unwrap();
            let error = OrderBuilder::try_from_order(&order).unwrap_err();

            assert!(
                matches!(error, Error::OrderConversion(message) if message.contains(missing_field))
            );
        }
    }

    /// SINGLE orders with children fail because child strategies would be dropped.
    #[test]
    fn rejects_single_order_with_children() {
        let order: Order = serde_json::from_value(serde_json::json!({
            "session": "NORMAL",
            "duration": "DAY",
            "orderType": "MARKET",
            "orderStrategyType": "SINGLE",
            "orderLegCollection": [{
                "orderLegType": "EQUITY",
                "instruction": "BUY",
                "quantity": expected_number(1.0),
                "instrument": { "assetType": "EQUITY", "symbol": "MSFT" }
            }],
            "childOrderStrategies": [{ "orderStrategyType": "OCO" }]
        }))
        .unwrap();

        let error = OrderBuilder::try_from_order(&order).unwrap_err();

        assert!(matches!(error, Error::OrderConversion(message) if message.contains("SINGLE")));
    }

    /// Common response-level quantity is allowed when it matches the single leg.
    #[test]
    fn accepts_matching_top_level_quantity() {
        let order: Order = serde_json::from_value(serde_json::json!({
            "session": "NORMAL",
            "duration": "DAY",
            "orderType": "MARKET",
            "quantity": expected_number(1.0),
            "orderStrategyType": "SINGLE",
            "orderLegCollection": [{
                "orderLegType": "EQUITY",
                "instruction": "BUY",
                "quantity": expected_number(1.0),
                "instrument": { "assetType": "EQUITY", "symbol": "MSFT" }
            }]
        }))
        .unwrap();

        let builder = OrderBuilder::try_from_order(&order).unwrap();
        let json = serde_json::to_value(&builder).unwrap();

        assert!(json.get("quantity").is_none());
        assert_eq!(
            json["orderLegCollection"][0]["quantity"],
            expected_number(1.0)
        );
    }

    /// Unsupported or inconsistent top-level request fields fail rather than being dropped.
    #[test]
    fn rejects_unsupported_top_level_request_fields() {
        let mut value = serde_json::json!({
            "session": "NORMAL",
            "duration": "DAY",
            "orderType": "MARKET",
            "quantity": expected_number(2.0),
            "orderStrategyType": "SINGLE",
            "orderLegCollection": [{
                "orderLegType": "EQUITY",
                "instruction": "BUY",
                "quantity": expected_number(1.0),
                "instrument": { "assetType": "EQUITY", "symbol": "MSFT" }
            }]
        });
        let order: Order = serde_json::from_value(value.clone()).unwrap();
        let error = OrderBuilder::try_from_order(&order).unwrap_err();
        assert!(
            matches!(error, Error::OrderConversion(message) if message.contains("does not match"))
        );

        value["quantity"] = expected_number(1.0);
        value["taxLotMethod"] = serde_json::json!("FIFO");
        let order: Order = serde_json::from_value(value).unwrap();
        let error = OrderBuilder::try_from_order(&order).unwrap_err();
        assert!(
            matches!(error, Error::OrderConversion(message) if message.contains("taxLotMethod"))
        );
    }

    /// Top-level quantity with multiple legs fails because this builder cannot model spreads yet.
    #[test]
    fn rejects_top_level_quantity_for_multi_leg_orders() {
        let order: Order = serde_json::from_value(serde_json::json!({
            "session": "NORMAL",
            "duration": "DAY",
            "orderType": "MARKET",
            "quantity": expected_number(2.0),
            "orderStrategyType": "SINGLE",
            "orderLegCollection": [
                {
                    "orderLegType": "EQUITY",
                    "instruction": "BUY",
                    "quantity": expected_number(1.0),
                    "instrument": { "assetType": "EQUITY", "symbol": "MSFT" }
                },
                {
                    "orderLegType": "EQUITY",
                    "instruction": "BUY",
                    "quantity": expected_number(1.0),
                    "instrument": { "assetType": "EQUITY", "symbol": "AAPL" }
                }
            ]
        }))
        .unwrap();

        let error = OrderBuilder::try_from_order(&order).unwrap_err();

        assert!(
            matches!(error, Error::OrderConversion(message) if message.contains("without exactly one leg"))
        );
    }

    /// Unsupported leg request fields fail rather than being dropped.
    #[test]
    fn rejects_unsupported_leg_request_fields() {
        for (field, expected) in [
            ("quantityType", "quantityType"),
            ("positionEffect", "positionEffect"),
            ("divCapGains", "divCapGains"),
            ("toSymbol", "toSymbol"),
        ] {
            let mut value = serde_json::json!({
                "session": "NORMAL",
                "duration": "DAY",
                "orderType": "MARKET",
                "orderStrategyType": "SINGLE",
                "orderLegCollection": [{
                    "orderLegType": "EQUITY",
                    "instruction": "BUY",
                    "quantity": expected_number(1.0),
                    "instrument": { "assetType": "EQUITY", "symbol": "MSFT" }
                }]
            });
            value["orderLegCollection"][0][field] = match field {
                "quantityType" => serde_json::json!("ALL_SHARES"),
                "positionEffect" => serde_json::json!("OPENING"),
                "divCapGains" => serde_json::json!("REINVEST"),
                _ => serde_json::json!("MSFT2"),
            };

            let order: Order = serde_json::from_value(value).unwrap();
            let error = OrderBuilder::try_from_order(&order).unwrap_err();

            assert!(matches!(error, Error::OrderConversion(message) if message.contains(expected)));
        }
    }

    /// Missing leg-field errors include the leg index for actionable diagnostics.
    #[test]
    fn missing_leg_field_errors_include_index() {
        let order: Order = serde_json::from_value(serde_json::json!({
            "session": "NORMAL",
            "duration": "DAY",
            "orderType": "MARKET",
            "orderStrategyType": "SINGLE",
            "orderLegCollection": [
                {
                    "orderLegType": "EQUITY",
                    "instruction": "BUY",
                    "quantity": expected_number(1.0),
                    "instrument": { "assetType": "EQUITY", "symbol": "MSFT" }
                },
                {
                    "orderLegType": "EQUITY",
                    "quantity": expected_number(1.0),
                    "instrument": { "assetType": "EQUITY", "symbol": "AAPL" }
                }
            ]
        }))
        .unwrap();

        let error = OrderBuilder::try_from_order(&order).unwrap_err();

        assert!(
            matches!(error, Error::OrderConversion(message) if message.contains("orderLegCollection[1].instruction"))
        );
    }

    /// Unknown response-only order types fail instead of guessing a request type.
    #[test]
    fn rejects_unknown_order_type() {
        let order: Order = serde_json::from_value(serde_json::json!({
            "session": "NORMAL",
            "duration": "DAY",
            "orderType": "UNKNOWN",
            "orderStrategyType": "SINGLE",
            "orderLegCollection": [{
                "orderLegType": "EQUITY",
                "instruction": "BUY",
                "quantity": expected_number(1.0),
                "instrument": { "assetType": "EQUITY", "symbol": "MSFT" }
            }]
        }))
        .unwrap();

        let error = OrderBuilder::try_from_order(&order).unwrap_err();

        assert!(matches!(error, Error::OrderConversion(message) if message.contains("UNKNOWN")));
    }

    /// Malformed OCO trees fail with a conversion error.
    #[test]
    fn rejects_oco_without_two_children() {
        let order: Order = serde_json::from_value(serde_json::json!({
            "orderStrategyType": "OCO",
            "childOrderStrategies": []
        }))
        .unwrap();

        let error = OrderBuilder::try_from_order(&order).unwrap_err();

        assert!(matches!(error, Error::OrderConversion(message) if message.contains("requires 2")));
    }
}
