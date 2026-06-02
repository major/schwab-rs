use std::io;

/// Application error with stable machine-readable classification.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Required authentication configuration was not provided.
    #[error("missing required authentication setting: {0}")]
    MissingAuthConfig(&'static str),
    /// The saved token file is missing.
    #[error("token file not found: {0}")]
    TokenFileMissing(String),
    /// Reading or writing local state failed.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// Local JSON could not be decoded or encoded.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// schwab-rs returned an error.
    #[error("Schwab error: {0}")]
    Schwab(#[from] schwab::Error),
    /// Order validation failed (invalid strikes, missing fields, etc.).
    #[error("{0}")]
    OrderValidation(String),
    /// Account selection or validation failed.
    #[error("{0}")]
    AccountValidation(String),
    /// Schwab returned an account response envelope this CLI could not decode.
    #[error("unexpected Schwab {endpoint} response shape: expected {expected}; got {shape}")]
    AccountResponseShape {
        /// Human-readable account endpoint label.
        endpoint: &'static str,
        /// Expected sanitized shape description.
        expected: &'static str,
        /// Sanitized shape metadata from the actual response.
        shape: String,
    },
    /// The symbol does not have listed options or was not found.
    #[error("symbol has no listed options: {symbol}")]
    OptionsSymbolNotFound { symbol: String },
    /// Options command input validation failed.
    #[error("{message}")]
    OptionsValidation { message: String },
    /// Market-data command input validation failed.
    #[error("{message}")]
    MarketValidation { message: String },
    /// Not enough candle data to compute the indicator.
    #[error("not enough candle data for {indicator}: need {needed} candles, got {got}")]
    TaInsufficientData {
        needed: usize,
        got: usize,
        indicator: String,
    },
    /// Unrecognized interval string.
    #[error(
        "unrecognized interval '{interval}': valid values are daily, weekly, 1min, 5min, 15min, 30min"
    )]
    TaInvalidInterval { interval: String },
    /// Indicator math failure (e.g., division by zero).
    #[error("TA calculation error in {indicator}: {reason}")]
    TaCalculationError { indicator: String, reason: String },
    /// Preview digest operation failed (save, load, verify, or expired).
    #[error("preview error: {0}")]
    Preview(String),
    /// Mutable operations are disabled in the agent config.
    #[error("mutable operations are disabled by default")]
    MutableDisabled,
    /// A removed or legacy command needs a migration hint.
    #[error("{message}")]
    CommandMigration {
        /// Human-readable migration failure.
        message: &'static str,
        /// Exact replacement command hint.
        hint: &'static str,
    },
}

