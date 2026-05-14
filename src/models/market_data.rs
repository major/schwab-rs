use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Number;
use super::enums::*;

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// Map of symbol to its quote response object.
pub type QuoteResponse = HashMap<String, QuoteResponseObject>;

// ---------------------------------------------------------------------------
// QuoteResponseObject (untagged union of all asset-type responses + error)
// ---------------------------------------------------------------------------

/// Quote response dispatched by asset type.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
#[allow(missing_docs)]
pub enum QuoteResponseObject {
    Equity(EquityResponse),
    Option(OptionResponse),
    MutualFund(MutualFundResponse),
    Forex(ForexResponse),
    Future(FutureResponse),
    FutureOption(FutureOptionResponse),
    Index(IndexResponse),
    Error(QuoteError),
}

// ---------------------------------------------------------------------------
// Per-asset response structs
// ---------------------------------------------------------------------------

/// Equity asset quote response.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct EquityResponse {
    pub asset_main_type: Option<AssetMainType>,
    pub asset_sub_type: Option<EquityAssetSubType>,
    pub extended: Option<ExtendedMarket>,
    pub fundamental: Option<Fundamental>,
    pub quote: Option<EquityQuote>,
    pub quote_type: Option<QuoteType>,
    pub realtime: Option<bool>,
    pub reference: Option<EquityReference>,
    pub regular: Option<RegularMarket>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
}

/// Option contract quote response.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct OptionResponse {
    pub asset_main_type: Option<AssetMainType>,
    pub quote: Option<OptionQuote>,
    pub realtime: Option<bool>,
    pub reference: Option<OptionReference>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
}

/// Forex pair quote response.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct ForexResponse {
    pub asset_main_type: Option<AssetMainType>,
    pub quote: Option<ForexQuote>,
    pub realtime: Option<bool>,
    pub reference: Option<ForexReference>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
}

/// Futures contract quote response.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct FutureResponse {
    pub asset_main_type: Option<AssetMainType>,
    pub quote: Option<FutureQuote>,
    pub realtime: Option<bool>,
    pub reference: Option<FutureReference>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
}

/// Futures option contract quote response.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct FutureOptionResponse {
    pub asset_main_type: Option<AssetMainType>,
    pub quote: Option<FutureOptionQuote>,
    pub realtime: Option<bool>,
    pub reference: Option<FutureOptionReference>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
}

/// Index quote response.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct IndexResponse {
    pub asset_main_type: Option<AssetMainType>,
    pub quote: Option<IndexQuote>,
    pub realtime: Option<bool>,
    pub reference: Option<IndexReference>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
}

/// Mutual fund quote response.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct MutualFundResponse {
    pub asset_main_type: Option<AssetMainType>,
    pub asset_sub_type: Option<MutualFundAssetSubType>,
    pub fundamental: Option<Fundamental>,
    pub quote: Option<MutualFundQuote>,
    pub realtime: Option<bool>,
    pub reference: Option<MutualFundReference>,
    pub ssid: Option<i64>,
    pub symbol: Option<String>,
}

// ---------------------------------------------------------------------------
// Per-asset quote structs
// ---------------------------------------------------------------------------

/// Equity quote market data.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct EquityQuote {
    #[serde(rename = "52WeekHigh")]
    pub week_high_52: Option<Number>,
    #[serde(rename = "52WeekLow")]
    pub week_low_52: Option<Number>,
    #[serde(rename = "askMICId")]
    pub ask_mic_id: Option<String>,
    pub ask_price: Option<Number>,
    pub ask_size: Option<i32>,
    pub ask_time: Option<i64>,
    #[serde(rename = "bidMICId")]
    pub bid_mic_id: Option<String>,
    pub bid_price: Option<Number>,
    pub bid_size: Option<i32>,
    pub bid_time: Option<i64>,
    pub close_price: Option<Number>,
    pub high_price: Option<Number>,
    #[serde(rename = "lastMICId")]
    pub last_mic_id: Option<String>,
    pub last_price: Option<Number>,
    pub last_size: Option<i32>,
    pub low_price: Option<Number>,
    pub mark: Option<Number>,
    pub mark_change: Option<Number>,
    pub mark_percent_change: Option<Number>,
    pub net_change: Option<Number>,
    pub net_percent_change: Option<Number>,
    pub open_price: Option<Number>,
    pub quote_time: Option<i64>,
    pub security_status: Option<String>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
    pub volatility: Option<Number>,
}

/// Option quote market data.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct OptionQuote {
    #[serde(rename = "52WeekHigh")]
    pub week_high_52: Option<Number>,
    #[serde(rename = "52WeekLow")]
    pub week_low_52: Option<Number>,
    pub ask_price: Option<Number>,
    pub ask_size: Option<i32>,
    pub bid_price: Option<Number>,
    pub bid_size: Option<i32>,
    pub close_price: Option<Number>,
    pub delta: Option<Number>,
    pub gamma: Option<Number>,
    pub high_price: Option<Number>,
    pub implied_yield: Option<Number>,
    pub ind_ask_price: Option<Number>,
    pub ind_bid_price: Option<Number>,
    pub ind_quote_time: Option<i64>,
    pub last_price: Option<Number>,
    pub last_size: Option<i32>,
    pub low_price: Option<Number>,
    pub mark: Option<Number>,
    pub mark_change: Option<Number>,
    pub mark_percent_change: Option<Number>,
    pub money_intrinsic_value: Option<Number>,
    pub net_change: Option<Number>,
    pub net_percent_change: Option<Number>,
    pub open_interest: Option<Number>,
    pub open_price: Option<Number>,
    pub quote_time: Option<i64>,
    pub rho: Option<Number>,
    pub security_status: Option<String>,
    pub theoretical_option_value: Option<Number>,
    pub theta: Option<Number>,
    pub time_value: Option<Number>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
    pub underlying_price: Option<Number>,
    pub vega: Option<Number>,
    pub volatility: Option<Number>,
}

