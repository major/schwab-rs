//! Level-one equity streaming data types.

use super::super::Number;

/// Field selector for level-one equity streaming subscriptions.
///
/// Each variant corresponds to a numeric field index in the Schwab streaming protocol.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EquityField {
    Symbol = 0,
    BidPrice = 1,
    AskPrice = 2,
    LastPrice = 3,
    BidSize = 4,
    AskSize = 5,
    AskExchangeId = 6,
    BidExchangeId = 7,
    TotalVolume = 8,
    LastSize = 9,
    HighPrice = 10,
    LowPrice = 11,
    ClosePrice = 12,
    ExchangeId = 13,
    Marginable = 14,
    Description = 15,
    LastExchangeId = 16,
    OpenPrice = 17,
    NetChange = 18,
    High52Week = 19,
    Low52Week = 20,
    PeRatio = 21,
    AnnualDividendAmount = 22,
    DividendYield = 23,
    Nav = 24,
    ExchangeName = 25,
    DividendDate = 26,
    RegularMarketQuote = 27,
    RegularMarketTrade = 28,
    RegularMarketLastPrice = 29,
    RegularMarketLastSize = 30,
    RegularMarketNetChange = 31,
    SecurityStatus = 32,
    MarkPrice = 33,
    QuoteTime = 34,
    TradeTime = 35,
    RegularMarketTradeTime = 36,
    BidTime = 37,
    AskTime = 38,
    AskMicId = 39,
    BidMicId = 40,
    LastMicId = 41,
    NetPercentChange = 42,
    RegularMarketPercentChange = 43,
    MarkPriceNetChange = 44,
    MarkPricePercentChange = 45,
    HardToBorrowQuantity = 46,
    HardToBorrowRate = 47,
    HardToBorrow = 48,
    Shortable = 49,
    PostMarketNetChange = 50,
    PostMarketPercentChange = 51,
}

impl EquityField {
    /// Return the numeric field index used in the Schwab streaming protocol.
    pub fn index(&self) -> u32 {
        *self as u32
    }

    /// Return all `EquityField` variants in index order.
    pub fn all() -> &'static [EquityField] {
        use EquityField::*;
        &[
            Symbol,
            BidPrice,
            AskPrice,
            LastPrice,
            BidSize,
            AskSize,
            AskExchangeId,
            BidExchangeId,
            TotalVolume,
            LastSize,
            HighPrice,
            LowPrice,
            ClosePrice,
            ExchangeId,
            Marginable,
            Description,
            LastExchangeId,
            OpenPrice,
            NetChange,
            High52Week,
            Low52Week,
            PeRatio,
            AnnualDividendAmount,
            DividendYield,
            Nav,
            ExchangeName,
            DividendDate,
            RegularMarketQuote,
            RegularMarketTrade,
            RegularMarketLastPrice,
            RegularMarketLastSize,
            RegularMarketNetChange,
            SecurityStatus,
            MarkPrice,
            QuoteTime,
            TradeTime,
            RegularMarketTradeTime,
            BidTime,
            AskTime,
            AskMicId,
            BidMicId,
            LastMicId,
            NetPercentChange,
            RegularMarketPercentChange,
            MarkPriceNetChange,
            MarkPricePercentChange,
            HardToBorrowQuantity,
            HardToBorrowRate,
            HardToBorrow,
            Shortable,
            PostMarketNetChange,
            PostMarketPercentChange,
        ]
    }
}

