//! Stable machine-readable output helpers for CLI failures.

use crate::error::AppError;

/// Stable error payload for machine-readable failures.
#[serde_with::skip_serializing_none]
#[derive(Debug, serde::Serialize)]
pub struct ErrorBody {
    /// Stable error code.
    pub code: &'static str,
    /// Short human-readable error message.
    pub message: String,
    /// Error category for coarse agent decisions.
    pub category: &'static str,
    /// Whether retrying without user action may succeed.
    pub retryable: bool,
    /// Optional remediation hint.
    pub hint: Option<&'static str>,
}

impl From<&AppError> for ErrorBody {
    fn from(error: &AppError) -> Self {
        Self {
            code: error.code(),
            message: error.to_string(),
            category: error.category(),
            retryable: error.retryable(),
            hint: error.hint(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::AppError;
    use crate::output::ErrorBody;

    #[test]
    fn error_body_from_app_error_maps_all_fields() {
        let app_err = AppError::TokenFileMissing("/tmp/token.json".to_string());
        let body = ErrorBody::from(&app_err);

        assert_eq!(body.code, "auth.token_missing");
        assert!(body.message.contains("token file not found"));
        assert_eq!(body.category, "auth");
        assert!(!body.retryable);
        assert!(body.hint.is_some());
    }

    #[test]
    fn error_body_without_hint_omits_hint_in_json() {
        let app_err = AppError::Io(std::io::Error::other("oops"));
        let body = ErrorBody::from(&app_err);
        let serialized = serde_json::to_string(&body).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert!(parsed.get("hint").is_none());
    }
}
