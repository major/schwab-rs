//! Level-two book streaming data types.

use super::super::Number;

/// Field selector for level-two book streaming subscriptions.
///
/// Each variant corresponds to a numeric field index in the Schwab streaming protocol.
///
/// # Examples
///
/// ```
/// use schwab::BookField;
///
/// assert_eq!(BookField::Symbol.index(), 0);
/// assert_eq!(BookField::BidSideLevels.index(), 2);
/// ```
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BookField {
    Symbol = 0,
    MarketSnapshotTime = 1,
    BidSideLevels = 2,
    AskSideLevels = 3,
}

impl BookField {
    /// Return the numeric field index used in the Schwab streaming protocol.
    #[must_use]
    pub fn index(&self) -> u32 {
        *self as u32
    }

    /// Return all `BookField` variants in index order.
    #[must_use]
    pub fn all() -> &'static [BookField] {
        use BookField::*;
        &[Symbol, MarketSnapshotTime, BidSideLevels, AskSideLevels]
    }
}

/// Level-two book streaming data for a single symbol.
///
/// The same payload shape is used by `NYSE_BOOK`, `NASDAQ_BOOK`, and `OPTIONS_BOOK`.
/// Nested bid and ask levels are whole-book snapshots from the streaming service.
///
/// # Examples
///
/// ```
/// use schwab::Book;
///
/// let data = Book {
///     symbol: Some("AAPL".to_string()),
///     ..Default::default()
/// };
/// assert_eq!(data.symbol.as_deref(), Some("AAPL"));
/// ```
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Book {
    // Named metadata fields (string-keyed in the protocol)
    pub key: Option<String>,
    pub delayed: Option<bool>,
    pub asset_main_type: Option<String>,
    pub asset_sub_type: Option<String>,
    pub cusip: Option<String>,
    // Numeric data fields (index-keyed: "0", "1", ...)
    pub symbol: Option<String>,
    pub market_snapshot_time: Option<i64>,
    pub bid_side_levels: Option<Vec<BookPriceLevel>>,
    pub ask_side_levels: Option<Vec<BookPriceLevel>>,
}

/// Price level in a level-two book snapshot.
///
/// # Examples
///
/// ```
/// use schwab::BookPriceLevel;
///
/// let level = BookPriceLevel {
///     aggregate_size: Some(100),
///     ..Default::default()
/// };
/// assert_eq!(level.aggregate_size, Some(100));
/// ```
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BookPriceLevel {
    pub price: Option<Number>,
    pub aggregate_size: Option<i64>,
    pub market_maker_count: Option<i64>,
    pub market_makers: Option<Vec<BookMarketMaker>>,
}

/// Market maker entry within a level-two book price level.
///
/// # Examples
///
/// ```
/// use schwab::BookMarketMaker;
///
/// let maker = BookMarketMaker {
///     market_maker_id: Some("NSDQ".to_string()),
///     ..Default::default()
/// };
/// assert_eq!(maker.market_maker_id.as_deref(), Some("NSDQ"));
/// ```
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BookMarketMaker {
    pub market_maker_id: Option<String>,
    pub size: Option<i64>,
    pub quote_time: Option<i64>,
}

/// Parse a [`Number`] from a [`serde_json::Value`].
fn parse_num(v: &serde_json::Value) -> Option<Number> {
    serde_json::from_value::<Number>(v.clone()).ok()
}

fn indexed_value(value: &serde_json::Value, index: usize) -> Option<&serde_json::Value> {
    if let Some(items) = value.as_array() {
        return items.get(index);
    }

    let key = index.to_string();
    value.as_object().and_then(|map| map.get(&key))
}

fn parse_price_levels(value: &serde_json::Value) -> Option<Vec<BookPriceLevel>> {
    let levels = value.as_array()?;

    Some(
        levels
            .iter()
            .map(|level| BookPriceLevel {
                price: indexed_value(level, 0).and_then(parse_num),
                aggregate_size: indexed_value(level, 1).and_then(serde_json::Value::as_i64),
                market_maker_count: indexed_value(level, 2).and_then(serde_json::Value::as_i64),
                market_makers: indexed_value(level, 3).and_then(parse_market_makers),
            })
            .collect(),
    )
}

