use super::super::Number;

/// Fields available in level-one option streaming data.
///
/// Each variant maps to an index in the Schwab streaming response array.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum OptionField {
    Symbol = 0,
    Description = 1,
    BidPrice = 2,
    AskPrice = 3,
    LastPrice = 4,
    HighPrice = 5,
    LowPrice = 6,
    ClosePrice = 7,
    TotalVolume = 8,
    OpenInterest = 9,
    Volatility = 10,
    IntrinsicValue = 11,
    ExpirationYear = 12,
    Multiplier = 13,
    StrikePrice = 14,
    ContractType = 15,
    Underlying = 16,
    ExpirationMonth = 17,
    TimeValue = 18,
    ExpirationDay = 19,
    DaysToExpiration = 20,
    OpenPrice = 21,
    BidSize = 22,
    AskSize = 23,
    LastSize = 24,
    NetChange = 25,
    SecurityStatus = 26,
    Mark = 27,
    QuoteTime = 28,
    TradeTime = 29,
    ExchangeId = 30,
    ExchangeName = 31,
    LastTradingDay = 32,
    SettlementType = 33,
    NetPercentChange = 34,
    MarkNetChange = 35,
    MarkPercentChange = 36,
    ImpliedYield = 37,
    IsPennyPilot = 38,
    OptionRoot = 39,
    High52Week = 40,
    Low52Week = 41,
    IndicativeAskPrice = 42,
    IndicativeBidPrice = 43,
    IndicativeQuoteTime = 44,
    ExerciseType = 45,
    Delta = 46,
    Gamma = 47,
    Theta = 48,
    Vega = 49,
    Rho = 50,
    TheoreticalOptionValue = 51,
    UnderlyingPrice = 52,
    UvExpirationType = 53,
    ExpirationString = 54,
    DeliverableNote = 55,
}

static ALL_FIELDS: [OptionField; 56] = [
    OptionField::Symbol,
    OptionField::Description,
    OptionField::BidPrice,
    OptionField::AskPrice,
    OptionField::LastPrice,
    OptionField::HighPrice,
    OptionField::LowPrice,
    OptionField::ClosePrice,
    OptionField::TotalVolume,
    OptionField::OpenInterest,
    OptionField::Volatility,
    OptionField::IntrinsicValue,
    OptionField::ExpirationYear,
    OptionField::Multiplier,
    OptionField::StrikePrice,
    OptionField::ContractType,
    OptionField::Underlying,
    OptionField::ExpirationMonth,
    OptionField::TimeValue,
    OptionField::ExpirationDay,
    OptionField::DaysToExpiration,
    OptionField::OpenPrice,
    OptionField::BidSize,
    OptionField::AskSize,
    OptionField::LastSize,
    OptionField::NetChange,
    OptionField::SecurityStatus,
    OptionField::Mark,
    OptionField::QuoteTime,
    OptionField::TradeTime,
    OptionField::ExchangeId,
    OptionField::ExchangeName,
    OptionField::LastTradingDay,
    OptionField::SettlementType,
    OptionField::NetPercentChange,
    OptionField::MarkNetChange,
    OptionField::MarkPercentChange,
    OptionField::ImpliedYield,
    OptionField::IsPennyPilot,
    OptionField::OptionRoot,
    OptionField::High52Week,
    OptionField::Low52Week,
    OptionField::IndicativeAskPrice,
    OptionField::IndicativeBidPrice,
    OptionField::IndicativeQuoteTime,
    OptionField::ExerciseType,
    OptionField::Delta,
    OptionField::Gamma,
    OptionField::Theta,
    OptionField::Vega,
    OptionField::Rho,
    OptionField::TheoreticalOptionValue,
    OptionField::UnderlyingPrice,
    OptionField::UvExpirationType,
    OptionField::ExpirationString,
    OptionField::DeliverableNote,
];

impl OptionField {
    /// Return the numeric index for this field.
    pub fn index(&self) -> u32 {
        *self as u32
    }

    /// Return a slice of all field variants in index order.
    pub fn all() -> &'static [OptionField] {
        &ALL_FIELDS
    }
}