/// Forex quote market data.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct ForexQuote {
    #[serde(rename = "52WeekHigh")]
    pub week_high_52: Option<Number>,
    #[serde(rename = "52WeekLow")]
    pub week_low_52: Option<Number>,
    pub ask_price: Option<Number>,
    pub ask_size: Option<i32>,
    pub bid_price: Option<Number>,
    pub bid_size: Option<i32>,
    pub close_price: Option<Number>,
    pub high_price: Option<Number>,
    pub last_price: Option<Number>,
    pub last_size: Option<i32>,
    pub low_price: Option<Number>,
    pub mark: Option<Number>,
    pub net_change: Option<Number>,
    pub net_percent_change: Option<Number>,
    pub open_price: Option<Number>,
    pub quote_time: Option<i64>,
    pub security_status: Option<String>,
    pub tick: Option<Number>,
    pub tick_amount: Option<Number>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
}

/// Futures quote market data.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct FutureQuote {
    #[serde(rename = "askMICId")]
    pub ask_mic_id: Option<String>,
    pub ask_price: Option<Number>,
    pub ask_size: Option<i32>,
    pub ask_time: Option<i64>,
    #[serde(rename = "bidMICId")]
    pub bid_mic_id: Option<String>,
    pub bid_price: Option<Number>,
    pub bid_size: Option<i32>,
    pub bid_time: Option<i64>,
    pub close_price: Option<Number>,
    pub future_percent_change: Option<Number>,
    pub high_price: Option<Number>,
    #[serde(rename = "lastMICId")]
    pub last_mic_id: Option<String>,
    pub last_price: Option<Number>,
    pub last_size: Option<i32>,
    pub low_price: Option<Number>,
    pub mark: Option<Number>,
    pub net_change: Option<Number>,
    pub open_interest: Option<i32>,
    pub open_price: Option<Number>,
    pub quote_time: Option<i64>,
    pub quoted_in_session: Option<bool>,
    pub security_status: Option<String>,
    pub settle_time: Option<i64>,
    pub tick: Option<Number>,
    pub tick_amount: Option<Number>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
}

/// Futures option quote market data.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct FutureOptionQuote {
    #[serde(rename = "askMICId")]
    pub ask_mic_id: Option<String>,
    pub ask_price: Option<Number>,
    pub ask_size: Option<i32>,
    #[serde(rename = "bidMICId")]
    pub bid_mic_id: Option<String>,
    pub bid_price: Option<Number>,
    pub bid_size: Option<i32>,
    pub close_price: Option<Number>,
    pub high_price: Option<Number>,
    #[serde(rename = "lastMICId")]
    pub last_mic_id: Option<String>,
    pub last_price: Option<Number>,
    pub last_size: Option<i32>,
    pub low_price: Option<Number>,
    pub mark: Option<Number>,
    pub mark_change: Option<Number>,
    pub net_change: Option<Number>,
    pub net_percent_change: Option<Number>,
    pub open_interest: Option<i32>,
    pub open_price: Option<Number>,
    pub quote_time: Option<i64>,
    pub security_status: Option<String>,
    /// Spec typo: field is `settlemetPrice` (missing 'n'), kept as-is.
    pub settlemet_price: Option<Number>,
    pub tick: Option<Number>,
    pub tick_amount: Option<Number>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
}

/// Index quote market data.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct IndexQuote {
    #[serde(rename = "52WeekHigh")]
    pub week_high_52: Option<Number>,
    #[serde(rename = "52WeekLow")]
    pub week_low_52: Option<Number>,
    pub close_price: Option<Number>,
    pub high_price: Option<Number>,
    pub last_price: Option<Number>,
    pub low_price: Option<Number>,
    pub net_change: Option<Number>,
    pub net_percent_change: Option<Number>,
    pub open_price: Option<Number>,
    pub security_status: Option<String>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
}

/// Mutual fund quote market data.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct MutualFundQuote {
    #[serde(rename = "52WeekHigh")]
    pub week_high_52: Option<Number>,
    #[serde(rename = "52WeekLow")]
    pub week_low_52: Option<Number>,
    pub close_price: Option<Number>,
    #[serde(rename = "nAV")]
    pub nav: Option<Number>,
    pub net_change: Option<Number>,
    pub net_percent_change: Option<Number>,
    pub security_status: Option<String>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
}

// ---------------------------------------------------------------------------
// Per-asset reference structs
// ---------------------------------------------------------------------------

/// Equity reference details.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct EquityReference {
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub fsi_desc: Option<String>,
    pub htb_quantity: Option<i32>,
    pub htb_rate: Option<Number>,
    pub is_hard_to_borrow: Option<bool>,
    pub is_shortable: Option<bool>,
    pub otc_market_tier: Option<String>,
}

/// Option contract reference details.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct OptionReference {
    pub contract_type: Option<ContractType>,
    pub cusip: Option<String>,
    pub days_to_expiration: Option<i32>,
    pub deliverables: Option<String>,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub exercise_type: Option<ExerciseType>,
    pub expiration_day: Option<i32>,
    pub expiration_month: Option<i32>,
    pub expiration_type: Option<ExpirationType>,
    pub expiration_year: Option<i32>,
    pub is_penny_pilot: Option<bool>,
    pub last_trading_day: Option<i64>,
    pub multiplier: Option<Number>,
    pub settlement_type: Option<SettlementType>,
    pub strike_price: Option<Number>,
    pub underlying: Option<String>,
}

