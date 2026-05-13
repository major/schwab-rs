use serde::Deserialize;

use super::Number;
use super::enums::*;

// ---------------------------------------------------------------------------
// OrderResponse (from Location header, not JSON)
// ---------------------------------------------------------------------------

/// Response metadata returned by order placement and replacement endpoints.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderResponse {
    /// Raw `Location` header returned by Schwab.
    pub location: Option<String>,
    /// Parsed trailing order ID from `location`, when Schwab includes one.
    pub order_id: Option<i64>,
}

impl OrderResponse {
    pub(crate) fn from_location_header(headers: &reqwest::header::HeaderMap) -> Self {
        let location = headers
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
        let order_id = location
            .as_deref()
            .map(|value| value.trim_end_matches('/'))
            .and_then(|trimmed| trimmed.rsplit('/').next())
            .and_then(|value| value.parse::<i64>().ok());
        Self { location, order_id }
    }
}

// ---------------------------------------------------------------------------
// Service Error
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceError {
    pub errors: Option<Vec<String>>,
    pub message: Option<String>,
}

// ---------------------------------------------------------------------------
// Account types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub securities_account: Option<SecuritiesAccount>,
}

/// Discriminated by the `type` field: `"MARGIN"` or `"CASH"`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SecuritiesAccount {
    #[serde(rename = "MARGIN")]
    Margin(MarginAccount),
    #[serde(rename = "CASH")]
    Cash(CashAccount),
}

/// allOf SecuritiesAccountBase + margin-specific balances.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MarginAccount {
    // Inlined from SecuritiesAccountBase
    pub account_number: Option<String>,
    pub is_closing_only_restricted: Option<bool>,
    pub is_day_trader: Option<bool>,
    pub pfcb_flag: Option<bool>,
    pub positions: Option<Vec<Position>>,
    pub round_trips: Option<i32>,
    #[serde(rename = "type")]
    pub r#type: Option<SecuritiesAccountType>,
    // MarginAccount-specific
    pub current_balances: Option<MarginBalance>,
    pub initial_balances: Option<MarginInitialBalance>,
    pub projected_balances: Option<MarginBalance>,
}

/// allOf SecuritiesAccountBase + cash-specific balances.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CashAccount {
    // Inlined from SecuritiesAccountBase
    pub account_number: Option<String>,
    pub is_closing_only_restricted: Option<bool>,
    pub is_day_trader: Option<bool>,
    pub pfcb_flag: Option<bool>,
    pub positions: Option<Vec<Position>>,
    pub round_trips: Option<i32>,
    #[serde(rename = "type")]
    pub r#type: Option<SecuritiesAccountType>,
    // CashAccount-specific
    pub current_balances: Option<CashBalance>,
    pub initial_balances: Option<CashInitialBalance>,
    pub projected_balances: Option<CashBalance>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MarginBalance {
    pub available_funds: Option<Number>,
    pub available_funds_non_marginable_trade: Option<Number>,
    pub buying_power: Option<Number>,
    pub buying_power_non_marginable_trade: Option<Number>,
    pub day_trading_buying_power: Option<Number>,
    pub day_trading_buying_power_call: Option<Number>,
    pub equity: Option<Number>,
    pub equity_percentage: Option<Number>,
    pub is_in_call: Option<Number>,
    pub long_margin_value: Option<Number>,
    pub maintenance_call: Option<Number>,
    pub maintenance_requirement: Option<Number>,
    pub margin_balance: Option<Number>,
    pub option_buying_power: Option<Number>,
    pub reg_t_call: Option<Number>,
    pub short_balance: Option<Number>,
    pub short_margin_value: Option<Number>,
    pub sma: Option<Number>,
    pub stock_buying_power: Option<Number>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CashBalance {
    pub cash_available_for_trading: Option<Number>,
    pub cash_available_for_withdrawal: Option<Number>,
    pub cash_call: Option<Number>,
    pub cash_debit_call_value: Option<Number>,
    pub long_non_marginable_market_value: Option<Number>,
    pub total_cash: Option<Number>,
    pub unsettled_cash: Option<Number>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MarginInitialBalance {
    pub account_value: Option<Number>,
    pub accrued_interest: Option<Number>,
    pub available_funds_non_marginable_trade: Option<Number>,
    pub bond_value: Option<Number>,
    pub buying_power: Option<Number>,
    pub cash_available_for_trading: Option<Number>,
    pub cash_balance: Option<Number>,
    pub cash_receipts: Option<Number>,
    pub day_trading_buying_power: Option<Number>,
    pub day_trading_buying_power_call: Option<Number>,
    pub day_trading_equity_call: Option<Number>,
    pub equity: Option<Number>,
    pub equity_percentage: Option<Number>,
    pub is_in_call: Option<Number>,
    pub liquidation_value: Option<Number>,
    pub long_margin_value: Option<Number>,
    pub long_option_market_value: Option<Number>,
    pub long_stock_value: Option<Number>,
    pub maintenance_call: Option<Number>,
    pub maintenance_requirement: Option<Number>,
    pub margin: Option<Number>,
    pub margin_balance: Option<Number>,
    pub margin_equity: Option<Number>,
    pub money_market_fund: Option<Number>,
    pub mutual_fund_value: Option<Number>,
    pub pending_deposits: Option<Number>,
    pub reg_t_call: Option<Number>,
    pub short_balance: Option<Number>,
    pub short_margin_value: Option<Number>,
    pub short_option_market_value: Option<Number>,
    pub short_stock_value: Option<Number>,
    pub total_cash: Option<Number>,
    pub unsettled_cash: Option<Number>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CashInitialBalance {
    pub account_value: Option<Number>,
    pub accrued_interest: Option<Number>,
    pub bond_value: Option<Number>,
    pub cash_available_for_trading: Option<Number>,
    pub cash_available_for_withdrawal: Option<Number>,
    pub cash_balance: Option<Number>,
    pub cash_debit_call_value: Option<Number>,
    pub cash_receipts: Option<Number>,
    pub is_in_call: Option<Number>,
    pub liquidation_value: Option<Number>,
    pub long_option_market_value: Option<Number>,
    pub long_stock_value: Option<Number>,
    pub money_market_fund: Option<Number>,
    pub mutual_fund_value: Option<Number>,
    pub pending_deposits: Option<Number>,
    pub short_option_market_value: Option<Number>,
    pub short_stock_value: Option<Number>,
    pub unsettled_cash: Option<Number>,
}

// ---------------------------------------------------------------------------
// Account instruments
// ---------------------------------------------------------------------------

/// Discriminated union of account instrument types.
/// Variants ordered most-specific first for `untagged` deserialization.
/// Check `asset_type` on the deserialized struct to identify the kind.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AccountsInstrument {
    Option(AccountOption),
    FixedIncome(AccountFixedIncome),
    CashEquivalent(AccountCashEquivalent),
    Equity(AccountEquity),
    MutualFund(AccountMutualFund),
}

/// allOf AccountsBaseInstrument (no extra fields).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountEquity {
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
}

/// allOf AccountsBaseInstrument + fixed-income fields.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountFixedIncome {
    // Inlined from AccountsBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // AccountFixedIncome-specific
    pub factor: Option<Number>,
    pub maturity_date: Option<String>,
    pub variable_rate: Option<Number>,
}

