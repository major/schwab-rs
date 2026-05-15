//! Equity chart streaming data types.

use super::super::Number;

/// Field selector for equity chart streaming subscriptions.
///
/// Each variant corresponds to a numeric field index in the Schwab streaming protocol.
///
/// # Examples
///
/// ```
/// use schwab::ChartEquityField;
///
/// assert_eq!(ChartEquityField::Key.index(), 0);
/// assert_eq!(ChartEquityField::OpenPrice.index(), 1);
/// ```
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChartEquityField {
    Key = 0,
    OpenPrice = 1,
    HighPrice = 2,
    LowPrice = 3,
    ClosePrice = 4,
    Volume = 5,
    Sequence = 6,
    ChartTime = 7,
    ChartDay = 8,
}

impl ChartEquityField {
    /// Return the numeric field index used in the Schwab streaming protocol.
    pub fn index(&self) -> u32 {
        *self as u32
    }

    /// Return all `ChartEquityField` variants in index order.
    pub fn all() -> &'static [ChartEquityField] {
        use ChartEquityField::*;
        &[
            Key, OpenPrice, HighPrice, LowPrice, ClosePrice, Volume, Sequence, ChartTime, ChartDay,
        ]
    }
}

/// Equity chart streaming data for a single symbol.
///
/// All fields are `Option<T>` because the Schwab API sends only subscribed fields.
/// Named metadata fields use string keys; numeric data fields use numeric string keys.
///
/// # Examples
///
/// ```
/// use schwab::ChartEquity;
///
/// let data = ChartEquity {
///     chart_time: Some(1234567890000),
///     ..Default::default()
/// };
/// assert_eq!(data.chart_time, Some(1234567890000));
/// ```
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartEquity {
    // Named metadata fields (string-keyed in the protocol)
    pub key: Option<String>,
    pub delayed: Option<bool>,
    pub asset_main_type: Option<String>,
    pub asset_sub_type: Option<String>,
    pub cusip: Option<String>,
    // Numeric data fields (index-keyed: "0", "1", ...)
    pub symbol: Option<String>,
    pub open_price: Option<Number>,
    pub high_price: Option<Number>,
    pub low_price: Option<Number>,
    pub close_price: Option<Number>,
    pub volume: Option<i64>,
    pub sequence: Option<i64>,
    pub chart_time: Option<i64>,
    pub chart_day: Option<i64>,
}

/// Parse a [`Number`] from a [`serde_json::Value`].
///
/// Works for both `f64` (default) and `rust_decimal::Decimal` (`decimal` feature).
fn parse_num(v: &serde_json::Value) -> Option<Number> {
    serde_json::from_value::<Number>(v.clone()).ok()
}

impl ChartEquity {
    /// Construct a [`ChartEquity`] from a streaming data map entry.
    ///
    /// The map uses named string keys for metadata (`"key"`, `"delayed"`) and
    /// numeric string keys (`"0"`, `"1"`, ...) for field data.
    /// Returns `None` if `value` is not a JSON object.
    pub(crate) fn from_value(value: &serde_json::Value) -> Option<Self> {
        let map = value.as_object()?;
        let mut s = Self {
            key: map.get("key").and_then(|v| v.as_str()).map(String::from),
            delayed: map.get("delayed").and_then(|v| v.as_bool()),
            asset_main_type: map
                .get("assetMainType")
                .and_then(|v| v.as_str())
                .map(String::from),
            asset_sub_type: map
                .get("assetSubType")
                .and_then(|v| v.as_str())
                .map(String::from),
            cusip: map.get("cusip").and_then(|v| v.as_str()).map(String::from),
            ..Self::default()
        };

        // Numeric-keyed data fields
        for (key, val) in map {
            match key.as_str() {
                "0" => s.symbol = val.as_str().map(String::from),
                "1" => s.open_price = parse_num(val),
                "2" => s.high_price = parse_num(val),
                "3" => s.low_price = parse_num(val),
                "4" => s.close_price = parse_num(val),
                "5" => s.volume = val.as_i64(),
                "6" => s.sequence = val.as_i64(),
                "7" => s.chart_time = val.as_i64(),
                "8" => s.chart_day = val.as_i64(),
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
        assert_eq!(ChartEquityField::Key.index(), 0);
    }

    #[test]
    fn field_index_last() {
        assert_eq!(ChartEquityField::ChartDay.index(), 8);
    }

    #[test]
    fn all_fields_count() {
        assert_eq!(ChartEquityField::all().len(), 9);
    }

    #[test]
    fn all_fields_sequential_indices() {
        for (i, field) in ChartEquityField::all().iter().enumerate() {
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
            "1": 150.25,
            "2": 151.0,
            "5": 1000000,
            "7": 1234567890000_i64
        });

        let chart = ChartEquity::from_value(&input).expect("should parse JSON object");

        assert_eq!(chart.key, Some("AAPL".to_string()));
        assert_eq!(chart.open_price, Some("150.25".parse().unwrap()));
        assert_eq!(chart.high_price, Some("151".parse().unwrap()));
        assert_eq!(chart.volume, Some(1000000));
        assert_eq!(chart.chart_time, Some(1234567890000));
        // Fields not in sample remain None
        assert_eq!(chart.symbol, None);
        assert_eq!(chart.low_price, None);
    }

    #[test]
    fn from_value_returns_none_for_non_object() {
        assert!(ChartEquity::from_value(&json!(42)).is_none());
        assert!(ChartEquity::from_value(&json!("text")).is_none());
        assert!(ChartEquity::from_value(&json!(null)).is_none());
    }
}
