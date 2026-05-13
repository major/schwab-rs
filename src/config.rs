use crate::{Error, Result};

const DEFAULT_MARKET_DATA_BASE_URL: &str = "https://api.schwabapi.com/marketdata/v1";
const DEFAULT_TRADER_BASE_URL: &str = "https://api.schwabapi.com/trader/v1";

/// Configuration used to create a [`crate::Client`].
#[derive(Clone, Eq, PartialEq)]
pub struct Config {
    pub(crate) market_data_base_url: String,
    pub(crate) trader_base_url: String,
    pub(crate) bearer_token: Option<String>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Config")
            .field("market_data_base_url", &self.market_data_base_url)
            .field("trader_base_url", &self.trader_base_url)
            .field(
                "bearer_token",
                &self.bearer_token.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            market_data_base_url: DEFAULT_MARKET_DATA_BASE_URL.to_string(),
            trader_base_url: DEFAULT_TRADER_BASE_URL.to_string(),
            bearer_token: None,
        }
    }
}

impl Config {
    /// Creates a configuration with Schwab's production base URLs.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the Market Data base URL.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::EmptyBaseUrl`] if the URL is empty or
    /// [`crate::Error::InvalidBaseUrl`] if it cannot be parsed.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Result<Self> {
        self.market_data_base_url = normalize_base_url(base_url)?;
        Ok(self)
    }

    /// Overrides the Trader API base URL.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::EmptyBaseUrl`] if the URL is empty or
    /// [`crate::Error::InvalidBaseUrl`] if it cannot be parsed.
    pub fn trader_base_url(mut self, base_url: impl Into<String>) -> Result<Self> {
        self.trader_base_url = normalize_base_url(base_url)?;
        Ok(self)
    }

    /// Adds a bearer token to subsequent requests.
    #[must_use]
    pub fn bearer_token(mut self, bearer_token: impl Into<String>) -> Self {
        let bearer_token = bearer_token.into().trim().to_string();
        self.bearer_token = (!bearer_token.is_empty()).then_some(bearer_token);
        self
    }
}

fn normalize_base_url(base_url: impl Into<String>) -> Result<String> {
    let base_url = base_url.into().trim().trim_end_matches('/').to_string();
    if base_url.trim().is_empty() {
        return Err(Error::EmptyBaseUrl);
    }
    reqwest::Url::parse(&base_url).map_err(|error| Error::InvalidBaseUrl {
        base_url: base_url.clone(),
        message: error.to_string(),
    })?;
    Ok(base_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_base_url() {
        assert!(matches!(
            Config::new().base_url("   "),
            Err(Error::EmptyBaseUrl)
        ));
    }

    #[test]
    fn rejects_invalid_base_urls() {
        assert!(matches!(
            Config::new().base_url("not a url"),
            Err(Error::InvalidBaseUrl { .. })
        ));
        assert!(matches!(
            Config::new().trader_base_url("not a url"),
            Err(Error::InvalidBaseUrl { .. })
        ));
    }
}
