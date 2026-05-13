pub mod enums;
pub mod market_data;
pub mod trader;

pub use enums::*;
pub use market_data::*;
pub use trader::*;

/// Numeric type for financial values.
///
/// Defaults to [`f64`]. Enable the `decimal` crate feature to switch to
/// [`rust_decimal::Decimal`], which avoids floating-point rounding in
/// financial calculations.
#[cfg(not(feature = "decimal"))]
pub type Number = f64;

/// Numeric type for financial values backed by [`rust_decimal::Decimal`].
///
/// Activated by the `decimal` crate feature.
#[cfg(feature = "decimal")]
pub type Number = rust_decimal::Decimal;

/// Compatibility alias: use `market_data::QuoteResponse` for new code.
pub type Quotes = market_data::QuoteResponse;

/// Compatibility alias: use `market_data::QuoteResponseObject` for new code.
pub type Quote = market_data::QuoteResponseObject;