/// Level-one equity streaming data for a single symbol.
///
/// All fields are `Option<T>` because the Schwab API sends only subscribed fields.
/// Named metadata fields use string keys; numeric data fields use numeric string keys.
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelOneEquity {
    // Named metadata fields (string-keyed in the protocol)
    pub key: Option<String>,
    pub delayed: Option<bool>,
    pub asset_main_type: Option<String>,
    pub asset_sub_type: Option<String>,
    pub cusip: Option<String>,
    // Numeric data fields (index-keyed: "0", "1", ...)
    pub symbol: Option<String>,
    pub bid_price: Option<Number>,
    pub ask_price: Option<Number>,
    pub last_price: Option<Number>,
    pub bid_size: Option<i64>,
    pub ask_size: Option<i64>,
    pub ask_exchange_id: Option<String>,
    pub bid_exchange_id: Option<String>,
    pub total_volume: Option<i64>,
    pub last_size: Option<i64>,
    pub high_price: Option<Number>,
    pub low_price: Option<Number>,
    pub close_price: Option<Number>,
    pub exchange_id: Option<String>,
    pub marginable: Option<bool>,
    pub description: Option<String>,
    pub last_exchange_id: Option<String>,
    pub open_price: Option<Number>,
    pub net_change: Option<Number>,
    pub high_52_week: Option<Number>,
    pub low_52_week: Option<Number>,
    pub pe_ratio: Option<Number>,
    pub annual_dividend_amount: Option<Number>,
    pub dividend_yield: Option<Number>,
    pub nav: Option<Number>,
    pub exchange_name: Option<String>,
    pub dividend_date: Option<String>,
    pub regular_market_quote: Option<bool>,
    pub regular_market_trade: Option<bool>,
    pub regular_market_last_price: Option<Number>,
    pub regular_market_last_size: Option<i64>,
    pub regular_market_net_change: Option<Number>,
    pub security_status: Option<String>,
    pub mark_price: Option<Number>,
    pub quote_time: Option<i64>,
    pub trade_time: Option<i64>,
    pub regular_market_trade_time: Option<i64>,
    pub bid_time: Option<i64>,
    pub ask_time: Option<i64>,
    pub ask_mic_id: Option<String>,
    pub bid_mic_id: Option<String>,
    pub last_mic_id: Option<String>,
    pub net_percent_change: Option<Number>,
    pub regular_market_percent_change: Option<Number>,
    pub mark_price_net_change: Option<Number>,
    pub mark_price_percent_change: Option<Number>,
    pub hard_to_borrow_quantity: Option<i64>,
    pub hard_to_borrow_rate: Option<Number>,
    pub hard_to_borrow: Option<i64>,
    pub shortable: Option<i64>,
    pub post_market_net_change: Option<Number>,
    pub post_market_percent_change: Option<Number>,
}

/// Parse a [`Number`] from a [`serde_json::Value`].
///
/// Works for both `f64` (default) and `rust_decimal::Decimal` (`decimal` feature).
fn parse_num(v: &serde_json::Value) -> Option<Number> {
    serde_json::from_value::<Number>(v.clone()).ok()
}