/// Forex pair reference details.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct ForexReference {
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub is_tradable: Option<bool>,
    pub market_maker: Option<String>,
    pub product: Option<String>,
    pub trading_hours: Option<String>,
}

/// Futures contract reference details.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct FutureReference {
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub future_active_symbol: Option<String>,
    pub future_expiration_date: Option<Number>,
    pub future_is_active: Option<bool>,
    pub future_is_tradable: Option<bool>,
    pub future_multiplier: Option<Number>,
    pub future_price_format: Option<String>,
    pub future_settlement_price: Option<Number>,
    pub future_trading_hours: Option<String>,
    pub product: Option<String>,
}

/// Futures option contract reference details.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct FutureOptionReference {
    pub contract_type: Option<ContractType>,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
    pub expiration_date: Option<i64>,
    pub expiration_style: Option<String>,
    pub multiplier: Option<Number>,
    pub strike_price: Option<Number>,
    pub underlying: Option<String>,
}

/// Index reference details.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct IndexReference {
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
}

/// Mutual fund reference details.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct MutualFundReference {
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub exchange_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Shared quote/market structs
// ---------------------------------------------------------------------------

/// Extended hours market data.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct ExtendedMarket {
    pub ask_price: Option<Number>,
    pub ask_size: Option<i32>,
    pub bid_price: Option<Number>,
    pub bid_size: Option<i32>,
    pub last_price: Option<Number>,
    pub last_size: Option<i32>,
    pub mark: Option<Number>,
    pub quote_time: Option<i64>,
    /// Spec declares type=number with format=int64; use f64 to match JSON.
    pub total_volume: Option<Number>,
    pub trade_time: Option<i64>,
}

/// Regular session market data.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct RegularMarket {
    pub regular_market_last_price: Option<Number>,
    pub regular_market_last_size: Option<i32>,
    pub regular_market_net_change: Option<Number>,
    pub regular_market_percent_change: Option<Number>,
    pub regular_market_trade_time: Option<i64>,
}

/// Fundamental financial data for an equity.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Fundamental {
    pub avg_10_days_volume: Option<Number>,
    pub avg_1_year_volume: Option<Number>,
    pub declaration_date: Option<String>,
    pub div_amount: Option<Number>,
    pub div_ex_date: Option<String>,
    pub div_freq: Option<u32>,
    pub div_pay_amount: Option<Number>,
    pub div_pay_date: Option<String>,
    pub div_yield: Option<Number>,
    pub eps: Option<Number>,
    pub fund_leverage_factor: Option<Number>,
    pub fund_strategy: Option<FundStrategy>,
    pub next_div_ex_date: Option<String>,
    pub next_div_pay_date: Option<String>,
    pub pe_ratio: Option<Number>,
}

// ---------------------------------------------------------------------------
// QuoteError
// ---------------------------------------------------------------------------

/// Error details for a failed quote lookup.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct QuoteError {
    pub invalid_cusips: Option<Vec<String>>,
    #[serde(rename = "invalidSSIDs")]
    pub invalid_ssids: Option<Vec<i64>>,
    pub invalid_symbols: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Option chain types
// ---------------------------------------------------------------------------

/// Option chain data for a symbol.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct OptionChain {
    pub call_exp_date_map: Option<HashMap<String, HashMap<String, Vec<OptionContract>>>>,
    pub days_to_expiration: Option<Number>,
    pub interest_rate: Option<Number>,
    pub interval: Option<Number>,
    pub is_delayed: Option<bool>,
    pub is_index: Option<bool>,
    pub put_exp_date_map: Option<HashMap<String, HashMap<String, Vec<OptionContract>>>>,
    pub status: Option<String>,
    pub strategy: Option<OptionStrategy>,
    pub symbol: Option<String>,
    pub underlying: Option<Underlying>,
    pub underlying_price: Option<Number>,
    pub volatility: Option<Number>,
}

/// All numeric fields use f64 regardless of the spec's declared integer types,
/// because the Schwab API frequently returns floats where the spec says integer.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct OptionContract {
    pub ask_price: Option<Number>,
    pub ask_size: Option<Number>,
    pub bid_price: Option<Number>,
    pub bid_size: Option<Number>,
    pub close_price: Option<Number>,
    pub days_to_expiration: Option<Number>,
    pub deliverable_note: Option<String>,
    pub delta: Option<Number>,
    pub description: Option<String>,
    pub exchange_name: Option<String>,
    pub expiration_date: Option<String>,
    pub expiration_type: Option<ExpirationType>,
    pub gamma: Option<Number>,
    pub high_price: Option<Number>,
    pub intrinsic_value: Option<Number>,
    pub is_in_the_money: Option<bool>,
    pub is_index_option: Option<bool>,
    pub is_mini: Option<bool>,
    pub is_non_standard: Option<bool>,
    pub is_penny_pilot: Option<bool>,
    pub last_price: Option<Number>,
    pub last_size: Option<Number>,
    pub last_trading_day: Option<Number>,
    pub low_price: Option<Number>,
    pub mark_change: Option<Number>,
    pub mark_percent_change: Option<Number>,
    pub mark_price: Option<Number>,
    pub multiplier: Option<Number>,
    pub net_change: Option<Number>,
    pub open_interest: Option<Number>,
    pub open_price: Option<Number>,
    pub option_deliverables_list: Option<Vec<OptionDeliverables>>,
    pub option_root: Option<String>,
    pub percent_change: Option<Number>,
    pub put_call: Option<PutCall>,
    pub quote_time_in_long: Option<Number>,
    pub rho: Option<Number>,
    pub settlement_type: Option<SettlementType>,
    pub strike_price: Option<Number>,
    pub symbol: Option<String>,
    pub theoretical_option_value: Option<Number>,
    pub theoretical_volatility: Option<Number>,
    pub theta: Option<Number>,
    pub time_value: Option<Number>,
    pub total_volume: Option<Number>,
    pub trade_date: Option<Number>,
    pub trade_time_in_long: Option<Number>,
    pub vega: Option<Number>,
    pub volatility: Option<Number>,
}

