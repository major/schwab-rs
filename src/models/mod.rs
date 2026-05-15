/// Enumeration types shared across market data and trader APIs.
pub mod enums;
/// Market data response types for quotes, option chains, candles, and instruments.
pub mod market_data;
/// Trader response types for accounts, orders, transactions, and user preferences.
pub mod trader;

pub use enums::*;
pub use market_data::*;
pub use trader::*;

cfg_select! {
    feature = "decimal" => {
        /// Numeric type for financial values backed by [`rust_decimal::Decimal`].
        ///
        /// Activated by the `decimal` crate feature to avoid floating-point
        /// rounding in financial calculations.
        pub type Number = rust_decimal::Decimal;
    }
    _ => {
        /// Numeric type for financial values.
        ///
        /// Defaults to [`f64`]. Enable the `decimal` crate feature to switch to
        /// `rust_decimal::Decimal`, which avoids floating-point rounding in
        /// financial calculations.
        pub type Number = f64;
    }
}

/// Compatibility alias: use `market_data::QuoteResponse` for new code.
pub type Quotes = market_data::QuoteResponse;

/// Compatibility alias: use `market_data::QuoteResponseObject` for new code.
pub type Quote = market_data::QuoteResponseObject;
