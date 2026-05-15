//! Level-one forex streaming data types.

use super::super::Number;

/// Field selector for level-one forex streaming subscriptions.
///
/// Each variant corresponds to a numeric field index in the Schwab streaming protocol.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ForexField {
    Symbol = 0,
    BidPrice = 1,
    AskPrice = 2,
    LastPrice = 3,
    BidSize = 4,
    AskSize = 5,
    TotalVolume = 6,
    LastSize = 7,
    QuoteTime = 8,
    TradeTime = 9,
    HighPrice = 10,
    LowPrice = 11,
    ClosePrice = 12,
    ExchangeId = 13,
    Description = 14,
    OpenPrice = 15,
    NetChange = 16,
    PercentChange = 17,
    ExchangeName = 18,
    Digits = 19,
    SecurityStatus = 20,
    Tick = 21,
    TickAmount = 22,
    Product = 23,
    TradingHours = 24,
    IsTradable = 25,
    MarketMaker = 26,
    High52Week = 27,
    Low52Week = 28,
    Mark = 29,
}

impl ForexField {
    /// Return the numeric field index used in the Schwab streaming protocol.
    pub fn index(&self) -> u32 {
        *self as u32
    }

    /// Return all `ForexField` variants in index order.
    pub fn all() -> &'static [ForexField] {
        use ForexField::*;
        &[
            Symbol,
            BidPrice,
            AskPrice,
            LastPrice,
            BidSize,
            AskSize,
            TotalVolume,
            LastSize,
            QuoteTime,
            TradeTime,
            HighPrice,
            LowPrice,
            ClosePrice,
            ExchangeId,
            Description,
            OpenPrice,
            NetChange,
            PercentChange,
            ExchangeName,
            Digits,
            SecurityStatus,
            Tick,
            TickAmount,
            Product,
            TradingHours,
            IsTradable,
            MarketMaker,
            High52Week,
            Low52Week,
            Mark,
        ]
    }
}

/// Level-one forex streaming data for a single currency pair.
///
/// All fields are `Option<T>` because the Schwab API sends only subscribed fields.
/// Named metadata fields use string keys; numeric data fields use numeric string keys.
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelOneForex {
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
    pub total_volume: Option<i64>,
    pub last_size: Option<i64>,
    pub quote_time: Option<i64>,
    pub trade_time: Option<i64>,
    pub high_price: Option<Number>,
    pub low_price: Option<Number>,
    pub close_price: Option<Number>,
    pub exchange_id: Option<String>,
    pub description: Option<String>,
    pub open_price: Option<Number>,
    pub net_change: Option<Number>,
    pub percent_change: Option<Number>,
    pub exchange_name: Option<String>,
    pub digits: Option<i64>,
    pub security_status: Option<String>,
    pub tick: Option<Number>,
    pub tick_amount: Option<Number>,
    pub product: Option<String>,
    pub trading_hours: Option<String>,
    pub is_tradable: Option<bool>,
    pub market_maker: Option<String>,
    pub high_52_week: Option<Number>,
    pub low_52_week: Option<Number>,
    pub mark: Option<Number>,
}

/// Parse a [`Number`] from a [`serde_json::Value`].
///
/// Works for both `f64` (default) and `rust_decimal::Decimal` (`decimal` feature).
fn parse_num(v: &serde_json::Value) -> Option<Number> {
    serde_json::from_value::<Number>(v.clone()).ok()
}

impl LevelOneForex {
    /// Construct a [`LevelOneForex`] from a streaming data map entry.
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
                "6" => s.total_volume = val.as_i64(),
                "7" => s.last_size = val.as_i64(),
                "8" => s.quote_time = val.as_i64(),
                "9" => s.trade_time = val.as_i64(),
                "10" => s.high_price = parse_num(val),
                "11" => s.low_price = parse_num(val),
                "12" => s.close_price = parse_num(val),
                "13" => s.exchange_id = val.as_str().map(String::from),
                "14" => s.description = val.as_str().map(String::from),
                "15" => s.open_price = parse_num(val),
                "16" => s.net_change = parse_num(val),
                "17" => s.percent_change = parse_num(val),
                "18" => s.exchange_name = val.as_str().map(String::from),
                "19" => s.digits = val.as_i64(),
                "20" => s.security_status = val.as_str().map(String::from),
                "21" => s.tick = parse_num(val),
                "22" => s.tick_amount = parse_num(val),
                "23" => s.product = val.as_str().map(String::from),
                "24" => s.trading_hours = val.as_str().map(String::from),
                "25" => s.is_tradable = val.as_bool(),
                "26" => s.market_maker = val.as_str().map(String::from),
                "27" => s.high_52_week = parse_num(val),
                "28" => s.low_52_week = parse_num(val),
                "29" => s.mark = parse_num(val),
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
        assert_eq!(ForexField::Symbol.index(), 0);
    }

    #[test]
    fn field_index_last() {
        assert_eq!(ForexField::Mark.index(), 29);
    }

    #[test]
    fn all_fields_count() {
        assert_eq!(ForexField::all().len(), 30);
    }

    #[test]
    fn all_fields_sequential_indices() {
        for (i, field) in ForexField::all().iter().enumerate() {
            assert_eq!(field.index() as usize, i, "field at position {i} has wrong index");
        }
    }

    #[test]
    fn from_value_parses_sample() {
        let input = json!({
            "key": "EUR/USD",
            "delayed": false,
            "1": 1.0850,
            "14": "Euro/US Dollar"
        });

        let forex = LevelOneForex::from_value(&input).expect("should parse JSON object");

        assert_eq!(forex.key, Some("EUR/USD".to_string()));
        assert_eq!(forex.delayed, Some(false));
        assert_eq!(forex.bid_price, Some("1.085".parse().unwrap()));
        assert_eq!(forex.description, Some("Euro/US Dollar".to_string()));
        // Fields not in sample remain None
        assert_eq!(forex.symbol, None);
        assert_eq!(forex.last_price, None);
    }

    #[test]
    fn from_value_returns_none_for_non_object() {
        assert!(LevelOneForex::from_value(&json!(42)).is_none());
        assert!(LevelOneForex::from_value(&json!("text")).is_none());
        assert!(LevelOneForex::from_value(&json!(null)).is_none());
    }
}
