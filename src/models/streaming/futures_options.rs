use super::super::Number;

/// Field identifiers for level-one futures option streaming data.
///
/// Each variant maps to a numeric index used by the Schwab streaming API.
///
/// The enum has 32 variants with sequential indices starting at 0.
/// Use [`FuturesOptionField::all()`] to get a slice of every variant and
/// [`FuturesOptionField::index()`] to retrieve the numeric index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum FuturesOptionField {
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
    ExpirationDate = 28,
    ExpirationStyle = 29,
    StrikePrice = 30,
    ContractType = 31,
}

impl FuturesOptionField {
    /// Return the numeric index for this field.
    pub fn index(&self) -> u32 {
        *self as u32
    }

    /// Return a slice of all field variants.
    pub fn all() -> &'static [FuturesOptionField] {
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
            Self::ExpirationDate,
            Self::ExpirationStyle,
            Self::StrikePrice,
            Self::ContractType,
        ]
    }
}

/// Parse a JSON value as a [`Number`].
fn parse_num(v: &serde_json::Value) -> Option<Number> {
    serde_json::from_value::<Number>(v.clone()).ok()
}

/// Level-one futures option data from the Schwab streaming API.
///
/// Built from index-keyed JSON via the crate-internal `from_value` parser
/// rather than serde `Deserialize`, because the streaming API uses numeric
/// string keys (e.g. `"1"`, `"2"`) instead of named fields.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(missing_docs)]
pub struct LevelOneFuturesOption {
    // Metadata (string-keyed)
    pub key: Option<String>,
    pub delayed: Option<bool>,
    pub asset_main_type: Option<String>,
    pub asset_sub_type: Option<String>,
    pub cusip: Option<String>,

    // Data fields (index-keyed)
    pub symbol: Option<String>,
    pub bid_price: Option<Number>,
    pub ask_price: Option<Number>,
    pub last_price: Option<Number>,
    pub bid_size: Option<i64>,
    pub ask_size: Option<i64>,
    pub bid_id: Option<String>,
    pub ask_id: Option<String>,
    pub total_volume: Option<i64>,
    pub last_size: Option<i64>,
    pub quote_time: Option<i64>,
    pub trade_time: Option<i64>,
    pub high_price: Option<Number>,
    pub low_price: Option<Number>,
    pub close_price: Option<Number>,
    pub exchange_id: Option<String>,
    pub description: Option<String>,
    pub last_id: Option<String>,
    pub open_price: Option<Number>,
    pub net_change: Option<Number>,
    pub future_percent_change: Option<Number>,
    pub exchange_name: Option<String>,
    pub security_status: Option<String>,
    pub open_interest: Option<i64>,
    pub mark: Option<Number>,
    pub tick: Option<Number>,
    pub tick_amount: Option<Number>,
    pub product: Option<String>,
    pub expiration_date: Option<Number>,
    pub expiration_style: Option<String>,
    pub strike_price: Option<Number>,
    pub contract_type: Option<String>,
}