/// Option contract deliverable details.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct OptionDeliverables {
    pub asset_type: Option<String>,
    pub currency_type: Option<String>,
    pub deliverable_units: Option<String>,
    pub symbol: Option<String>,
}

/// Underlying security data for an option chain.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Underlying {
    pub ask: Option<Number>,
    pub ask_size: Option<i32>,
    pub bid: Option<Number>,
    pub bid_size: Option<i32>,
    pub change: Option<Number>,
    pub close: Option<Number>,
    pub delayed: Option<bool>,
    pub description: Option<String>,
    pub exchange_name: Option<ExchangeName>,
    pub fifty_two_week_high: Option<Number>,
    pub fifty_two_week_low: Option<Number>,
    pub high_price: Option<Number>,
    pub last: Option<Number>,
    pub low_price: Option<Number>,
    pub mark: Option<Number>,
    pub mark_change: Option<Number>,
    pub mark_percent_change: Option<Number>,
    pub open_price: Option<Number>,
    pub percent_change: Option<Number>,
    pub quote_time: Option<i64>,
    pub symbol: Option<String>,
    pub total_volume: Option<i64>,
    pub trade_time: Option<i64>,
}

/// Option expiration chain response.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct ExpirationChain {
    pub expiration_list: Option<Vec<Expiration>>,
    pub status: Option<String>,
}

/// Option expiration details.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Expiration {
    pub days_to_expiration: Option<i32>,
    pub expiration: Option<String>,
    pub expiration_type: Option<ExpirationType>,
    pub option_roots: Option<String>,
    pub settlement_type: Option<SettlementType>,
    pub standard: Option<bool>,
}

// ---------------------------------------------------------------------------
// Price history
// ---------------------------------------------------------------------------

/// List of price history candles.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct CandleList {
    pub candles: Option<Vec<Candle>>,
    pub empty: Option<bool>,
    pub previous_close: Option<Number>,
    pub previous_close_date: Option<i64>,
    #[serde(rename = "previousCloseDateISO8601")]
    pub previous_close_date_iso8601: Option<String>,
    pub symbol: Option<String>,
}

/// Single price history candle (OHLCV).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Candle {
    pub close: Option<Number>,
    pub datetime: Option<i64>,
    #[serde(rename = "datetimeISO8601")]
    pub datetime_iso8601: Option<String>,
    pub high: Option<Number>,
    pub low: Option<Number>,
    pub open: Option<Number>,
    pub volume: Option<i64>,
}

// ---------------------------------------------------------------------------
// Instruments
// ---------------------------------------------------------------------------

/// Instrument lookup response entry.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct InstrumentResponse {
    pub asset_type: Option<MarketType>,
    pub bond_factor: Option<String>,
    pub bond_instrument_info: Option<Bond>,
    pub bond_multiplier: Option<String>,
    pub bond_price: Option<Number>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub fundamental: Option<FundamentalInst>,
    pub instrument_info: Option<Instrument>,
    pub symbol: Option<String>,
    pub r#type: Option<MarketType>,
}

/// Instrument search result.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Instrument {
    pub asset_type: Option<MarketType>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub symbol: Option<String>,
    pub r#type: Option<MarketType>,
}

/// Bond instrument details.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Bond {
    pub asset_type: Option<MarketType>,
    pub bond_factor: Option<String>,
    pub bond_multiplier: Option<String>,
    pub bond_price: Option<Number>,
    pub cusip: Option<String>,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub symbol: Option<String>,
    pub r#type: Option<MarketType>,
}