/// Level-one option quote from the Schwab streaming API.
///
/// Built from a raw JSON object via the crate-internal `from_value` parser.
/// Metadata fields use string keys; data fields use numeric index keys.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(missing_docs)]
pub struct LevelOneOption {
    // Metadata (string-keyed)
    pub key: Option<String>,
    pub delayed: Option<bool>,
    pub asset_main_type: Option<String>,
    pub asset_sub_type: Option<String>,
    pub cusip: Option<String>,

    // Data fields (numeric-keyed)
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub bid_price: Option<Number>,
    pub ask_price: Option<Number>,
    pub last_price: Option<Number>,
    pub high_price: Option<Number>,
    pub low_price: Option<Number>,
    pub close_price: Option<Number>,
    pub total_volume: Option<i64>,
    pub open_interest: Option<i64>,
    pub volatility: Option<Number>,
    pub intrinsic_value: Option<Number>,
    pub expiration_year: Option<i64>,
    pub multiplier: Option<Number>,
    pub strike_price: Option<Number>,
    pub contract_type: Option<String>,
    pub underlying: Option<String>,
    pub expiration_month: Option<i64>,
    pub time_value: Option<Number>,
    pub expiration_day: Option<i64>,
    pub days_to_expiration: Option<i64>,
    pub open_price: Option<Number>,
    pub bid_size: Option<i64>,
    pub ask_size: Option<i64>,
    pub last_size: Option<i64>,
    pub net_change: Option<Number>,
    pub security_status: Option<String>,
    pub mark: Option<Number>,
    pub quote_time: Option<i64>,
    pub trade_time: Option<i64>,
    pub exchange_id: Option<String>,
    pub exchange_name: Option<String>,
    pub last_trading_day: Option<i64>,
    pub settlement_type: Option<String>,
    pub net_percent_change: Option<Number>,
    pub mark_net_change: Option<Number>,
    pub mark_percent_change: Option<Number>,
    pub implied_yield: Option<Number>,
    pub is_penny_pilot: Option<bool>,
    pub option_root: Option<String>,
    pub high_52_week: Option<Number>,
    pub low_52_week: Option<Number>,
    pub indicative_ask_price: Option<Number>,
    pub indicative_bid_price: Option<Number>,
    pub indicative_quote_time: Option<i64>,
    pub exercise_type: Option<String>,
    pub delta: Option<Number>,
    pub gamma: Option<Number>,
    pub theta: Option<Number>,
    pub vega: Option<Number>,
    pub rho: Option<Number>,
    pub theoretical_option_value: Option<Number>,
    pub underlying_price: Option<Number>,
    pub uv_expiration_type: Option<String>,
    pub expiration_string: Option<String>,
    pub deliverable_note: Option<String>,
}

/// Parse a Number from a JSON value.
fn parse_num(v: &serde_json::Value) -> Option<Number> {
    serde_json::from_value::<Number>(v.clone()).ok()
}

