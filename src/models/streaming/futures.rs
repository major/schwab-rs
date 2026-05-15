use super::super::Number;

/// Parse a JSON value as a [`Number`].
fn parse_num(v: &serde_json::Value) -> Option<Number> {
    serde_json::from_value::<Number>(v.clone()).ok()
}

/// Field indices for level-one futures streaming data.
///
/// Each variant maps to a numeric key in the Schwab streaming JSON protocol.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum FuturesField {
    Symbol = 0,
    BidPrice = 1,
    AskPrice = 2,
    LastPrice = 3,
    BidSize = 4,
    AskSize = 5,
    BidId = 6,
    AskId = 7,
    TotalVolume = 8,
    LastSize = 9,
    QuoteTime = 10,
    TradeTime = 11,
    HighPrice = 12,
    LowPrice = 13,
    ClosePrice = 14,
    ExchangeId = 15,
    Description = 16,
    LastId = 17,
    OpenPrice = 18,
    NetChange = 19,
    FuturePercentChange = 20,
    ExchangeName = 21,
    SecurityStatus = 22,
    OpenInterest = 23,
    Mark = 24,
    Tick = 25,
    TickAmount = 26,
    Product = 27,
    FuturePriceFormat = 28,
    FutureTradingHours = 29,
    FutureIsTradable = 30,
    FutureMultiplier = 31,
    FutureIsActive = 32,
    FutureSettlementPrice = 33,
    FutureActiveSymbol = 34,
    FutureExpirationDate = 35,
    ExpirationStyle = 36,
    AskTime = 37,
    BidTime = 38,
    QuotedInSession = 39,
    SettlementDate = 40,
}

impl FuturesField {
    /// Return the numeric index used by the streaming protocol.
    pub fn index(&self) -> u32 {
        *self as u32
    }

    /// Return a slice of all field variants in index order.
    pub fn all() -> &'static [FuturesField] {
        &[
            Self::Symbol,
            Self::BidPrice,
            Self::AskPrice,
            Self::LastPrice,
            Self::BidSize,
            Self::AskSize,
            Self::BidId,
            Self::AskId,
            Self::TotalVolume,
            Self::LastSize,
            Self::QuoteTime,
            Self::TradeTime,
            Self::HighPrice,
            Self::LowPrice,
            Self::ClosePrice,
            Self::ExchangeId,
            Self::Description,
            Self::LastId,
            Self::OpenPrice,
            Self::NetChange,
            Self::FuturePercentChange,
            Self::ExchangeName,
            Self::SecurityStatus,
            Self::OpenInterest,
            Self::Mark,
            Self::Tick,
            Self::TickAmount,
            Self::Product,
            Self::FuturePriceFormat,
            Self::FutureTradingHours,
            Self::FutureIsTradable,
            Self::FutureMultiplier,
            Self::FutureIsActive,
            Self::FutureSettlementPrice,
            Self::FutureActiveSymbol,
            Self::FutureExpirationDate,
            Self::ExpirationStyle,
            Self::AskTime,
            Self::BidTime,
            Self::QuotedInSession,
            Self::SettlementDate,
        ]
    }
}

/// Level-one futures data from the Schwab streaming WebSocket API.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(missing_docs)]
pub struct LevelOneFutures {
    // Metadata (string-keyed)
    pub key: Option<String>,
    pub delayed: Option<bool>,
    pub asset_main_type: Option<String>,
    pub asset_sub_type: Option<String>,
    pub cusip: Option<String>,

    // String fields
    pub symbol: Option<String>,
    pub bid_id: Option<String>,
    pub ask_id: Option<String>,
    pub exchange_id: Option<String>,
    pub description: Option<String>,
    pub last_id: Option<String>,
    pub exchange_name: Option<String>,
    pub security_status: Option<String>,
    pub product: Option<String>,
    pub future_price_format: Option<String>,
    pub future_trading_hours: Option<String>,
    pub future_active_symbol: Option<String>,
    pub expiration_style: Option<String>,

    // Bool fields
    pub future_is_tradable: Option<bool>,
    pub future_is_active: Option<bool>,
    pub quoted_in_session: Option<bool>,

    // i64 fields
    pub bid_size: Option<i64>,
    pub ask_size: Option<i64>,
    pub total_volume: Option<i64>,
    pub last_size: Option<i64>,
    pub quote_time: Option<i64>,
    pub trade_time: Option<i64>,
    pub open_interest: Option<i64>,
    pub ask_time: Option<i64>,
    pub bid_time: Option<i64>,

    // Number fields
    pub bid_price: Option<Number>,
    pub ask_price: Option<Number>,
    pub last_price: Option<Number>,
    pub high_price: Option<Number>,
    pub low_price: Option<Number>,
    pub close_price: Option<Number>,
    pub open_price: Option<Number>,
    pub net_change: Option<Number>,
    pub future_percent_change: Option<Number>,
    pub mark: Option<Number>,
    pub tick: Option<Number>,
    pub tick_amount: Option<Number>,
    pub future_multiplier: Option<Number>,
    pub future_settlement_price: Option<Number>,
    pub future_expiration_date: Option<Number>,
    pub settlement_date: Option<Number>,
}

