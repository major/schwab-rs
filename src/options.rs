//! Builder types for optional OpenAPI query parameters.

use crate::Result;
use crate::query::{push_optional, required_text};

/// Optional query parameters for [`crate::Client::get_quotes_with_options`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QuoteOptions {
    pub(crate) fields: Option<String>,
    pub(crate) indicative: bool,
}

impl QuoteOptions {
    /// Creates quote options with Schwab's default response fields.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests a comma-separated subset of Schwab quote root nodes.
    #[must_use]
    pub fn fields(mut self, fields: impl Into<String>) -> Self {
        self.fields = Some(fields.into());
        self
    }

    /// Includes indicative ETF quotes when Schwab supports them for the symbol.
    #[must_use]
    pub fn indicative(mut self, indicative: bool) -> Self {
        self.indicative = indicative;
        self
    }
}

/// Optional query parameters for `GET /chains`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptionChainOptions {
    pub(crate) symbol: String,
    query: Vec<(&'static str, String)>,
}

impl OptionChainOptions {
    /// Creates option-chain parameters with the required underlying symbol.
    #[must_use]
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            query: Vec::new(),
        }
    }

    /// Adds an optional string query parameter exactly as Schwab documents it.
    #[must_use]
    pub fn parameter(mut self, name: &'static str, value: impl Into<String>) -> Self {
        let value = value.into();
        if !value.trim().is_empty() {
            self.query.push((name, value));
        }
        self
    }

    /// Adds an optional integer query parameter exactly as Schwab documents it.
    #[must_use]
    pub fn integer_parameter(mut self, name: &'static str, value: i64) -> Self {
        self.query.push((name, value.to_string()));
        self
    }

    /// Adds an optional floating-point query parameter exactly as Schwab documents it.
    #[must_use]
    pub fn number_parameter(mut self, name: &'static str, value: f64) -> Self {
        self.query.push((name, value.to_string()));
        self
    }

    /// Includes the underlying quote in the chain response.
    #[must_use]
    pub fn include_underlying_quote(mut self, include: bool) -> Self {
        if include {
            self.query
                .push(("includeUnderlyingQuote", include.to_string()));
        }
        self
    }

    pub(crate) fn into_query(self) -> Vec<(&'static str, String)> {
        let mut query = vec![("symbol", self.symbol)];
        query.extend(self.query);
        query
    }
}

/// Optional query parameters for `GET /movers/{symbol_id}`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MoverOptions {
    sort: Option<String>,
    frequency: Option<i64>,
}

impl MoverOptions {
    /// Creates empty mover options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets Schwab's `sort` query parameter.
    #[must_use]
    pub fn sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    /// Sets Schwab's `frequency` query parameter.
    #[must_use]
    pub fn frequency(mut self, frequency: i64) -> Self {
        self.frequency = Some(frequency);
        self
    }

    pub(crate) fn into_query(self) -> Vec<(&'static str, String)> {
        let mut query = Vec::new();
        push_optional(&mut query, "sort", self.sort);
        if let Some(frequency) = self.frequency {
            query.push(("frequency", frequency.to_string()));
        }
        query
    }
}

/// Optional query parameters for `GET /pricehistory`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PriceHistoryOptions {
    query: Vec<(&'static str, String)>,
}

impl PriceHistoryOptions {
    /// Creates empty price-history options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an optional query parameter exactly as Schwab documents it.
    #[must_use]
    pub fn parameter(mut self, name: &'static str, value: impl Into<String>) -> Self {
        let value = value.into();
        if !value.trim().is_empty() {
            self.query.push((name, value));
        }
        self
    }

    /// Adds an optional integer query parameter exactly as Schwab documents it.
    #[must_use]
    pub fn integer_parameter(mut self, name: &'static str, value: i64) -> Self {
        self.query.push((name, value.to_string()));
        self
    }

    /// Adds an optional boolean query parameter exactly as Schwab documents it.
    #[must_use]
    pub fn bool_parameter(mut self, name: &'static str, value: bool) -> Self {
        self.query.push((name, value.to_string()));
        self
    }

    pub(crate) fn into_query(self) -> Vec<(&'static str, String)> {
        self.query
    }
}

/// Required and optional query parameters shared by Trader order-list endpoints.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderListOptions {
    from_entered_time: String,
    to_entered_time: String,
    max_results: Option<i64>,
    status: Option<String>,
}

impl OrderListOptions {
    /// Creates order-list options with Schwab's required time range.
    #[must_use]
    pub fn new(from_entered_time: impl Into<String>, to_entered_time: impl Into<String>) -> Self {
        Self {
            from_entered_time: from_entered_time.into(),
            to_entered_time: to_entered_time.into(),
            max_results: None,
            status: None,
        }
    }

    /// Sets Schwab's `maxResults` query parameter.
    #[must_use]
    pub fn max_results(mut self, max_results: i64) -> Self {
        self.max_results = Some(max_results);
        self
    }

    /// Sets Schwab's `status` query parameter.
    #[must_use]
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub(crate) fn into_query(self) -> Result<Vec<(&'static str, String)>> {
        let from_entered_time = required_text("fromEnteredTime", &self.from_entered_time)?;
        let to_entered_time = required_text("toEnteredTime", &self.to_entered_time)?;
        let mut query = vec![
            ("fromEnteredTime", from_entered_time.to_owned()),
            ("toEnteredTime", to_entered_time.to_owned()),
        ];
        if let Some(max_results) = self.max_results {
            query.push(("maxResults", max_results.to_string()));
        }
        push_optional(&mut query, "status", self.status);
        Ok(query)
    }
}

/// Required and optional query parameters for Trader transaction-list endpoints.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionListOptions {
    start_date: String,
    end_date: String,
    types: String,
    symbol: Option<String>,
}

impl TransactionListOptions {
    /// Creates transaction-list options with Schwab's required date range and transaction types.
    #[must_use]
    pub fn new(
        start_date: impl Into<String>,
        end_date: impl Into<String>,
        types: impl Into<String>,
    ) -> Self {
        Self {
            start_date: start_date.into(),
            end_date: end_date.into(),
            types: types.into(),
            symbol: None,
        }
    }

    /// Sets Schwab's optional `symbol` query parameter.
    #[must_use]
    pub fn symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    pub(crate) fn into_query(self) -> Result<Vec<(&'static str, String)>> {
        let start_date = required_text("startDate", &self.start_date)?;
        let end_date = required_text("endDate", &self.end_date)?;
        let types = required_text("types", &self.types)?;
        let mut query = vec![
            ("startDate", start_date.to_owned()),
            ("endDate", end_date.to_owned()),
            ("types", types.to_owned()),
        ];
        push_optional(&mut query, "symbol", self.symbol);
        Ok(query)
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;

    use super::*;

    #[test]
    fn required_options_reject_empty_values() {
        let options = OrderListOptions::new("", "2024-01-31T00:00:00Z");
        assert!(matches!(
            options.into_query(),
            Err(Error::MissingRequiredParameter("fromEnteredTime"))
        ));
    }

    #[test]
    fn optional_query_helpers_trim_empty_values() {
        assert!(MoverOptions::new().sort("   ").into_query().is_empty());
        assert_eq!(
            OrderListOptions::new("2024-01-01", "2024-01-02")
                .status("   ")
                .into_query()
                .unwrap(),
            vec![
                ("fromEnteredTime", "2024-01-01".to_string()),
                ("toEnteredTime", "2024-01-02".to_string()),
            ]
        );
    }
}