/// Instrument with fundamental data.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct FundamentalInst {
    pub avg_10_days_volume: Option<i64>,
    pub avg_1_day_volume: Option<i64>,
    pub avg_3_month_volume: Option<i64>,
    pub beta: Option<Number>,
    pub book_value_per_share: Option<Number>,
    pub corpaction_date: Option<String>,
    pub current_ratio: Option<Number>,
    pub declaration_date: Option<String>,
    pub div_growth_rate_3_year: Option<Number>,
    pub dividend_amount: Option<Number>,
    pub dividend_date: Option<String>,
    pub dividend_freq: Option<i32>,
    pub dividend_pay_amount: Option<Number>,
    pub dividend_pay_date: Option<String>,
    pub dividend_yield: Option<Number>,
    pub dtn_volume: Option<i64>,
    pub eps: Option<Number>,
    pub eps_change: Option<Number>,
    #[serde(rename = "epsChangePercentTTM")]
    pub eps_change_percent_ttm: Option<Number>,
    pub eps_change_year: Option<Number>,
    #[serde(rename = "epsTTM")]
    pub eps_ttm: Option<Number>,
    pub fund_leverage_factor: Option<Number>,
    pub fund_strategy: Option<String>,
    #[serde(rename = "grossMarginMRQ")]
    pub gross_margin_mrq: Option<Number>,
    #[serde(rename = "grossMarginTTM")]
    pub gross_margin_ttm: Option<Number>,
    pub high_52: Option<Number>,
    pub interest_coverage: Option<Number>,
    pub low_52: Option<Number>,
    pub lt_debt_to_equity: Option<Number>,
    pub market_cap: Option<Number>,
    pub market_cap_float: Option<Number>,
    #[serde(rename = "netProfitMarginMRQ")]
    pub net_profit_margin_mrq: Option<Number>,
    #[serde(rename = "netProfitMarginTTM")]
    pub net_profit_margin_ttm: Option<Number>,
    pub next_dividend_date: Option<String>,
    pub next_dividend_pay_date: Option<String>,
    #[serde(rename = "operatingMarginMRQ")]
    pub operating_margin_mrq: Option<Number>,
    #[serde(rename = "operatingMarginTTM")]
    pub operating_margin_ttm: Option<Number>,
    pub pb_ratio: Option<Number>,
    pub pcf_ratio: Option<Number>,
    pub pe_ratio: Option<Number>,
    pub peg_ratio: Option<Number>,
    pub pr_ratio: Option<Number>,
    pub quick_ratio: Option<Number>,
    pub return_on_assets: Option<Number>,
    pub return_on_equity: Option<Number>,
    pub return_on_investment: Option<Number>,
    pub rev_change_in: Option<Number>,
    #[serde(rename = "revChangeTTM")]
    pub rev_change_ttm: Option<Number>,
    pub rev_change_year: Option<Number>,
    pub shares_outstanding: Option<Number>,
    pub short_int_day_to_cover: Option<Number>,
    pub short_int_to_float: Option<Number>,
    pub symbol: Option<String>,
    pub total_debt_to_capital: Option<Number>,
    pub total_debt_to_equity: Option<Number>,
    pub vol_10_day_avg: Option<Number>,
    pub vol_1_day_avg: Option<Number>,
    pub vol_3_month_avg: Option<Number>,
}

/// Wrapper for the instruments search response.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct InstrumentsResponse {
    pub instruments: Option<Vec<InstrumentResponse>>,
}

// ---------------------------------------------------------------------------
// Market hours
// ---------------------------------------------------------------------------

/// Market hours for a trading session.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Hours {
    pub category: Option<String>,
    pub date: Option<String>,
    pub exchange: Option<String>,
    pub is_open: Option<bool>,
    pub market_type: Option<MarketType>,
    pub product: Option<String>,
    pub product_name: Option<String>,
    pub session_hours: Option<HashMap<String, Vec<Interval>>>,
}

/// Start and end times for a market session interval.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Interval {
    pub end: Option<String>,
    pub start: Option<String>,
}

// ---------------------------------------------------------------------------
// Movers
// ---------------------------------------------------------------------------

/// Market mover/screener result entry.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct Screener {
    pub change: Option<Number>,
    pub description: Option<String>,
    pub direction: Option<Direction>,
    pub last: Option<Number>,
    pub symbol: Option<String>,
    pub total_volume: Option<i64>,
}

