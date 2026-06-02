//! Shared types used across order and equity command modules.

use clap::ValueEnum;

use crate::error::AppError;

// ---------------------------------------------------------------------------
// Session / Duration choice enums (CLI-facing)
// ---------------------------------------------------------------------------

/// Trading session for the order.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SessionChoice {
    /// Regular market hours.
    #[value(alias = "regular")]
    Normal,
    /// Pre-market session.
    #[value(alias = "pre")]
    Am,
    /// After-hours session.
    #[value(alias = "post")]
    Pm,
    /// Extended hours (pre-market through after-hours).
    #[value(alias = "extended")]
    Seamless,
}

impl From<SessionChoice> for schwab::Session {
    fn from(choice: SessionChoice) -> Self {
        match choice {
            SessionChoice::Normal => schwab::Session::Normal,
            SessionChoice::Am => schwab::Session::Am,
            SessionChoice::Pm => schwab::Session::Pm,
            SessionChoice::Seamless => schwab::Session::Seamless,
        }
    }
}

/// Time-in-force for the order.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DurationChoice {
    /// Good for the current trading day only.
    #[value(alias = "DAY")]
    Day,
    /// Good until cancelled (typically 60-180 days depending on broker).
    #[value(alias = "gtc", alias = "GTC")]
    GoodTillCancel,
    /// Fill the entire order immediately or cancel it.
    #[value(alias = "fok", alias = "FOK")]
    FillOrKill,
    /// Fill as much as possible immediately, cancel the rest.
    #[value(alias = "ioc", alias = "IOC")]
    ImmediateOrCancel,
}

impl From<DurationChoice> for schwab::Duration {
    fn from(choice: DurationChoice) -> Self {
        match choice {
            DurationChoice::Day => schwab::Duration::Day,
            DurationChoice::GoodTillCancel => schwab::Duration::GoodTillCancel,
            DurationChoice::FillOrKill => schwab::Duration::FillOrKill,
            DurationChoice::ImmediateOrCancel => schwab::Duration::ImmediateOrCancel,
        }
    }
}

// ---------------------------------------------------------------------------
// Number conversion helper
// ---------------------------------------------------------------------------

/// Converts an `f64` CLI argument to [`schwab::Number`].
///
/// Without the `decimal` feature this is a no-op cast. With `decimal` enabled
/// the value is converted via `Decimal::try_from`, which rejects infinities and
/// NaN.
#[cfg(not(feature = "decimal"))]
pub fn to_number(v: f64) -> Result<schwab::Number, AppError> {
    Ok(v)
}

/// Converts an `f64` CLI argument to [`schwab::Number`] (decimal variant).
///
/// Converts via string formatting to avoid a direct `rust_decimal` dependency.
/// This mirrors the `serde-with-float` round-trip path that the API uses.
#[cfg(feature = "decimal")]
pub fn to_number(v: f64) -> Result<schwab::Number, AppError> {
    use core::str::FromStr;
    let s = format!("{v}");
    schwab::Number::from_str(&s)
        .map_err(|_| AppError::OrderValidation(format!("cannot convert {v} to decimal")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_choice_maps_to_schwab_session() {
        assert!(matches!(
            schwab::Session::from(SessionChoice::Normal),
            schwab::Session::Normal
        ));
        assert!(matches!(
            schwab::Session::from(SessionChoice::Am),
            schwab::Session::Am
        ));
        assert!(matches!(
            schwab::Session::from(SessionChoice::Pm),
            schwab::Session::Pm
        ));
        assert!(matches!(
            schwab::Session::from(SessionChoice::Seamless),
            schwab::Session::Seamless
        ));
    }

    #[test]
    fn duration_choice_maps_to_schwab_duration() {
        assert!(matches!(
            schwab::Duration::from(DurationChoice::Day),
            schwab::Duration::Day
        ));
        assert!(matches!(
            schwab::Duration::from(DurationChoice::GoodTillCancel),
            schwab::Duration::GoodTillCancel
        ));
        assert!(matches!(
            schwab::Duration::from(DurationChoice::FillOrKill),
            schwab::Duration::FillOrKill
        ));
        assert!(matches!(
            schwab::Duration::from(DurationChoice::ImmediateOrCancel),
            schwab::Duration::ImmediateOrCancel
        ));
    }

    #[test]
    fn to_number_accepts_finite_value() {
        let value = to_number(42.5).expect("finite value should convert");

        assert_eq!(value.to_string(), "42.5");
    }
}