impl LevelOneEquity {
    /// Construct a [`LevelOneEquity`] from a streaming data map entry.
    ///
    /// The map uses named string keys for metadata (`"key"`, `"delayed"`) and
    /// numeric string keys (`"0"`, `"1"`, ...) for field data.
    /// Returns `None` if `value` is not a JSON object.
    pub(crate) fn from_value(value: &serde_json::Value) -> Option<Self> {
        let map = value.as_object()?;
        let mut s = Self::default();

        // Named metadata fields
        s.key = map.get("key").and_then(|v| v.as_str()).map(String::from);
        s.delayed = map.get("delayed").and_then(|v| v.as_bool());
        s.asset_main_type = map
            .get("assetMainType")
            .and_then(|v| v.as_str())
            .map(String::from);
        s.asset_sub_type = map
            .get("assetSubType")
            .and_then(|v| v.as_str())
            .map(String::from);
        s.cusip = map.get("cusip").and_then(|v| v.as_str()).map(String::from);

        // Numeric-keyed data fields
        for (key, val) in map {
            match key.as_str() {
                "0" => s.symbol = val.as_str().map(String::from),
                "1" => s.bid_price = parse_num(val),
                "2" => s.ask_price = parse_num(val),
                "3" => s.last_price = parse_num(val),
                "4" => s.bid_size = val.as_i64(),
                "5" => s.ask_size = val.as_i64(),
                "6" => s.ask_exchange_id = val.as_str().map(String::from),
                "7" => s.bid_exchange_id = val.as_str().map(String::from),
                "8" => s.total_volume = val.as_i64(),
                "9" => s.last_size = val.as_i64(),
                "10" => s.high_price = parse_num(val),
                "11" => s.low_price = parse_num(val),
                "12" => s.close_price = parse_num(val),
                "13" => s.exchange_id = val.as_str().map(String::from),
                "14" => s.marginable = val.as_bool(),
                "15" => s.description = val.as_str().map(String::from),
                "16" => s.last_exchange_id = val.as_str().map(String::from),
                "17" => s.open_price = parse_num(val),
                "18" => s.net_change = parse_num(val),
                "19" => s.high_52_week = parse_num(val),
                "20" => s.low_52_week = parse_num(val),
                "21" => s.pe_ratio = parse_num(val),
                "22" => s.annual_dividend_amount = parse_num(val),
                "23" => s.dividend_yield = parse_num(val),
                "24" => s.nav = parse_num(val),
                "25" => s.exchange_name = val.as_str().map(String::from),
                "26" => s.dividend_date = val.as_str().map(String::from),
                "27" => s.regular_market_quote = val.as_bool(),
                "28" => s.regular_market_trade = val.as_bool(),
                "29" => s.regular_market_last_price = parse_num(val),
                "30" => s.regular_market_last_size = val.as_i64(),
                "31" => s.regular_market_net_change = parse_num(val),
                "32" => s.security_status = val.as_str().map(String::from),
                "33" => s.mark_price = parse_num(val),
                "34" => s.quote_time = val.as_i64(),
                "35" => s.trade_time = val.as_i64(),
                "36" => s.regular_market_trade_time = val.as_i64(),
                "37" => s.bid_time = val.as_i64(),
                "38" => s.ask_time = val.as_i64(),
                "39" => s.ask_mic_id = val.as_str().map(String::from),
                "40" => s.bid_mic_id = val.as_str().map(String::from),
                "41" => s.last_mic_id = val.as_str().map(String::from),
                "42" => s.net_percent_change = parse_num(val),
                "43" => s.regular_market_percent_change = parse_num(val),
                "44" => s.mark_price_net_change = parse_num(val),
                "45" => s.mark_price_percent_change = parse_num(val),
                "46" => s.hard_to_borrow_quantity = val.as_i64(),
                "47" => s.hard_to_borrow_rate = parse_num(val),
                "48" => s.hard_to_borrow = val.as_i64(),
                "49" => s.shortable = val.as_i64(),
                "50" => s.post_market_net_change = parse_num(val),
                "51" => s.post_market_percent_change = parse_num(val),
                _ => {}
            }
        }

        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn field_index_first() {
        assert_eq!(EquityField::Symbol.index(), 0);
    }

    #[test]
    fn field_index_last() {
        assert_eq!(EquityField::PostMarketPercentChange.index(), 51);
    }

    #[test]
    fn all_fields_count() {
        assert_eq!(EquityField::all().len(), 52);
    }

    #[test]
    fn all_fields_sequential_indices() {
        for (i, field) in EquityField::all().iter().enumerate() {
            assert_eq!(
                field.index() as usize,
                i,
                "field at position {i} has wrong index"
            );
        }
    }

    #[test]
    fn from_value_parses_sample() {
        let input = json!({
            "key": "AAPL",
            "delayed": false,
            "1": 150.25,
            "5": 100,
            "15": "Apple Inc."
        });

        let equity = LevelOneEquity::from_value(&input).expect("should parse JSON object");

        assert_eq!(equity.key, Some("AAPL".to_string()));
        assert_eq!(equity.delayed, Some(false));
        assert_eq!(equity.bid_price, Some("150.25".parse().unwrap()));
        assert_eq!(equity.ask_size, Some(100));
        assert_eq!(equity.description, Some("Apple Inc.".to_string()));
        // Fields not in sample remain None
        assert_eq!(equity.symbol, None);
        assert_eq!(equity.last_price, None);
    }

    #[test]
    fn from_value_returns_none_for_non_object() {
        assert!(LevelOneEquity::from_value(&json!(42)).is_none());
        assert!(LevelOneEquity::from_value(&json!("text")).is_none());
        assert!(LevelOneEquity::from_value(&json!(null)).is_none());
    }
}