impl AppError {
    /// Returns the process exit code for this error.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::MissingAuthConfig(_) | Self::TokenFileMissing(_) => 3,
            Self::Io(_) | Self::Json(_) => 20,
            Self::Schwab(error) => classify_schwab_error(error).0,
            Self::OrderValidation(_) => 10,
            Self::AccountValidation(_) => 10,
            Self::AccountResponseShape { .. } => 20,
            Self::OptionsSymbolNotFound { .. } | Self::OptionsValidation { .. } => 10,
            Self::MarketValidation { .. } => 10,
            Self::TaInsufficientData { .. } | Self::TaInvalidInterval { .. } => 10,
            Self::TaCalculationError { .. } => 20,
            Self::Preview(_) => 11,
            Self::MutableDisabled => 10,
            Self::CommandMigration { .. } => 2,
        }
    }

    /// Returns a stable error code.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingAuthConfig(_) => "auth.config_missing",
            Self::TokenFileMissing(_) => "auth.token_missing",
            Self::Io(_) => "io.error",
            Self::Json(_) => "json.error",
            Self::Schwab(error) => classify_schwab_error(error).1,
            Self::OrderValidation(_) => "order.validation_failed",
            Self::AccountValidation(_) => "account.validation_failed",
            Self::AccountResponseShape { .. } => "account.response_shape",
            Self::OptionsSymbolNotFound { .. } => "options.symbol_not_found",
            Self::OptionsValidation { .. } => "options.validation_failed",
            Self::MarketValidation { .. } => "market.validation_failed",
            Self::TaInsufficientData { .. } => "ta.insufficient_data",
            Self::TaInvalidInterval { .. } => "ta.invalid_interval",
            Self::TaCalculationError { .. } => "ta.calculation_error",
            Self::Preview(_) => "order.preview_failed",
            Self::MutableDisabled => "config.mutable_disabled",
            Self::CommandMigration { .. } => "usage.migration",
        }
    }

    /// Returns a coarse error category.
    #[must_use]
    pub fn category(&self) -> &'static str {
        match self {
            Self::MissingAuthConfig(_) | Self::TokenFileMissing(_) => "auth",
            Self::Io(_) => "io",
            Self::Json(_) => "json",
            Self::Schwab(error) => classify_schwab_error(error).2,
            Self::OrderValidation(_) | Self::Preview(_) => "order",
            Self::AccountValidation(_) | Self::AccountResponseShape { .. } => "account",
            Self::OptionsSymbolNotFound { .. } | Self::OptionsValidation { .. } => "options",
            Self::MarketValidation { .. } => "market",
            Self::TaInsufficientData { .. }
            | Self::TaInvalidInterval { .. }
            | Self::TaCalculationError { .. } => "ta",
            Self::MutableDisabled => "config",
            Self::CommandMigration { .. } => "usage",
        }
    }

    /// Returns true if retrying may succeed without changing inputs.
    #[must_use]
    pub fn retryable(&self) -> bool {
        matches!(self, Self::Schwab(schwab::Error::Request(_)))
    }

    /// Returns a remediation hint when the action is obvious.
    #[must_use]
    pub fn hint(&self) -> Option<&'static str> {
        match self {
            Self::MissingAuthConfig(_) => Some(
                "Add client_id and client_secret to ~/.config/schwab-agent/config.json or set SCHWAB_CLIENT_ID and SCHWAB_CLIENT_SECRET.",
            ),
            Self::TokenFileMissing(_) => {
                Some("Run auth login-url, then auth exchange, to create a token file.")
            }
            Self::OptionsSymbolNotFound { .. } => {
                Some("Check that the symbol is correct and has listed options")
            }
            Self::AccountValidation(_) => {
                Some("Run account to list available account hashes and nicknames.")
            }
            Self::AccountResponseShape { .. } => Some(
                "Schwab returned an account response shape this version does not recognize. Update schwab-agent or report the sanitized shape metadata.",
            ),
            Self::MarketValidation { .. } => Some(
                "Use --fields with one or more supported quote output fields, for example sym,last,pct,vol.",
            ),
            Self::Schwab(schwab::Error::AuthExpired | schwab::Error::AuthRequired) => {
                Some("Run auth refresh, or re-authenticate with auth login-url and auth exchange.")
            }
            Self::TaInsufficientData { .. } => Some("Try a shorter interval or fewer --points."),
            Self::TaInvalidInterval { .. } => {
                Some("Valid intervals: daily, weekly, 1min, 5min, 15min, 30min")
            }
            Self::MutableDisabled => Some(
                "Set \"i-also-like-to-live-dangerously\": true in ~/.config/schwab-agent/config.json to enable order placement, cancellation, and replacement.",
            ),
            Self::CommandMigration { hint, .. } => Some(hint),
            _ => None,
        }
    }
}

/// Classifies a `schwab::Error` into `(exit_code, code, category)`.
///
/// These values are part of the public JSON output contract and must not change
/// between releases without a version bump.
fn classify_schwab_error(error: &schwab::Error) -> (i32, &'static str, &'static str) {
    match error {
        schwab::Error::AuthRequired => (3, "auth.required", "auth"),
        schwab::Error::AuthExpired => (3, "auth.expired", "auth"),
        schwab::Error::AuthCallback(_) => (3, "auth.callback_failed", "auth"),
        schwab::Error::HttpStatus { .. } => (4, "schwab.http_status", "schwab"),
        schwab::Error::Request(_) => (1, "schwab.request_failed", "schwab"),
        schwab::Error::Decode { .. } => (1, "schwab.decode_failed", "schwab"),
        schwab::Error::Json(_) => (1, "auth.json_failed", "auth"),
        schwab::Error::Io(_) => (1, "auth.io_failed", "auth"),
        schwab::Error::EmptySymbols => (10, "input.empty_symbols", "input"),
        schwab::Error::MissingRequiredParameter(_) => (10, "input.missing_parameter", "input"),
        schwab::Error::InvalidAuthConfig { .. } => (20, "auth.config_invalid", "auth"),
        schwab::Error::EmptyBaseUrl | schwab::Error::InvalidBaseUrl { .. } => {
            (20, "config.base_url_invalid", "config")
        }
        schwab::Error::Encode(_) => (1, "json.encode_failed", "json"),
        // Catch-all for schwab error variants added after our last explicit update.
        // Prevents compilation failures when schwab adds new variants before we
        // classify them. Revisit when bumping the schwab dependency.
        #[allow(unreachable_patterns)]
        _ => (1, "schwab.unknown", "schwab"),
    }
}

#[cfg(test)]
mod tests;