/// Wrapper for the movers/screener response.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct ScreenerResponse {
    pub screeners: Option<Vec<Screener>>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::n;

    /// Equity JSON via the untagged QuoteResponseObject enum.
    /// Verifies the `52WeekHigh` serde rename, camelCase mapping, and
    /// that nested quote/reference/regular structs populate correctly.
    #[test]
    fn deserialize_quote_response_equity() {
        let json = r#"{
            "assetMainType": "EQUITY",
            "fundamental": {
                "avg10DaysVolume": 1,
                "avg1YearVolume": 0,
                "divAmount": 1.1,
                "divFreq": 4,
                "divYield": 1.1,
                "eps": 0,
                "peRatio": 1.1
            },
            "quote": {
                "52WeekHigh": 169.0,
                "52WeekLow": 1.1,
                "askMICId": "MEMX",
                "askPrice": 168.41,
                "askSize": 400,
                "askTime": 1644854683672,
                "bidMICId": "IEGX",
                "bidPrice": 168.4,
                "bidSize": 400,
                "bidTime": 1644854683633,
                "closePrice": 177.57,
                "highPrice": 169.0,
                "lastMICId": "XADF",
                "lastPrice": 168.405,
                "lastSize": 200,
                "lowPrice": 167.09,
                "mark": 168.405,
                "markChange": -9.165,
                "markPercentChange": -5.161,
                "netChange": -9.165,
                "netPercentChange": -5.161,
                "openPrice": 167.37,
                "quoteTime": 1644854683672,
                "securityStatus": "Normal",
                "totalVolume": 22361159,
                "tradeTime": 1644854683408,
                "volatility": 0.0347
            },
            "quoteType": "NBBO",
            "realtime": true,
            "reference": {
                "cusip": "037833100",
                "description": "Apple Inc",
                "exchange": "Q",
                "exchangeName": "NASDAQ"
            },
            "regular": {
                "regularMarketLastPrice": 168.405,
                "regularMarketLastSize": 2,
                "regularMarketNetChange": -9.165,
                "regularMarketPercentChange": -5.161,
                "regularMarketTradeTime": 1644854683408
            },
            "ssid": 1973757747,
            "symbol": "AAPL"
        }"#;

        let result: QuoteResponseObject = serde_json::from_str(json).unwrap();
        match result {
            QuoteResponseObject::Equity(eq) => {
                assert_eq!(eq.symbol, Some("AAPL".to_string()));
                assert_eq!(eq.asset_main_type, Some(AssetMainType::Equity));
                assert_eq!(eq.ssid, Some(1973757747));
                assert_eq!(eq.realtime, Some(true));
                assert_eq!(eq.quote_type, Some(QuoteType::Nbbo));

                let quote = eq.quote.unwrap();
                assert_eq!(quote.week_high_52, Some(n(169.0)));
                assert_eq!(quote.week_low_52, Some(n(1.1)));
                assert_eq!(quote.ask_mic_id, Some("MEMX".to_string()));
                assert_eq!(quote.total_volume, Some(22361159));

                let reference = eq.reference.unwrap();
                assert_eq!(reference.cusip, Some("037833100".to_string()));
                assert_eq!(reference.description, Some("Apple Inc".to_string()));

                let regular = eq.regular.unwrap();
                assert_eq!(regular.regular_market_last_price, Some(n(168.405)));
                assert_eq!(regular.regular_market_last_size, Some(2));
            }
            other => panic!("expected Equity variant, got {other:?}"),
        }
    }

    /// Forex quote deserialized directly (verifying a second asset type family).
    /// Tests forex-specific fields like tick, tickAmount, and the 52WeekHigh rename.
    #[test]
    fn deserialize_forex_response() {
        let json = r#"{
            "assetMainType": "FOREX",
            "quote": {
                "52WeekHigh": 1.135,
                "52WeekLow": 1.1331,
                "askPrice": 1.13456,
                "askSize": 1000000,
                "bidPrice": 1.13434,
                "bidSize": 1000000,
                "closePrice": 1.13191,
                "highPrice": 1.135,
                "lastPrice": 1.13445,
                "lastSize": 0,
                "lowPrice": 1.1331,
                "mark": 1.13445,
                "netChange": 0.00254,
                "netPercentChange": 0.0,
                "openPrice": 1.13324,
                "quoteTime": 1637236739892,
                "securityStatus": "Unknown",
                "tick": 0.0,
                "tickAmount": 0.0,
                "totalVolume": 0,
                "tradeTime": 1637236739892
            },
            "realtime": true,
            "reference": {
                "description": "Euro/USDollar Spot",
                "exchange": "T",
                "exchangeName": "GFT",
                "isTradable": false,
                "marketMaker": "",
                "product": "",
                "tradingHours": ""
            },
            "ssid": 1,
            "symbol": "EUR/USD"
        }"#;

        let result: ForexResponse = serde_json::from_str(json).unwrap();
        assert_eq!(result.symbol, Some("EUR/USD".to_string()));
        assert_eq!(result.asset_main_type, Some(AssetMainType::Forex));

        let quote = result.quote.unwrap();
        assert_eq!(quote.week_high_52, Some(n(1.135)));
        assert_eq!(quote.tick, Some(n(0.0)));
        assert_eq!(quote.ask_size, Some(1000000));

        let reference = result.reference.unwrap();
        assert_eq!(reference.is_tradable, Some(false));
        assert_eq!(reference.exchange_name, Some("GFT".to_string()));
    }

    /// OptionChain with nested callExpDateMap (HashMap<String, HashMap<String, Vec<OptionContract>>>).
    /// Verifies numeric fields like daysToExpiration arrive as f64.
    #[test]
    fn deserialize_option_chain() {
        let json = r#"{
            "symbol": "AAPL",
            "status": "SUCCESS",
            "strategy": "SINGLE",
            "isDelayed": false,
            "isIndex": false,
            "daysToExpiration": 0.0,
            "interestRate": 5.275,
            "underlyingPrice": 170.5,
            "volatility": 29.0,
            "callExpDateMap": {
                "2024-01-19:30": {
                    "170.0": [
                        {
                            "symbol": "AAPL  240119C00170000",
                            "description": "AAPL Jan 19 2024 170 Call",
                            "putCall": "CALL",
                            "exchangeName": "OPR",
                            "bidPrice": 8.5,
                            "askPrice": 8.75,
                            "lastPrice": 8.6,
                            "bidSize": 10.0,
                            "askSize": 15.0,
                            "lastSize": 5.0,
                            "strikePrice": 170.0,
                            "daysToExpiration": 150.0,
                            "delta": 0.52,
                            "gamma": 0.02,
                            "theta": -0.05,
                            "vega": 0.35,
                            "rho": 0.12,
                            "openInterest": 5000.0,
                            "totalVolume": 1200.0,
                            "isInTheMoney": true,
                            "multiplier": 100.0,
                            "expirationDate": "2024-01-19",
                            "expirationType": "S",
                            "settlementType": "P"
                        }
                    ]
                }
            },
            "putExpDateMap": {}
        }"#;

        let chain: OptionChain = serde_json::from_str(json).unwrap();
        assert_eq!(chain.symbol, Some("AAPL".to_string()));
        assert_eq!(chain.status, Some("SUCCESS".to_string()));
        assert_eq!(chain.strategy, Some(OptionStrategy::Single));
        assert_eq!(chain.is_delayed, Some(false));
        assert_eq!(chain.interest_rate, Some(n(5.275)));
        assert_eq!(chain.underlying_price, Some(n(170.5)));

        let call_map = chain.call_exp_date_map.unwrap();
        let exp_date = call_map.get("2024-01-19:30").unwrap();
        let contracts = exp_date.get("170.0").unwrap();
        assert_eq!(contracts.len(), 1);

        let contract = &contracts[0];
        assert_eq!(contract.symbol, Some("AAPL  240119C00170000".to_string()));
        assert_eq!(contract.put_call, Some(PutCall::Call));
        assert_eq!(contract.strike_price, Some(n(170.0)));
        // Verify daysToExpiration arrives as f64
        assert_eq!(contract.days_to_expiration, Some(n(150.0)));
        assert_eq!(contract.delta, Some(n(0.52)));
        assert_eq!(contract.is_in_the_money, Some(true));
        // Numeric fields that spec declares as int but arrive as f64
        assert_eq!(contract.bid_size, Some(n(10.0)));
        assert_eq!(contract.open_interest, Some(n(5000.0)));
        assert_eq!(contract.multiplier, Some(n(100.0)));
    }

    /// CandleList with 3 candles, including the previousCloseDateISO8601 rename.
    #[test]
    fn deserialize_candle_list() {
        let json = r#"{
            "candles": [
                {
                    "close": 175.04,
                    "datetime": 1639137600000,
                    "datetimeISO8601": "2021-12-10T12:00:00Z",
                    "high": 175.15,
                    "low": 175.01,
                    "open": 175.01,
                    "volume": 10719
                },
                {
                    "close": 175.05,
                    "datetime": 1639137660000,
                    "high": 175.09,
                    "low": 175.05,
                    "open": 175.08,
                    "volume": 500
                },
                {
                    "close": 176.25,
                    "datetime": 1640307300000,
                    "high": 176.27,
                    "low": 176.22,
                    "open": 176.22,
                    "volume": 3395
                }
            ],
            "empty": false,
            "previousClose": 174.56,
            "previousCloseDate": 1639029600000,
            "previousCloseDateISO8601": "2021-12-09",
            "symbol": "AAPL"
        }"#;

        let candles: CandleList = serde_json::from_str(json).unwrap();
        assert_eq!(candles.symbol, Some("AAPL".to_string()));
        assert_eq!(candles.empty, Some(false));
        assert_eq!(candles.previous_close, Some(n(174.56)));
        assert_eq!(candles.previous_close_date, Some(1639029600000));
        assert_eq!(
            candles.previous_close_date_iso8601,
            Some("2021-12-09".to_string())
        );

        let list = candles.candles.unwrap();
        assert_eq!(list.len(), 3);

        assert_eq!(list[0].close, Some(n(175.04)));
        assert_eq!(list[0].datetime, Some(1639137600000));
        assert_eq!(
            list[0].datetime_iso8601,
            Some("2021-12-10T12:00:00Z".to_string())
        );
        assert_eq!(list[0].volume, Some(10719));

        assert_eq!(list[2].open, Some(n(176.22)));
        assert_eq!(list[2].high, Some(n(176.27)));
    }

    /// InstrumentsResponse wrapper with two instruments.
    #[test]
    fn deserialize_instruments_response() {
        let json = r#"{
            "instruments": [
                {
                    "assetType": "EQUITY",
                    "cusip": "037833100",
                    "description": "Apple Inc",
                    "exchange": "NASDAQ",
                    "symbol": "AAPL"
                },
                {
                    "assetType": "EQUITY",
                    "cusip": "060505104",
                    "description": "Bank Of America Corp",
                    "exchange": "NYSE",
                    "symbol": "BAC"
                }
            ]
        }"#;

        let resp: InstrumentsResponse = serde_json::from_str(json).unwrap();
        let instruments = resp.instruments.unwrap();
        assert_eq!(instruments.len(), 2);

        assert_eq!(instruments[0].symbol, Some("AAPL".to_string()));
        assert_eq!(instruments[0].asset_type, Some(MarketType::Equity));
        assert_eq!(instruments[0].cusip, Some("037833100".to_string()));
        assert_eq!(instruments[0].exchange, Some("NASDAQ".to_string()));

        assert_eq!(instruments[1].symbol, Some("BAC".to_string()));
        assert_eq!(
            instruments[1].description,
            Some("Bank Of America Corp".to_string())
        );
    }

    /// Hours with session_hours HashMap containing pre/regular/post market intervals.
    #[test]
    fn deserialize_hours() {
        let json = r#"{
            "category": "NULL",
            "date": "2022-04-14",
            "exchange": "NULL",
            "isOpen": true,
            "marketType": "EQUITY",
            "product": "EQ",
            "productName": "equity",
            "sessionHours": {
                "preMarket": [
                    {
                        "end": "2022-04-14T09:30:00-04:00",
                        "start": "2022-04-14T07:00:00-04:00"
                    }
                ],
                "regularMarket": [
                    {
                        "end": "2022-04-14T16:00:00-04:00",
                        "start": "2022-04-14T09:30:00-04:00"
                    }
                ],
                "postMarket": [
                    {
                        "end": "2022-04-14T20:00:00-04:00",
                        "start": "2022-04-14T16:00:00-04:00"
                    }
                ]
            }
        }"#;

        let hours: Hours = serde_json::from_str(json).unwrap();
        assert_eq!(hours.is_open, Some(true));
        assert_eq!(hours.market_type, Some(MarketType::Equity));
        assert_eq!(hours.product, Some("EQ".to_string()));
        assert_eq!(hours.product_name, Some("equity".to_string()));
        assert_eq!(hours.date, Some("2022-04-14".to_string()));

        let sessions = hours.session_hours.unwrap();
        assert_eq!(sessions.len(), 3);

        let regular = &sessions["regularMarket"];
        assert_eq!(regular.len(), 1);
        assert_eq!(
            regular[0].start,
            Some("2022-04-14T09:30:00-04:00".to_string())
        );
        assert_eq!(
            regular[0].end,
            Some("2022-04-14T16:00:00-04:00".to_string())
        );

        let pre = &sessions["preMarket"];
        assert_eq!(pre[0].start, Some("2022-04-14T07:00:00-04:00".to_string()));
    }

    /// ScreenerResponse wrapper with two movers.
    #[test]
    fn deserialize_screener_response() {
        let json = r#"{
            "screeners": [
                {
                    "change": 10.0,
                    "description": "Dow jones",
                    "direction": "up",
                    "last": 100.0,
                    "symbol": "$DJI",
                    "totalVolume": 100
                },
                {
                    "change": -5.2,
                    "description": "S&P 500",
                    "direction": "down",
                    "last": 4400.0,
                    "symbol": "$SPX",
                    "totalVolume": 628009977
                }
            ]
        }"#;

        let resp: ScreenerResponse = serde_json::from_str(json).unwrap();
        let screeners = resp.screeners.unwrap();
        assert_eq!(screeners.len(), 2);

        assert_eq!(screeners[0].symbol, Some("$DJI".to_string()));
        assert_eq!(screeners[0].direction, Some(Direction::Up));
        assert_eq!(screeners[0].change, Some(n(10.0)));
        assert_eq!(screeners[0].last, Some(n(100.0)));

        assert_eq!(screeners[1].symbol, Some("$SPX".to_string()));
        assert_eq!(screeners[1].direction, Some(Direction::Down));
        assert_eq!(screeners[1].total_volume, Some(628009977));
    }

    /// ExpirationChain with multiple expiration entries.
    #[test]
    fn deserialize_expiration_chain() {
        let json = r#"{
            "expirationList": [
                {
                    "daysToExpiration": 2,
                    "expiration": "2022-01-07",
                    "expirationType": "W",
                    "optionRoots": "AAPL",
                    "settlementType": "P",
                    "standard": true
                },
                {
                    "daysToExpiration": 16,
                    "expiration": "2022-01-21",
                    "expirationType": "S",
                    "optionRoots": "AAPL",
                    "settlementType": "P",
                    "standard": true
                },
                {
                    "daysToExpiration": 380,
                    "expiration": "2023-01-20",
                    "expirationType": "S",
                    "optionRoots": "AAPL",
                    "settlementType": "A",
                    "standard": false
                }
            ],
            "status": "SUCCESS"
        }"#;

        let chain: ExpirationChain = serde_json::from_str(json).unwrap();
        assert_eq!(chain.status, Some("SUCCESS".to_string()));

        let list = chain.expiration_list.unwrap();
        assert_eq!(list.len(), 3);

        assert_eq!(list[0].days_to_expiration, Some(2));
        assert_eq!(list[0].expiration, Some("2022-01-07".to_string()));
        assert_eq!(list[0].expiration_type, Some(ExpirationType::Weekly));
        assert_eq!(list[0].settlement_type, Some(SettlementType::Pm));
        assert_eq!(list[0].standard, Some(true));

        assert_eq!(list[2].days_to_expiration, Some(380));
        assert_eq!(list[2].expiration_type, Some(ExpirationType::Standard));
        assert_eq!(list[2].settlement_type, Some(SettlementType::Am));
        assert_eq!(list[2].standard, Some(false));
    }

    /// QuoteError with invalid symbols and SSIDs.
    #[test]
    fn deserialize_quote_error() {
        let json = r#"{
            "invalidSymbols": ["FAKESYM", "NOTREAL"],
            "invalidSSIDs": [12345, 67890],
            "invalidCusips": ["000000000"]
        }"#;

        let err: QuoteError = serde_json::from_str(json).unwrap();
        assert_eq!(
            err.invalid_symbols,
            Some(vec!["FAKESYM".to_string(), "NOTREAL".to_string()])
        );
        assert_eq!(err.invalid_ssids, Some(vec![12345, 67890]));
        assert_eq!(err.invalid_cusips, Some(vec!["000000000".to_string()]));
    }

    /// Equity response with null/missing optional fields.
    #[test]
    fn deserialize_equity_with_null_fields() {
        let json = r#"{
            "assetMainType": "EQUITY",
            "quote": {
                "52WeekHigh": 169.0,
                "52WeekLow": null,
                "lastPrice": 168.405,
                "totalVolume": 22361159,
                "securityStatus": "Normal"
            },
            "realtime": true,
            "symbol": "AAPL"
        }"#;

        let result: EquityResponse = serde_json::from_str(json).unwrap();
        assert_eq!(result.symbol, Some("AAPL".to_string()));
        // Fields not present in JSON are None
        assert_eq!(result.fundamental, None);
        assert_eq!(result.extended, None);
        assert_eq!(result.regular, None);
        assert_eq!(result.quote_type, None);
        assert_eq!(result.ssid, None);

        let quote = result.quote.unwrap();
        assert_eq!(quote.week_high_52, Some(n(169.0)));
        // Explicitly null field
        assert_eq!(quote.week_low_52, None);
        assert_eq!(quote.last_price, Some(n(168.405)));
        // Fields not present in the quote JSON
        assert_eq!(quote.ask_price, None);
        assert_eq!(quote.volatility, None);
    }

    /// Full QuoteResponse (HashMap<String, QuoteResponseObject>) with multiple symbols.
    #[test]
    fn deserialize_quote_response_map() {
        let json = r#"{
            "AAPL": {
                "assetMainType": "EQUITY",
                "quote": {
                    "52WeekHigh": 169.0,
                    "lastPrice": 168.405
                },
                "realtime": true,
                "symbol": "AAPL"
            },
            "$DJI": {
                "assetMainType": "INDEX",
                "quote": {
                    "52WeekHigh": 34744.56,
                    "lastPrice": 34436.13
                },
                "realtime": true,
                "symbol": "$DJI"
            }
        }"#;

        let resp: QuoteResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.len(), 2);
        assert!(resp.contains_key("AAPL"));
        assert!(resp.contains_key("$DJI"));

        // Both entries deserialize as Equity due to untagged enum with all-optional fields.
        // The assetMainType field value distinguishes them logically.
        match &resp["AAPL"] {
            QuoteResponseObject::Equity(eq) => {
                assert_eq!(eq.asset_main_type, Some(AssetMainType::Equity));
            }
            other => panic!("expected Equity variant for AAPL, got {other:?}"),
        }
    }
}