fn parse_market_makers(value: &serde_json::Value) -> Option<Vec<BookMarketMaker>> {
    let makers = value.as_array()?;

    Some(
        makers
            .iter()
            .map(|maker| BookMarketMaker {
                market_maker_id: indexed_value(maker, 0)
                    .and_then(serde_json::Value::as_str)
                    .map(String::from),
                size: indexed_value(maker, 1).and_then(serde_json::Value::as_i64),
                quote_time: indexed_value(maker, 2).and_then(serde_json::Value::as_i64),
            })
            .collect(),
    )
}

impl Book {
    /// Construct a [`Book`] from a streaming data map entry.
    ///
    /// The map uses named string keys for metadata (`"key"`, `"delayed"`) and
    /// numeric string keys (`"0"`, `"1"`, ...) for field data. Nested book
    /// level arrays follow Schwab's documented price-level and market-maker
    /// sub-field indexes.
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

        for (key, val) in map {
            match key.as_str() {
                "0" => s.symbol = val.as_str().map(String::from),
                "1" => s.market_snapshot_time = val.as_i64(),
                "2" => s.bid_side_levels = parse_price_levels(val),
                "3" => s.ask_side_levels = parse_price_levels(val),
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
        assert_eq!(BookField::Symbol.index(), 0);
    }

    #[test]
    fn field_index_last() {
        assert_eq!(BookField::AskSideLevels.index(), 3);
    }

    #[test]
    fn all_fields_count() {
        assert_eq!(BookField::all().len(), 4);
    }

    #[test]
    fn all_fields_sequential_indices() {
        for (i, field) in BookField::all().iter().enumerate() {
            assert_eq!(
                field.index() as usize,
                i,
                "field at position {i} has wrong index"
            );
        }
    }

    #[test]
    fn from_value_parses_array_backed_sample() {
        let input = json!({
            "key": "AAPL",
            "delayed": false,
            "0": "AAPL",
            "1": 1234567890000_i64,
            "2": [[150.25, 300, 2, [["NSDQ", 100, 1234567890001_i64]]]],
            "3": [[150.30, 200, 1, [["ARCA", 200, 1234567890002_i64]]]]
        });

        let book = Book::from_value(&input).expect("should parse JSON object");
        let bid_levels = book.bid_side_levels.expect("should have bid levels");
        let makers = bid_levels[0]
            .market_makers
            .as_ref()
            .expect("should have market makers");

        assert_eq!(book.key, Some("AAPL".to_string()));
        assert_eq!(book.symbol, Some("AAPL".to_string()));
        assert_eq!(book.market_snapshot_time, Some(1234567890000));
        assert_eq!(bid_levels[0].price, Some("150.25".parse().unwrap()));
        assert_eq!(bid_levels[0].aggregate_size, Some(300));
        assert_eq!(bid_levels[0].market_maker_count, Some(2));
        assert_eq!(makers[0].market_maker_id, Some("NSDQ".to_string()));
        assert_eq!(makers[0].size, Some(100));
    }

    #[test]
    fn from_value_parses_object_backed_nested_rows() {
        let input = json!({
            "0": "AAPL",
            "2": [{"0": 150.25, "1": 300, "2": 1, "3": [{"0": "NSDQ", "1": 100, "2": 1234567890001_i64}]}]
        });

        let book = Book::from_value(&input).expect("should parse JSON object");
        let bid_levels = book.bid_side_levels.expect("should have bid levels");
        let makers = bid_levels[0]
            .market_makers
            .as_ref()
            .expect("should have market makers");

        assert_eq!(bid_levels[0].price, Some("150.25".parse().unwrap()));
        assert_eq!(makers[0].quote_time, Some(1234567890001));
    }

    #[test]
    fn from_value_returns_none_for_non_object() {
        assert!(Book::from_value(&json!(42)).is_none());
        assert!(Book::from_value(&json!("text")).is_none());
        assert!(Book::from_value(&json!(null)).is_none());
    }
}