/// allOf AccountsBaseInstrument (no extra fields).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountMutualFund {
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
}

/// allOf AccountsBaseInstrument + option fields.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountOption {
    // Inlined from AccountsBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // AccountOption-specific
    pub option_deliverables: Option<Vec<AccountApiOptionDeliverable>>,
    pub option_multiplier: Option<i32>,
    pub put_call: Option<OptionPutCall>,
    #[serde(rename = "type")]
    pub r#type: Option<OptionType>,
    pub underlying_symbol: Option<String>,
}

/// allOf AccountsBaseInstrument + cash-equivalent type.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountCashEquivalent {
    // Inlined from AccountsBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // AccountCashEquivalent-specific
    #[serde(rename = "type")]
    pub r#type: Option<CashEquivalentType>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountApiOptionDeliverable {
    pub api_currency_type: Option<ApiCurrencyType>,
    pub asset_type: Option<AssetType>,
    pub deliverable_units: Option<Number>,
    pub symbol: Option<String>,
}

// ---------------------------------------------------------------------------
// Position & AccountNumberHash
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub aged_quantity: Option<Number>,
    pub average_long_price: Option<Number>,
    pub average_price: Option<Number>,
    pub average_short_price: Option<Number>,
    pub current_day_cost: Option<Number>,
    pub current_day_profit_loss: Option<Number>,
    pub current_day_profit_loss_percentage: Option<Number>,
    pub instrument: Option<AccountsInstrument>,
    pub long_open_profit_loss: Option<Number>,
    pub long_quantity: Option<Number>,
    pub maintenance_requirement: Option<Number>,
    pub market_value: Option<Number>,
    pub previous_session_long_quantity: Option<Number>,
    pub previous_session_short_quantity: Option<Number>,
    pub settled_long_quantity: Option<Number>,
    pub settled_short_quantity: Option<Number>,
    pub short_open_profit_loss: Option<Number>,
    pub short_quantity: Option<Number>,
    pub tax_lot_average_long_price: Option<Number>,
    pub tax_lot_average_short_price: Option<Number>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountNumberHash {
    pub account_number: Option<String>,
    pub hash_value: Option<String>,
}

// ---------------------------------------------------------------------------
// Order types
// ---------------------------------------------------------------------------

/// Recursive order: `child_order_strategies` and `replacing_order_collection`
/// contain `Vec<Order>`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub account_number: Option<i64>,
    pub activation_price: Option<Number>,
    pub cancel_time: Option<String>,
    pub cancelable: Option<bool>,
    pub child_order_strategies: Option<Vec<Order>>,
    pub close_time: Option<String>,
    pub complex_order_strategy_type: Option<ComplexOrderStrategyType>,
    pub destination_link_name: Option<String>,
    pub duration: Option<Duration>,
    pub editable: Option<bool>,
    pub entered_time: Option<String>,
    pub filled_quantity: Option<Number>,
    pub order_activity_collection: Option<Vec<OrderActivity>>,
    pub order_id: Option<i64>,
    pub order_leg_collection: Option<Vec<OrderLegCollection>>,
    pub order_strategy_type: Option<OrderStrategyType>,
    pub order_type: Option<OrderType>,
    pub price: Option<Number>,
    pub price_link_basis: Option<PriceLinkBasis>,
    pub price_link_type: Option<PriceLinkType>,
    pub quantity: Option<Number>,
    pub release_time: Option<String>,
    pub remaining_quantity: Option<Number>,
    pub replacing_order_collection: Option<Vec<Order>>,
    pub requested_destination: Option<RequestedDestination>,
    pub session: Option<Session>,
    pub special_instruction: Option<SpecialInstruction>,
    pub status: Option<OrderStatus>,
    pub status_description: Option<String>,
    pub stop_price: Option<Number>,
    pub stop_price_link_basis: Option<StopPriceLinkBasis>,
    pub stop_price_link_type: Option<StopPriceLinkType>,
    pub stop_price_offset: Option<Number>,
    pub stop_type: Option<StopType>,
    pub tag: Option<String>,
    pub tax_lot_method: Option<TaxLotMethod>,
}