impl LevelOneFuturesOption {
    /// Parse a streaming JSON object into a `LevelOneFuturesOption`.
    ///
    /// Returns `None` if `value` is not a JSON object.
    pub(crate) fn from_value(value: &serde_json::Value) -> Option<Self> {
        let obj = value.as_object()?;
        let mut result = Self::default();

        // Metadata fields
        result.key = obj.get("key").and_then(|v| v.as_str()).map(String::from);
        result.delayed = obj.get("delayed").and_then(|v| v.as_bool());
        result.asset_main_type = obj
            .get("assetMainType")
            .and_then(|v| v.as_str())
            .map(String::from);
        result.asset_sub_type = obj
            .get("assetSubType")
            .and_then(|v| v.as_str())
            .map(String::from);
        result.cusip = obj.get("cusip").and_then(|v| v.as_str()).map(String::from);

        // String fields (indices 0, 6, 7, 15, 16, 17, 21, 22, 27, 29, 31)
        result.symbol = obj.get("0").and_then(|v| v.as_str()).map(String::from);
        result.bid_id = obj.get("6").and_then(|v| v.as_str()).map(String::from);
        result.ask_id = obj.get("7").and_then(|v| v.as_str()).map(String::from);
        result.exchange_id = obj.get("15").and_then(|v| v.as_str()).map(String::from);
        result.description = obj.get("16").and_then(|v| v.as_str()).map(String::from);
        result.last_id = obj.get("17").and_then(|v| v.as_str()).map(String::from);
        result.exchange_name = obj.get("21").and_then(|v| v.as_str()).map(String::from);
        result.security_status = obj.get("22").and_then(|v| v.as_str()).map(String::from);
        result.product = obj.get("27").and_then(|v| v.as_str()).map(String::from);
        result.expiration_style = obj.get("29").and_then(|v| v.as_str()).map(String::from);
        result.contract_type = obj.get("31").and_then(|v| v.as_str()).map(String::from);

        // i64 fields (indices 4, 5, 8, 9, 10, 11, 23)
        result.bid_size = obj.get("4").and_then(|v| v.as_i64());
        result.ask_size = obj.get("5").and_then(|v| v.as_i64());
        result.total_volume = obj.get("8").and_then(|v| v.as_i64());
        result.last_size = obj.get("9").and_then(|v| v.as_i64());
        result.quote_time = obj.get("10").and_then(|v| v.as_i64());
        result.trade_time = obj.get("11").and_then(|v| v.as_i64());
        result.open_interest = obj.get("23").and_then(|v| v.as_i64());

        // Number fields (indices 1, 2, 3, 12, 13, 14, 18, 19, 20, 24, 25, 26, 28, 30)
        result.bid_price = obj.get("1").and_then(parse_num);
        result.ask_price = obj.get("2").and_then(parse_num);
        result.last_price = obj.get("3").and_then(parse_num);
        result.high_price = obj.get("12").and_then(parse_num);
        result.low_price = obj.get("13").and_then(parse_num);
        result.close_price = obj.get("14").and_then(parse_num);
        result.open_price = obj.get("18").and_then(parse_num);
        result.net_change = obj.get("19").and_then(parse_num);
        result.future_percent_change = obj.get("20").and_then(parse_num);
        result.mark = obj.get("24").and_then(parse_num);
        result.tick = obj.get("25").and_then(parse_num);
        result.tick_amount = obj.get("26").and_then(parse_num);
        result.expiration_date = obj.get("28").and_then(parse_num);
        result.strike_price = obj.get("30").and_then(parse_num);

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn field_all_has_32_variants() {
        assert_eq!(FuturesOptionField::all().len(), 32);
    }

    #[test]
    fn field_indices_are_sequential() {
        for (i, field) in FuturesOptionField::all().iter().enumerate() {
            assert_eq!(field.index(), i as u32);
        }
    }

    #[test]
    fn from_value_parses_futures_option() {
        let value = json!({
            "key": "/ESM25C5500",
            "1": 25.50,
            "30": 5500.0,
            "31": "C"
        });

        let parsed = LevelOneFuturesOption::from_value(&value).unwrap();
        assert_eq!(parsed.key, Some("/ESM25C5500".to_string()));
        assert_eq!(parsed.bid_price, Some("25.5".parse().unwrap()));
        assert_eq!(parsed.strike_price, Some("5500.0".parse().unwrap()));
        assert_eq!(parsed.contract_type, Some("C".to_string()));
        // Unset fields remain None
        assert_eq!(parsed.symbol, None);
        assert_eq!(parsed.last_price, None);
    }

    #[test]
    fn from_value_returns_none_for_non_object() {
        assert!(LevelOneFuturesOption::from_value(&json!("not an object")).is_none());
        assert!(LevelOneFuturesOption::from_value(&json!(42)).is_none());
        assert!(LevelOneFuturesOption::from_value(&json!(null)).is_none());
    }
}