impl LevelOneOption {
    /// Parse a level-one option from a raw JSON object.
    ///
    /// Metadata fields (`key`, `delayed`, `assetMainType`, `assetSubType`,
    /// `cusip`) use string keys. Data fields use numeric string keys matching
    /// [`OptionField`] indices.
    ///
    /// Returns `None` if `value` is not a JSON object.
    pub(crate) fn from_value(value: &serde_json::Value) -> Option<Self> {
        let obj = value.as_object()?;
        let mut result = Self::default();

        // Metadata
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

        // Data fields by numeric index
        for (key, val) in obj {
            let idx: u32 = match key.parse() {
                Ok(n) => n,
                Err(_) => continue,
            };
            match idx {
                0 => result.symbol = val.as_str().map(String::from),
                1 => result.description = val.as_str().map(String::from),
                2 => result.bid_price = parse_num(val),
                3 => result.ask_price = parse_num(val),
                4 => result.last_price = parse_num(val),
                5 => result.high_price = parse_num(val),
                6 => result.low_price = parse_num(val),
                7 => result.close_price = parse_num(val),
                8 => result.total_volume = val.as_i64(),
                9 => result.open_interest = val.as_i64(),
                10 => result.volatility = parse_num(val),
                11 => result.intrinsic_value = parse_num(val),
                12 => result.expiration_year = val.as_i64(),
                13 => result.multiplier = parse_num(val),
                14 => result.strike_price = parse_num(val),
                15 => result.contract_type = val.as_str().map(String::from),
                16 => result.underlying = val.as_str().map(String::from),
                17 => result.expiration_month = val.as_i64(),
                18 => result.time_value = parse_num(val),
                19 => result.expiration_day = val.as_i64(),
                20 => result.days_to_expiration = val.as_i64(),
                21 => result.open_price = parse_num(val),
                22 => result.bid_size = val.as_i64(),
                23 => result.ask_size = val.as_i64(),
                24 => result.last_size = val.as_i64(),
                25 => result.net_change = parse_num(val),
                26 => result.security_status = val.as_str().map(String::from),
                27 => result.mark = parse_num(val),
                28 => result.quote_time = val.as_i64(),
                29 => result.trade_time = val.as_i64(),
                30 => result.exchange_id = val.as_str().map(String::from),
                31 => result.exchange_name = val.as_str().map(String::from),
                32 => result.last_trading_day = val.as_i64(),
                33 => result.settlement_type = val.as_str().map(String::from),
                34 => result.net_percent_change = parse_num(val),
                35 => result.mark_net_change = parse_num(val),
                36 => result.mark_percent_change = parse_num(val),
                37 => result.implied_yield = parse_num(val),
                38 => result.is_penny_pilot = val.as_bool(),
                39 => result.option_root = val.as_str().map(String::from),
                40 => result.high_52_week = parse_num(val),
                41 => result.low_52_week = parse_num(val),
                42 => result.indicative_ask_price = parse_num(val),
                43 => result.indicative_bid_price = parse_num(val),
                44 => result.indicative_quote_time = val.as_i64(),
                45 => result.exercise_type = val.as_str().map(String::from),
                46 => result.delta = parse_num(val),
                47 => result.gamma = parse_num(val),
                48 => result.theta = parse_num(val),
                49 => result.vega = parse_num(val),
                50 => result.rho = parse_num(val),
                51 => result.theoretical_option_value = parse_num(val),
                52 => result.underlying_price = parse_num(val),
                53 => result.uv_expiration_type = val.as_str().map(String::from),
                54 => result.expiration_string = val.as_str().map(String::from),
                55 => result.deliverable_note = val.as_str().map(String::from),
                _ => {}
            }
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn option_field_count() {
        assert_eq!(OptionField::all().len(), 56);
    }

    #[test]
    fn option_field_indices() {
        assert_eq!(OptionField::Symbol.index(), 0);
        assert_eq!(OptionField::StrikePrice.index(), 14);
        assert_eq!(OptionField::Delta.index(), 46);
        assert_eq!(OptionField::DeliverableNote.index(), 55);
    }

    #[test]
    fn option_field_all_matches_indices() {
        for (i, field) in OptionField::all().iter().enumerate() {
            assert_eq!(
                field.index() as usize,
                i,
                "field {field:?} at position {i} has wrong index"
            );
        }
    }

    #[test]
    fn from_value_parses_option_fields() {
        let v = json!({
            "key": "AAPL  251219C00200000",
            "delayed": false,
            "2": 5.50,
            "14": 200.0,
            "46": 0.45
        });
        let opt = LevelOneOption::from_value(&v).unwrap();

        assert_eq!(opt.key.as_deref(), Some("AAPL  251219C00200000"));
        assert_eq!(opt.delayed, Some(false));

        let expected_bid: Number = "5.50".parse().unwrap();
        assert_eq!(opt.bid_price, Some(expected_bid));

        let expected_strike: Number = "200.0".parse().unwrap();
        assert_eq!(opt.strike_price, Some(expected_strike));

        let expected_delta: Number = "0.45".parse().unwrap();
        assert_eq!(opt.delta, Some(expected_delta));

        // Unprovided fields should be None
        assert_eq!(opt.symbol, None);
        assert_eq!(opt.gamma, None);
    }

    #[test]
    fn from_value_returns_none_for_non_object() {
        assert!(LevelOneOption::from_value(&json!(42)).is_none());
        assert!(LevelOneOption::from_value(&json!("hello")).is_none());
        assert!(LevelOneOption::from_value(&json!(null)).is_none());
    }
}
