/// Convenient result type used by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the Schwab client.
///
/// # Examples
///
/// Match on specific error variants to handle different failure modes:
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use schwab::{Client, Config, Error};
///
/// let client = Client::new(Config::new().bearer_token("my-token"));
/// match client.get_quotes(["AAPL"]).await {
///     Ok(quotes) => println!("got {} quotes", quotes.len()),
///     Err(Error::HttpStatus { status: 401, .. }) => eprintln!("token expired"),
///     Err(Error::HttpStatus { status, .. }) => eprintln!("HTTP {status}"),
///     Err(e) => eprintln!("other error: {e}"),
/// }
/// # Ok(())
/// # }
/// ```
///
/// Validate configuration before making requests:
///
/// ```
/// use schwab::{Config, Error};
///
/// let result = Config::new().base_url("not a url");
/// assert!(matches!(result, Err(Error::InvalidBaseUrl { .. })));
/// ```
#[derive(thiserror::Error)]
pub enum Error {
    /// The configured base URL was empty or only whitespace.
    #[error("base URL cannot be empty")]
    EmptyBaseUrl,
    /// The configured base URL could not be parsed as a URL.
    #[error("invalid base URL {base_url:?}: {message}")]
    InvalidBaseUrl {
        /// The URL string that failed to parse.
        base_url: String,
        /// A human-readable description of why parsing failed.
        message: String,
    },
    /// The caller tried to request quotes without any non-empty symbols.
    #[error("at least one symbol is required")]
    EmptySymbols,
    /// A required OpenAPI path, query, or body parameter was empty.
    #[error("required parameter {0} cannot be empty")]
    MissingRequiredParameter(&'static str),
    /// The configured Schwab OAuth setting is invalid.
    #[error("invalid auth config {field}: {message}")]
    InvalidAuthConfig {
        /// The configuration field that is invalid.
        field: &'static str,
        /// A human-readable description of the validation failure.
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
    HttpStatus {
        /// The HTTP status code returned by Schwab.
        status: u16,
        /// The response body, truncated for safety.
        body: String,
    },
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

    #[test]
    fn debug_impl_covers_all_variants() {
        let serde_err = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let serde_err2 = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let serde_err3 = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let serde_err4 = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");

        let variants: Vec<Error> = vec![
            Error::EmptyBaseUrl,
            Error::InvalidBaseUrl {
                base_url: "bad".into(),
                message: "nope".into(),
            },
            Error::EmptySymbols,
            Error::MissingRequiredParameter("cusip"),
            Error::InvalidAuthConfig {
                field: "client_id",
                message: "empty".into(),
            },
            Error::AuthRequired,
            Error::AuthExpired,
            Error::AuthCallback("timeout".into()),
            Error::Io(io_err),
            Error::Encode(serde_err),
            Error::Json(serde_err2),
            Error::HttpStatus {
                status: 401,
                body: "secret data".into(),
            },
            // Request variant tested in client::tests (requires async + network).
            Error::Decode {
                source: serde_err3,
                body: "raw payload".into(),
            },
        ];

        let debug_strings: Vec<String> = variants.iter().map(|v| format!("{v:?}")).collect();

        assert_eq!(debug_strings[0], "EmptyBaseUrl");
        assert!(debug_strings[1].contains("InvalidBaseUrl"));
        assert_eq!(debug_strings[2], "EmptySymbols");
        assert!(debug_strings[3].contains("MissingRequiredParameter"));
        assert!(debug_strings[4].contains("InvalidAuthConfig"));
        assert_eq!(debug_strings[5], "AuthRequired");
        assert_eq!(debug_strings[6], "AuthExpired");
        assert!(debug_strings[7].contains("AuthCallback"));
        assert!(debug_strings[8].contains("Io"));
        assert!(debug_strings[9].contains("Encode"));
        assert!(debug_strings[10].contains("Json"));
        // HttpStatus body is redacted
        assert!(debug_strings[11].contains("<redacted>"));
        assert!(!debug_strings[11].contains("secret data"));
        // Decode body is redacted
        assert!(debug_strings[12].contains("<redacted>"));
        assert!(!debug_strings[12].contains("raw payload"));

        // Also verify Display for remaining untested variants
        assert_eq!(
            Error::InvalidAuthConfig {
                field: "client_id",
                message: "empty".into(),
            }
            .to_string(),
            "invalid auth config client_id: empty"
        );
        assert_eq!(
            Error::AuthRequired.to_string(),
            "Schwab authentication is required"
        );
        assert_eq!(
            Error::AuthExpired.to_string(),
            "Schwab authentication is expired"
        );
        assert!(
            Error::AuthCallback("oops".into())
                .to_string()
                .contains("oops")
        );
        assert!(
            Error::Io(std::io::Error::other("disk"))
                .to_string()
                .contains("disk")
        );
        assert!(
            Error::Json(serde_err4)
                .to_string()
                .starts_with("failed to decode Schwab auth JSON:")
        );
        assert!(
            Error::Decode {
                source: serde_json::from_str::<serde_json::Value>("{").unwrap_err(),
                body: "x".into(),
            }
            .to_string()
            .starts_with("failed to decode Schwab response:")
        );
    }
}