/// Recursive order request: `child_order_strategies` and
/// `replacing_order_collection` contain `Vec<OrderRequest>`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    pub account_number: Option<i64>,
    pub activation_price: Option<Number>,
    pub cancel_time: Option<String>,
    pub cancelable: Option<bool>,
    pub child_order_strategies: Option<Vec<OrderRequest>>,
    pub close_time: Option<String>,
    pub complex_order_strategy_type: Option<ComplexOrderStrategyType>,
    pub destination_link_name: Option<String>,
    pub duration: Option<Duration>,
    pub editable: Option<bool>,
    pub entered_time: Option<String>,
    pub filled_quantity: Option<Number>,
    pub order_activity_collection: Option<Vec<OrderActivity>>,
    pub order_id: Option<i64>,
    pub order_leg_collection: Option<Vec<OrderLegCollection>>,
    pub order_strategy_type: Option<OrderStrategyType>,
    pub order_type: Option<OrderTypeRequest>,
    pub price: Option<Number>,
    pub price_link_basis: Option<PriceLinkBasis>,
    pub price_link_type: Option<PriceLinkType>,
    pub quantity: Option<Number>,
    pub release_time: Option<String>,
    pub remaining_quantity: Option<Number>,
    pub replacing_order_collection: Option<Vec<OrderRequest>>,
    pub session: Option<Session>,
    pub special_instruction: Option<SpecialInstruction>,
    pub status: Option<OrderStatus>,
    pub status_description: Option<String>,
    pub stop_price: Option<Number>,
    pub stop_price_link_basis: Option<StopPriceLinkBasis>,
    pub stop_price_link_type: Option<StopPriceLinkType>,
    pub stop_price_offset: Option<Number>,
    pub stop_type: Option<StopType>,
    pub tax_lot_method: Option<TaxLotMethod>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderLegCollection {
    pub div_cap_gains: Option<DividendCapitalGains>,
    pub instruction: Option<Instruction>,
    pub instrument: Option<AccountsInstrument>,
    pub leg_id: Option<i64>,
    pub order_leg_type: Option<InstrumentAssetType>,
    pub position_effect: Option<PositionEffect>,
    pub quantity: Option<Number>,
    pub quantity_type: Option<QuantityType>,
    pub to_symbol: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderActivity {
    pub activity_type: Option<OrderActivityType>,
    pub execution_legs: Option<Vec<ExecutionLeg>>,
    pub execution_type: Option<ExecutionType>,
    pub order_remaining_quantity: Option<Number>,
    pub quantity: Option<Number>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLeg {
    pub instrument_id: Option<i64>,
    pub leg_id: Option<i64>,
    pub mismarked_quantity: Option<Number>,
    pub price: Option<Number>,
    pub quantity: Option<Number>,
    pub time: Option<String>,
}

// ---------------------------------------------------------------------------
// Preview Order & supporting types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreviewOrder {
    pub commission_and_fee: Option<CommissionAndFee>,
    pub order_id: Option<i64>,
    pub order_strategy: Option<OrderStrategy>,
    pub order_validation_result: Option<OrderValidationResult>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderStrategy {
    pub account_number: Option<String>,
    pub advanced_order_type: Option<AdvancedOrderType>,
    pub all_or_none: Option<bool>,
    pub amount_indicator: Option<AmountIndicator>,
    pub close_time: Option<String>,
    pub discretionary: Option<bool>,
    pub duration: Option<Duration>,
    pub entered_time: Option<String>,
    pub filled_quantity: Option<Number>,
    pub order_balance: Option<OrderBalance>,
    pub order_legs: Option<Vec<OrderLeg>>,
    pub order_strategy_type: Option<OrderStrategyType>,
    pub order_type: Option<OrderType>,
    pub order_value: Option<Number>,
    pub order_version: Option<Number>,
    pub price: Option<Number>,
    pub quantity: Option<Number>,
    pub remaining_quantity: Option<Number>,
    pub sell_non_marginable_first: Option<bool>,
    pub session: Option<Session>,
    pub settlement_instruction: Option<SettlementInstruction>,
    pub status: Option<OrderStatus>,
    pub strategy: Option<ComplexOrderStrategyType>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderBalance {
    pub order_value: Option<Number>,
    pub projected_available_fund: Option<Number>,
    pub projected_buying_power: Option<Number>,
    pub projected_commission: Option<Number>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderLeg {
    pub ask_price: Option<Number>,
    pub asset_type: Option<AssetType>,
    pub bid_price: Option<Number>,
    pub final_symbol: Option<String>,
    pub instruction: Option<Instruction>,
    pub last_price: Option<Number>,
    pub leg_id: Option<Number>,
    pub mark_price: Option<Number>,
    pub projected_commission: Option<Number>,
    pub quantity: Option<Number>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderValidationResult {
    pub accepts: Option<Vec<OrderValidationDetail>>,
    pub alerts: Option<Vec<OrderValidationDetail>>,
    pub rejects: Option<Vec<OrderValidationDetail>>,
    pub reviews: Option<Vec<OrderValidationDetail>>,
    pub warns: Option<Vec<OrderValidationDetail>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderValidationDetail {
    pub activity_message: Option<String>,
    pub message: Option<String>,
    pub original_severity: Option<ApiRuleAction>,
    pub override_name: Option<String>,
    pub override_severity: Option<ApiRuleAction>,
    pub validation_rule_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Commission & Fee types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommissionAndFee {
    pub commission: Option<Commission>,
    pub fee: Option<Fees>,
    pub true_commission: Option<Commission>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Commission {
    pub commission_legs: Option<Vec<CommissionLeg>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommissionLeg {
    pub commission_values: Option<Vec<CommissionValue>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommissionValue {
    #[serde(rename = "type")]
    pub r#type: Option<FeeType>,
    pub value: Option<Number>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
    pub fee_legs: Option<Vec<FeeLeg>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeeLeg {
    pub fee_values: Option<Vec<FeeValue>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FeeValue {
    #[serde(rename = "type")]
    pub r#type: Option<FeeType>,
    pub value: Option<Number>,
}

// ---------------------------------------------------------------------------
// Transaction types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub account_number: Option<String>,
    pub activity_id: Option<i64>,
    pub activity_type: Option<TransactionActivityType>,
    pub description: Option<String>,
    pub net_amount: Option<Number>,
    pub order_id: Option<i64>,
    pub position_id: Option<i64>,
    pub settlement_date: Option<String>,
    pub status: Option<TransactionStatus>,
    pub sub_account: Option<SubAccount>,
    pub time: Option<String>,
    pub trade_date: Option<String>,
    pub transfer_items: Option<Vec<TransferItem>>,
    #[serde(rename = "type")]
    pub r#type: Option<TransactionType>,
    pub user: Option<UserDetails>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransferItem {
    pub amount: Option<Number>,
    pub cost: Option<Number>,
    pub fee_type: Option<FeeType>,
    pub instrument: Option<TransactionInstrument>,
    pub position_effect: Option<PositionEffect>,
    pub price: Option<Number>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserDetails {
    pub broker_rep_code: Option<String>,
    pub cd_domain_id: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub login: Option<String>,
    pub system_user_name: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<UserType>,
    pub user_id: Option<i64>,
}

// ---------------------------------------------------------------------------
// Transaction instruments
// ---------------------------------------------------------------------------

/// Discriminated union of transaction instrument types.
/// Variants ordered most-specific first for `untagged` deserialization.
/// Check `asset_type` on the deserialized struct to identify the kind.
/// The `Option` variant uses `Box` to break the recursive cycle with
/// `TransactionOption.deliverable`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TransactionInstrument {
    Option(Box<TransactionOption>),
    MutualFund(TransactionMutualFund),
    Future(TransactionFuture),
    FixedIncome(TransactionFixedIncome),
    Forex(TransactionForex),
    Index(TransactionIndex),
    CollectiveInvestment(TransactionCollectiveInvestment),
    Equity(TransactionEquity),
    CashEquivalent(TransactionCashEquivalent),
    Product(TransactionProduct),
    Currency(TransactionCurrency),
}

/// allOf TransactionBaseInstrument + equity type.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionEquity {
    // Inlined from TransactionBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // TransactionEquity-specific
    #[serde(rename = "type")]
    pub r#type: Option<TransactionEquityType>,
}

/// allOf TransactionBaseInstrument + option fields.
/// Contains a recursive `deliverable` field of type `TransactionInstrument`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOption {
    // Inlined from TransactionBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // TransactionOption-specific
    pub deliverable: Option<TransactionInstrument>,
    pub expiration_date: Option<String>,
    pub option_deliverables: Option<Vec<TransactionApiOptionDeliverable>>,
    pub option_premium_multiplier: Option<i64>,
    pub put_call: Option<OptionPutCall>,
    pub strike_price: Option<Number>,
    #[serde(rename = "type")]
    pub r#type: Option<OptionType>,
    pub underlying_cusip: Option<String>,
    pub underlying_symbol: Option<String>,
}

/// allOf TransactionBaseInstrument + fixed-income fields.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionFixedIncome {
    // Inlined from TransactionBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // TransactionFixedIncome-specific
    pub factor: Option<Number>,
    pub maturity_date: Option<String>,
    pub multiplier: Option<Number>,
    #[serde(rename = "type")]
    pub r#type: Option<TransactionFixedIncomeType>,
    pub variable_rate: Option<Number>,
}

/// allOf TransactionBaseInstrument + cash-equivalent type.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionCashEquivalent {
    // Inlined from TransactionBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // TransactionCashEquivalent-specific
    #[serde(rename = "type")]
    pub r#type: Option<CashEquivalentType>,
}

/// allOf TransactionBaseInstrument + mutual-fund fields.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionMutualFund {
    // Inlined from TransactionBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // TransactionMutualFund-specific
    pub exchange_cutoff_time: Option<String>,
    pub fund_family_name: Option<String>,
    pub fund_family_symbol: Option<String>,
    pub fund_group: Option<String>,
    pub purchase_cutoff_time: Option<String>,
    pub redemption_cutoff_time: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<TransactionMutualFundType>,
}

/// allOf TransactionBaseInstrument + collective-investment type.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionCollectiveInvestment {
    // Inlined from TransactionBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // CollectiveInvestment-specific
    #[serde(rename = "type")]
    pub r#type: Option<CollectiveInvestmentType>,
}

/// allOf TransactionBaseInstrument (no extra fields).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionCurrency {
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
}

/// allOf TransactionBaseInstrument + forex fields.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionForex {
    // Inlined from TransactionBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // Forex-specific
    pub base_currency: Option<TransactionCurrency>,
    pub counter_currency: Option<TransactionCurrency>,
    #[serde(rename = "type")]
    pub r#type: Option<ForexType>,
}

/// allOf TransactionBaseInstrument + future fields.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionFuture {
    // Inlined from TransactionBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // Future-specific
    pub active_contract: Option<bool>,
    pub expiration_date: Option<String>,
    pub first_notice_date: Option<String>,
    pub last_trading_date: Option<String>,
    pub multiplier: Option<Number>,
    #[serde(rename = "type")]
    pub r#type: Option<FutureType>,
}

/// allOf TransactionBaseInstrument + index fields.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionIndex {
    // Inlined from TransactionBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // Index-specific
    pub active_contract: Option<bool>,
    #[serde(rename = "type")]
    pub r#type: Option<IndexType>,
}

/// allOf TransactionBaseInstrument + product type.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionProduct {
    // Inlined from TransactionBaseInstrument
    pub asset_type: Option<InstrumentAssetType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub instrument_id: Option<i64>,
    pub net_change: Option<Number>,
    pub symbol: Option<String>,
    // Product-specific
    #[serde(rename = "type")]
    pub r#type: Option<ProductType>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionApiOptionDeliverable {
    pub asset_type: Option<AssetType>,
    pub deliverable: Option<TransactionInstrument>,
    pub deliverable_number: Option<i64>,
    pub deliverable_units: Option<Number>,
    pub root_symbol: Option<String>,
    pub strike_percent: Option<i64>,
}

// ---------------------------------------------------------------------------
// User Preferences
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserPreference {
    pub accounts: Option<Vec<UserPreferenceAccount>>,
    pub offers: Option<Vec<Offer>>,
    pub streamer_info: Option<Vec<StreamerInfo>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferenceAccount {
    pub account_color: Option<String>,
    pub account_number: Option<String>,
    pub auto_position_effect: Option<bool>,
    pub display_acct_id: Option<String>,
    pub nick_name: Option<String>,
    pub primary_account: Option<bool>,
    #[serde(rename = "type")]
    pub r#type: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Offer {
    pub level_2_permissions: Option<bool>,
    pub mkt_data_permission: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StreamerInfo {
    pub schwab_client_channel: Option<String>,
    pub schwab_client_correl_id: Option<String>,
    pub schwab_client_customer_id: Option<String>,
    pub schwab_client_function_id: Option<String>,
    pub streamer_socket_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Misc
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DateParam {
    pub date: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::n;

    /// Margin account deserialized through the internally-tagged SecuritiesAccount enum.
    /// Verifies allOf-inlined fields from SecuritiesAccountBase are present alongside
    /// margin-specific balance structs.
    #[test]
    fn deserialize_account_margin() {
        let json = r#"{
            "securitiesAccount": {
                "type": "MARGIN",
                "accountNumber": "12345678",
                "isClosingOnlyRestricted": false,
                "isDayTrader": true,
                "pfcbFlag": false,
                "roundTrips": 3,
                "currentBalances": {
                    "availableFunds": 25000.50,
                    "buyingPower": 50000.0,
                    "equity": 75000.0,
                    "marginBalance": -25000.0,
                    "shortBalance": 0.0
                },
                "initialBalances": {
                    "accountValue": 80000.0,
                    "cashBalance": 5000.0,
                    "equity": 75000.0,
                    "liquidationValue": 80000.0,
                    "longStockValue": 70000.0
                }
            }
        }"#;

        let account: Account = serde_json::from_str(json).unwrap();
        let sa = account.securities_account.unwrap();
        match sa {
            SecuritiesAccount::Margin(m) => {
                // allOf-inlined fields from SecuritiesAccountBase
                assert_eq!(m.account_number, Some("12345678".to_string()));
                assert_eq!(m.is_closing_only_restricted, Some(false));
                assert_eq!(m.is_day_trader, Some(true));
                assert_eq!(m.pfcb_flag, Some(false));
                assert_eq!(m.round_trips, Some(3));
                // r#type is None: serde(tag = "type") consumes the tag field

                // Margin-specific balances
                let bal = m.current_balances.unwrap();
                assert_eq!(bal.available_funds, Some(n(25000.50)));
                assert_eq!(bal.buying_power, Some(n(50000.0)));
                assert_eq!(bal.equity, Some(n(75000.0)));

                let init = m.initial_balances.unwrap();
                assert_eq!(init.account_value, Some(n(80000.0)));
                assert_eq!(init.long_stock_value, Some(n(70000.0)));
            }
            other => panic!("expected Margin variant, got {other:?}"),
        }
    }

    /// Cash account variant through the same internally-tagged enum.
    /// Verifies allOf-inlined fields and cash-specific balance structs.
    #[test]
    fn deserialize_account_cash() {
        let json = r#"{
            "securitiesAccount": {
                "type": "CASH",
                "accountNumber": "87654321",
                "isClosingOnlyRestricted": true,
                "isDayTrader": false,
                "pfcbFlag": true,
                "roundTrips": 0,
                "currentBalances": {
                    "cashAvailableForTrading": 10000.0,
                    "cashAvailableForWithdrawal": 8000.0,
                    "totalCash": 10000.0,
                    "unsettledCash": 2000.0
                },
                "initialBalances": {
                    "accountValue": 12000.0,
                    "cashBalance": 10000.0,
                    "liquidationValue": 12000.0
                }
            }
        }"#;

        let account: Account = serde_json::from_str(json).unwrap();
        let sa = account.securities_account.unwrap();
        match sa {
            SecuritiesAccount::Cash(c) => {
                assert_eq!(c.account_number, Some("87654321".to_string()));
                assert_eq!(c.is_closing_only_restricted, Some(true));
                assert_eq!(c.is_day_trader, Some(false));
                assert_eq!(c.pfcb_flag, Some(true));
                assert_eq!(c.round_trips, Some(0));
                // r#type is None: serde(tag = "type") consumes the tag field

                let bal = c.current_balances.unwrap();
                assert_eq!(bal.cash_available_for_trading, Some(n(10000.0)));
                assert_eq!(bal.cash_available_for_withdrawal, Some(n(8000.0)));
                assert_eq!(bal.total_cash, Some(n(10000.0)));
                assert_eq!(bal.unsettled_cash, Some(n(2000.0)));

                let init = c.initial_balances.unwrap();
                assert_eq!(init.account_value, Some(n(12000.0)));
                assert_eq!(init.cash_balance, Some(n(10000.0)));
            }
            other => panic!("expected Cash variant, got {other:?}"),
        }
    }

    /// Simple AccountNumberHash struct.
    #[test]
    fn deserialize_account_number_hash() {
        let json = r#"{
            "accountNumber": "12345678",
            "hashValue": "ABCDEF1234567890"
        }"#;

        let anh: AccountNumberHash = serde_json::from_str(json).unwrap();
        assert_eq!(anh.account_number, Some("12345678".to_string()));
        assert_eq!(anh.hash_value, Some("ABCDEF1234567890".to_string()));
    }

    /// Order with recursive childOrderStrategies containing a nested child order.
    /// Verifies Vec<Order> recursion deserializes correctly.
    #[test]
    fn deserialize_order_with_children() {
        let json = r#"{
            "orderId": 100001,
            "orderType": "LIMIT",
            "session": "NORMAL",
            "duration": "DAY",
            "orderStrategyType": "TRIGGER",
            "price": 150.0,
            "quantity": 10.0,
            "filledQuantity": 0.0,
            "remainingQuantity": 10.0,
            "status": "WORKING",
            "cancelable": true,
            "editable": false,
            "enteredTime": "2024-01-15T10:30:00+0000",
            "orderLegCollection": [{
                "instruction": "BUY",
                "quantity": 10.0,
                "instrument": {
                    "assetType": "EQUITY",
                    "symbol": "AAPL",
                    "cusip": "037833100",
                    "description": "Apple Inc"
                }
            }],
            "childOrderStrategies": [{
                "orderId": 100002,
                "orderType": "LIMIT",
                "session": "NORMAL",
                "duration": "GOOD_TILL_CANCEL",
                "orderStrategyType": "SINGLE",
                "price": 160.0,
                "quantity": 10.0,
                "status": "AWAITING_PARENT_ORDER",
                "cancelable": false,
                "editable": false,
                "orderLegCollection": [{
                    "instruction": "SELL",
                    "quantity": 10.0,
                    "instrument": {
                        "assetType": "EQUITY",
                        "symbol": "AAPL"
                    }
                }]
            }]
        }"#;

        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.order_id, Some(100001));
        assert_eq!(order.order_type, Some(OrderType::Limit));
        assert_eq!(order.session, Some(Session::Normal));
        assert_eq!(order.duration, Some(Duration::Day));
        assert_eq!(order.order_strategy_type, Some(OrderStrategyType::Trigger));
        assert_eq!(order.price, Some(n(150.0)));
        assert_eq!(order.status, Some(OrderStatus::Working));
        assert_eq!(order.cancelable, Some(true));

        let legs = order.order_leg_collection.unwrap();
        assert_eq!(legs.len(), 1);
        assert_eq!(legs[0].instruction, Some(Instruction::Buy));
        assert_eq!(legs[0].quantity, Some(n(10.0)));

        // Recursive children
        let children = order.child_order_strategies.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].order_id, Some(100002));
        assert_eq!(children[0].status, Some(OrderStatus::AwaitingParentOrder));
        assert_eq!(children[0].duration, Some(Duration::GoodTillCancel));
        assert_eq!(children[0].order_strategy_type, Some(OrderStrategyType::Single));
        assert_eq!(children[0].price, Some(n(160.0)));

        let child_legs = children[0].order_leg_collection.as_ref().unwrap();
        assert_eq!(child_legs[0].instruction, Some(Instruction::Sell));
    }

    /// Simple order without children or activity. Verifies core fields.
    #[test]
    fn deserialize_order_simple() {
        let json = r#"{
            "orderId": 200001,
            "orderType": "MARKET",
            "session": "NORMAL",
            "duration": "DAY",
            "orderStrategyType": "SINGLE",
            "quantity": 5.0,
            "filledQuantity": 5.0,
            "remainingQuantity": 0.0,
            "status": "FILLED",
            "cancelable": false,
            "editable": false,
            "enteredTime": "2024-01-15T14:00:00+0000",
            "closeTime": "2024-01-15T14:00:01+0000",
            "tag": "API_CLIENT",
            "orderLegCollection": [{
                "instruction": "BUY",
                "quantity": 5.0,
                "instrument": {
                    "assetType": "EQUITY",
                    "symbol": "MSFT"
                }
            }]
        }"#;

        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.order_id, Some(200001));
        assert_eq!(order.order_type, Some(OrderType::Market));
        assert_eq!(order.status, Some(OrderStatus::Filled));
        assert_eq!(order.filled_quantity, Some(n(5.0)));
        assert_eq!(order.remaining_quantity, Some(n(0.0)));
        assert_eq!(order.tag, Some("API_CLIENT".to_string()));
        assert!(order.child_order_strategies.is_none());
    }

    /// Transaction with a TransferItem containing a typed TransactionInstrument.
    /// Verifies the full nesting: Transaction -> TransferItem -> TransactionInstrument.
    #[test]
    fn deserialize_transaction() {
        let json = r#"{
            "accountNumber": "12345678",
            "activityId": 99887766,
            "activityType": "EXECUTION",
            "type": "TRADE",
            "status": "VALID",
            "tradeDate": "2024-01-15T00:00:00+0000",
            "settlementDate": "2024-01-17T00:00:00+0000",
            "netAmount": -1500.0,
            "subAccount": "CASH",
            "description": "BUY TRADE",
            "orderId": 300001,
            "transferItems": [{
                "instrument": {
                    "assetType": "EQUITY",
                    "symbol": "MSFT",
                    "cusip": "594918104",
                    "description": "Microsoft Corp",
                    "instrumentId": 54321,
                    "type": "COMMON_STOCK"
                },
                "amount": 10.0,
                "cost": 1500.0,
                "price": 150.0,
                "positionEffect": "OPENING"
            }]
        }"#;

        let txn: Transaction = serde_json::from_str(json).unwrap();
        assert_eq!(txn.account_number, Some("12345678".to_string()));
        assert_eq!(txn.activity_id, Some(99887766));
        assert_eq!(txn.activity_type, Some(TransactionActivityType::Execution));
        assert_eq!(txn.r#type, Some(TransactionType::Trade));
        assert_eq!(txn.status, Some(TransactionStatus::Valid));
        assert_eq!(txn.net_amount, Some(n(-1500.0)));
        assert_eq!(txn.sub_account, Some(SubAccount::Cash));
        assert_eq!(txn.order_id, Some(300001));

        let items = txn.transfer_items.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].amount, Some(n(10.0)));
        assert_eq!(items[0].cost, Some(n(1500.0)));
        assert_eq!(items[0].price, Some(n(150.0)));
        assert_eq!(items[0].position_effect, Some(PositionEffect::Opening));

        // Verify the instrument deserialized as Equity variant
        match items[0].instrument.as_ref().unwrap() {
            TransactionInstrument::Equity(eq) => {
                assert_eq!(eq.symbol, Some("MSFT".to_string()));
                assert_eq!(eq.cusip, Some("594918104".to_string()));
                assert_eq!(eq.r#type, Some(TransactionEquityType::CommonStock));
                assert_eq!(eq.asset_type, Some(InstrumentAssetType::Equity));
            }
            other => panic!("expected Equity variant, got {other:?}"),
        }
    }

    /// TransactionInstrument equity variant via untagged deserialization.
    /// The `type` field's enum (TransactionEquityType) rejects earlier variants
    /// (whose `type` maps to OptionType, etc.), cascading to the Equity variant.
    #[test]
    fn deserialize_transaction_instrument_equity() {
        let json = r#"{
            "assetType": "EQUITY",
            "symbol": "GOOG",
            "cusip": "02079K107",
            "description": "Alphabet Inc Class C",
            "instrumentId": 11111,
            "netChange": 2.35,
            "type": "COMMON_STOCK"
        }"#;

        let instr: TransactionInstrument = serde_json::from_str(json).unwrap();
        match instr {
            TransactionInstrument::Equity(eq) => {
                assert_eq!(eq.symbol, Some("GOOG".to_string()));
                assert_eq!(eq.asset_type, Some(InstrumentAssetType::Equity));
                assert_eq!(eq.r#type, Some(TransactionEquityType::CommonStock));
                assert_eq!(eq.net_change, Some(n(2.35)));
                assert_eq!(eq.instrument_id, Some(11111));
            }
            other => panic!("expected Equity variant, got {other:?}"),
        }
    }

    /// TransactionInstrument option variant via untagged deserialization.
    /// Verifies Box<TransactionOption> content is accessible and option-specific
    /// fields (putCall, strikePrice, expirationDate) populate correctly.
    #[test]
    fn deserialize_transaction_instrument_option() {
        let json = r#"{
            "assetType": "OPTION",
            "symbol": "AAPL  240119C00170000",
            "description": "AAPL Jan 19 2024 170 Call",
            "instrumentId": 67890,
            "cusip": "0AAPL.XA40170000",
            "putCall": "CALL",
            "strikePrice": 170.0,
            "expirationDate": "2024-01-19",
            "type": "VANILLA",
            "underlyingSymbol": "AAPL",
            "underlyingCusip": "037833100",
            "optionPremiumMultiplier": 100
        }"#;

        let instr: TransactionInstrument = serde_json::from_str(json).unwrap();
        match instr {
            TransactionInstrument::Option(boxed) => {
                // Verify Box<TransactionOption> content is accessible
                assert_eq!(boxed.symbol, Some("AAPL  240119C00170000".to_string()));
                assert_eq!(boxed.asset_type, Some(InstrumentAssetType::Option));
                assert_eq!(boxed.put_call, Some(OptionPutCall::Call));
                assert_eq!(boxed.strike_price, Some(n(170.0)));
                assert_eq!(
                    boxed.expiration_date,
                    Some("2024-01-19".to_string())
                );
                assert_eq!(boxed.r#type, Some(OptionType::Vanilla));
                assert_eq!(boxed.underlying_symbol, Some("AAPL".to_string()));
                assert_eq!(boxed.underlying_cusip, Some("037833100".to_string()));
                assert_eq!(boxed.option_premium_multiplier, Some(100));
            }
            other => panic!("expected Option variant, got {other:?}"),
        }
    }

    /// PreviewOrder with commission and fee sub-objects.
    /// Verifies the deeply nested commission/fee value structures.
    #[test]
    fn deserialize_preview_order() {
        let json = r#"{
            "orderId": 400001,
            "commissionAndFee": {
                "commission": {
                    "commissionLegs": [{
                        "commissionValues": [{
                            "type": "COMMISSION",
                            "value": 0.65
                        }]
                    }]
                },
                "fee": {
                    "feeLegs": [{
                        "feeValues": [{
                            "type": "SEC_FEE",
                            "value": 0.01
                        }, {
                            "type": "TAF_FEE",
                            "value": 0.02
                        }]
                    }]
                },
                "trueCommission": {
                    "commissionLegs": [{
                        "commissionValues": [{
                            "type": "COMMISSION",
                            "value": 0.00
                        }]
                    }]
                }
            },
            "orderStrategy": {
                "orderType": "LIMIT",
                "session": "NORMAL",
                "duration": "DAY",
                "orderStrategyType": "SINGLE",
                "price": 150.0,
                "quantity": 10.0,
                "status": "ACCEPTED"
            },
            "orderValidationResult": {
                "alerts": [{
                    "message": "You are about to place a marketable limit order.",
                    "validationRuleName": "MARKETABLE_LIMIT"
                }],
                "accepts": [],
                "rejects": [],
                "reviews": [],
                "warns": []
            }
        }"#;

        let preview: PreviewOrder = serde_json::from_str(json).unwrap();
        assert_eq!(preview.order_id, Some(400001));

        // Commission
        let caf = preview.commission_and_fee.unwrap();
        let comm_legs = caf.commission.unwrap().commission_legs.unwrap();
        let comm_val = &comm_legs[0].commission_values.as_ref().unwrap()[0];
        assert_eq!(comm_val.r#type, Some(FeeType::Commission));
        assert_eq!(comm_val.value, Some(n(0.65)));

        // Fees
        let fee_legs = caf.fee.unwrap().fee_legs.unwrap();
        let fee_vals = fee_legs[0].fee_values.as_ref().unwrap();
        assert_eq!(fee_vals.len(), 2);
        assert_eq!(fee_vals[0].r#type, Some(FeeType::SecFee));
        assert_eq!(fee_vals[0].value, Some(n(0.01)));
        assert_eq!(fee_vals[1].r#type, Some(FeeType::TafFee));
        assert_eq!(fee_vals[1].value, Some(n(0.02)));

        // OrderStrategy
        let strat = preview.order_strategy.unwrap();
        assert_eq!(strat.order_type, Some(OrderType::Limit));
        assert_eq!(strat.session, Some(Session::Normal));
        assert_eq!(strat.status, Some(OrderStatus::Accepted));
        assert_eq!(strat.price, Some(n(150.0)));

        // Validation result
        let vr = preview.order_validation_result.unwrap();
        let alerts = vr.alerts.unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(
            alerts[0].validation_rule_name,
            Some("MARKETABLE_LIMIT".to_string())
        );
    }

    /// UserPreference with accounts, offers, and streamerInfo arrays.
    #[test]
    fn deserialize_user_preference() {
        let json = r#"{
            "accounts": [{
                "accountNumber": "12345678",
                "primaryAccount": true,
                "type": "BROKERAGE",
                "nickName": "My Trading Account",
                "accountColor": "Blue",
                "displayAcctId": "...5678",
                "autoPositionEffect": true
            }],
            "offers": [{
                "level2Permissions": true,
                "mktDataPermission": "NP"
            }],
            "streamerInfo": [{
                "schwabClientCustomerId": "CUST123",
                "schwabClientCorrelId": "CORR456",
                "schwabClientChannel": "client",
                "schwabClientFunctionId": "FUNC789",
                "streamerSocketUrl": "wss://streamer.schwab.com/ws"
            }]
        }"#;

        let pref: UserPreference = serde_json::from_str(json).unwrap();

        let accounts = pref.accounts.unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account_number, Some("12345678".to_string()));
        assert_eq!(accounts[0].primary_account, Some(true));
        assert_eq!(accounts[0].r#type, Some("BROKERAGE".to_string()));
        assert_eq!(
            accounts[0].nick_name,
            Some("My Trading Account".to_string())
        );
        assert_eq!(accounts[0].auto_position_effect, Some(true));

        let offers = pref.offers.unwrap();
        assert_eq!(offers[0].level_2_permissions, Some(true));
        assert_eq!(offers[0].mkt_data_permission, Some("NP".to_string()));

        let streamer = pref.streamer_info.unwrap();
        assert_eq!(
            streamer[0].schwab_client_customer_id,
            Some("CUST123".to_string())
        );
        assert_eq!(
            streamer[0].streamer_socket_url,
            Some("wss://streamer.schwab.com/ws".to_string())
        );
    }
}