impl LevelOneFutures {
    /// Parse a streaming JSON object into a [`LevelOneFutures`].
    ///
    /// Returns `None` when `value` is not a JSON object.
    pub(crate) fn from_value(value: &serde_json::Value) -> Option<Self> {
        let obj = value.as_object()?;
        let mut result = Self::default();

        for (key, val) in obj {
            match key.as_str() {
                // Metadata fields (string-keyed)
                "key" => result.key = val.as_str().map(String::from),
                "delayed" => result.delayed = val.as_bool(),
                "assetMainType" => result.asset_main_type = val.as_str().map(String::from),
                "assetSubType" => result.asset_sub_type = val.as_str().map(String::from),
                "cusip" => result.cusip = val.as_str().map(String::from),

                // Numeric-keyed data fields
                numeric => {
                    if let Ok(idx) = numeric.parse::<u32>() {
                        match idx {
                            // String fields
                            0 => result.symbol = val.as_str().map(String::from),
                            6 => result.bid_id = val.as_str().map(String::from),
                            7 => result.ask_id = val.as_str().map(String::from),
                            15 => result.exchange_id = val.as_str().map(String::from),
                            16 => result.description = val.as_str().map(String::from),
                            17 => result.last_id = val.as_str().map(String::from),
                            21 => result.exchange_name = val.as_str().map(String::from),
                            22 => result.security_status = val.as_str().map(String::from),
                            27 => result.product = val.as_str().map(String::from),
                            28 => result.future_price_format = val.as_str().map(String::from),
                            29 => result.future_trading_hours = val.as_str().map(String::from),
                            34 => result.future_active_symbol = val.as_str().map(String::from),
                            36 => result.expiration_style = val.as_str().map(String::from),

                            // Bool fields
                            30 => result.future_is_tradable = val.as_bool(),
                            32 => result.future_is_active = val.as_bool(),
                            39 => result.quoted_in_session = val.as_bool(),

                            // i64 fields
                            4 => result.bid_size = val.as_i64(),
                            5 => result.ask_size = val.as_i64(),
                            8 => result.total_volume = val.as_i64(),
                            9 => result.last_size = val.as_i64(),
                            10 => result.quote_time = val.as_i64(),
                            11 => result.trade_time = val.as_i64(),
                            23 => result.open_interest = val.as_i64(),
                            37 => result.ask_time = val.as_i64(),
                            38 => result.bid_time = val.as_i64(),

                            // Number fields
                            1 => result.bid_price = parse_num(val),
                            2 => result.ask_price = parse_num(val),
                            3 => result.last_price = parse_num(val),
                            12 => result.high_price = parse_num(val),
                            13 => result.low_price = parse_num(val),
                            14 => result.close_price = parse_num(val),
                            18 => result.open_price = parse_num(val),
                            19 => result.net_change = parse_num(val),
                            20 => result.future_percent_change = parse_num(val),
                            24 => result.mark = parse_num(val),
                            25 => result.tick = parse_num(val),
                            26 => result.tick_amount = parse_num(val),
                            31 => result.future_multiplier = parse_num(val),
                            33 => result.future_settlement_price = parse_num(val),
                            35 => result.future_expiration_date = parse_num(val),
                            40 => result.settlement_date = parse_num(val),

                            _ => {}
                        }
                    }
                }
            }
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::n;

    #[test]
    fn field_count() {
        assert_eq!(FuturesField::all().len(), 41);
    }

    #[test]
    fn field_indices() {
        for (i, field) in FuturesField::all().iter().enumerate() {
            assert_eq!(
                field.index(),
                i as u32,
                "field {field:?} should have index {i}"
            );
        }
    }

    #[test]
    fn from_value_parses_futures() {
        let json = serde_json::json!({
            "key": "/ESM25",
            "delayed": false,
            "1": 5500.25,
            "16": "E-mini S&P 500"
        });

        let fut = LevelOneFutures::from_value(&json).expect("should parse");
        assert_eq!(fut.key, Some("/ESM25".to_string()));
        assert_eq!(fut.delayed, Some(false));
        assert_eq!(fut.bid_price, Some(n(5500.25)));
        assert_eq!(fut.description, Some("E-mini S&P 500".to_string()));
        // Unpopulated fields remain None
        assert_eq!(fut.symbol, None);
        assert_eq!(fut.ask_price, None);
    }

    #[test]
    fn from_value_returns_none_for_non_object() {
        assert!(LevelOneFutures::from_value(&serde_json::json!(42)).is_none());
        assert!(LevelOneFutures::from_value(&serde_json::json!("string")).is_none());
    }
}
