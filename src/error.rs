/// Convenient result type used by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the Schwab client.
#[derive(thiserror::Error)]
pub enum Error {
    /// The configured base URL was empty or only whitespace.
    #[error("base URL cannot be empty")]
    EmptyBaseUrl,
    /// The configured base URL could not be parsed as a URL.
    #[error("invalid base URL {base_url:?}: {message}")]
    InvalidBaseUrl { base_url: String, message: String },
    /// The caller tried to request quotes without any non-empty symbols.
    #[error("at least one symbol is required")]
    EmptySymbols,
    /// A required OpenAPI path, query, or body parameter was empty.
    #[error("required parameter {0} cannot be empty")]
    MissingRequiredParameter(&'static str),
    /// The configured Schwab OAuth setting is invalid.
    #[error("invalid auth config {field}: {message}")]
    InvalidAuthConfig {
        field: &'static str,
        message: String,
    },
    /// Schwab authentication must be completed before an access token is available.
    #[error("Schwab authentication is required")]
    AuthRequired,
    /// The stored Schwab refresh token is expired or revoked.
    #[error("Schwab authentication is expired")]
    AuthExpired,
    /// The localhost OAuth callback failed or returned invalid data.
    #[error("Schwab auth callback failed: {0}")]
    AuthCallback(String),
    /// Reading or writing authentication state failed.
    #[error("authentication I/O failed: {0}")]
    Io(#[source] std::io::Error),
    /// A JSON request body could not be encoded.
    #[error("failed to encode Schwab request: {0}")]
    Encode(#[source] serde_json::Error),
    /// Stored or returned JSON could not be decoded.
    #[error("failed to decode Schwab auth JSON: {0}")]
    Json(#[source] serde_json::Error),
    /// Schwab returned a non-success HTTP status.
    #[error("Schwab API returned HTTP {status}")]
    HttpStatus { status: u16, body: String },
    /// The HTTP request failed before a response body could be decoded.
    #[error("HTTP request failed: {0}")]
    Request(#[source] reqwest::Error),
    /// Schwab returned JSON that did not match the response type we expected.
    ///
    /// The raw `body` is preserved for debugging so callers can inspect the
    /// upstream payload that failed deserialization.
    #[error("failed to decode Schwab response: {source}")]
    Decode {
        /// The serde error that caused the decode failure.
        #[source]
        source: serde_json::Error,
        /// The raw response body that could not be deserialized.
        body: String,
    },
}

// Manual Debug impl to redact the HttpStatus body, which may contain
// account-specific details that should not appear in logs.
impl std::fmt::Debug for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyBaseUrl => formatter.write_str("EmptyBaseUrl"),
            Self::InvalidBaseUrl { base_url, message } => formatter
                .debug_struct("InvalidBaseUrl")
                .field("base_url", base_url)
                .field("message", message)
                .finish(),
            Self::EmptySymbols => formatter.write_str("EmptySymbols"),
            Self::MissingRequiredParameter(parameter) => formatter
                .debug_tuple("MissingRequiredParameter")
                .field(parameter)
                .finish(),
            Self::InvalidAuthConfig { field, message } => formatter
                .debug_struct("InvalidAuthConfig")
                .field("field", field)
                .field("message", message)
                .finish(),
            Self::AuthRequired => formatter.write_str("AuthRequired"),
            Self::AuthExpired => formatter.write_str("AuthExpired"),
            Self::AuthCallback(message) => formatter
                .debug_tuple("AuthCallback")
                .field(message)
                .finish(),
            Self::Io(error) => formatter.debug_tuple("Io").field(error).finish(),
            Self::Encode(error) => formatter.debug_tuple("Encode").field(error).finish(),
            Self::Json(error) => formatter.debug_tuple("Json").field(error).finish(),
            Self::HttpStatus { status, .. } => formatter
                .debug_struct("HttpStatus")
                .field("status", status)
                .field("body", &"<redacted>")
                .finish(),
            Self::Request(error) => formatter.debug_tuple("Request").field(error).finish(),
            Self::Decode { source, .. } => formatter
                .debug_struct("Decode")
                .field("source", source)
                .field("body", &"<redacted>")
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as StdError;

    use super::*;

    #[test]
    fn http_status_display_omits_response_body() {
        let error = Error::HttpStatus {
            status: 403,
            body: "account 123 token detail".to_string(),
        };

        assert_eq!(error.to_string(), "Schwab API returned HTTP 403");
        assert!(!format!("{error:?}").contains("account 123 token detail"));
    }

    #[test]
    fn display_and_sources_cover_sync_variants() {
        let encode_error = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let encode_error = Error::Encode(encode_error);

        assert_eq!(Error::EmptyBaseUrl.to_string(), "base URL cannot be empty");
        assert_eq!(
            (Error::InvalidBaseUrl {
                base_url: "not a url".to_string(),
                message: "relative URL without a base".to_string(),
            })
            .to_string(),
            "invalid base URL \"not a url\": relative URL without a base"
        );
        assert_eq!(
            Error::EmptySymbols.to_string(),
            "at least one symbol is required"
        );
        assert_eq!(
            Error::MissingRequiredParameter("symbol").to_string(),
            "required parameter symbol cannot be empty"
        );
        assert!(
            encode_error
                .to_string()
                .starts_with("failed to encode Schwab request:")
        );
        assert!(StdError::source(&encode_error).is_some());
        assert!(StdError::source(&Error::EmptyBaseUrl).is_none());
    }
}
