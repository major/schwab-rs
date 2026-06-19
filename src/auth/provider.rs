use super::oauth::refresh_token_file_with_client;
use super::*;

/// Refresh-capable OAuth token provider.
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> schwab::Result<()> {
/// use schwab::auth::{AuthConfig, FileTokenStore, Provider};
///
/// let config = AuthConfig::new(
///     "my-app-key",
///     "my-app-secret",
///     "https://127.0.0.1:8182/callback",
/// )?;
/// let provider = Provider::from_token_file(config, "schwab-token.json")?;
///
/// // Get a valid access token, refreshing automatically if needed.
/// let token = provider.token().await?;
///
/// // Build a ready-to-use API client from the current token.
/// let client = provider.client().await?;
/// # Ok(())
/// # }
/// ```
pub struct Provider {
    pub(super) config: AuthConfig,
    pub(super) store: Arc<dyn TokenStore>,
    http: reqwest::Client,
    refresh_lock: AsyncMutex<()>,
}

impl std::fmt::Debug for Provider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Provider")
            .field("config", &self.config)
            .field("store", &"<TokenStore>")
            .field("http", &"<reqwest::Client>")
            .finish()
    }
}

impl Provider {
    /// Creates a provider from an auth config and token store.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::InvalidAuthConfig`] if the auth configuration is invalid.
    pub fn new<S>(config: AuthConfig, store: S) -> Result<Self>
    where
        S: TokenStore + 'static,
    {
        Self::from_shared_store(config, Arc::new(store), reqwest::Client::new())
    }

    /// Creates a provider backed by a token file.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::InvalidAuthConfig`] if the auth configuration is invalid.
    pub fn from_token_file(config: AuthConfig, token_path: impl Into<PathBuf>) -> Result<Self> {
        Self::new(config, FileTokenStore::new(token_path))
    }

    pub(super) fn from_shared_store(
        config: AuthConfig,
        store: Arc<dyn TokenStore>,
        http: reqwest::Client,
    ) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config,
            store,
            http,
            refresh_lock: AsyncMutex::new(()),
        })
    }

    /// Returns a valid access token, refreshing and persisting it when needed.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::AuthExpired`] if the refresh token has expired locally.
    /// Returns [`crate::Error::RefreshTokenInvalid`] if Schwab rejects the refresh token.
    /// Returns an [`Error`] if the token store cannot be read, the refresh request
    /// fails, or the refreshed token cannot be persisted.
    #[instrument(skip_all)]
    pub async fn token(&self) -> Result<String> {
        let _guard = self.refresh_lock.lock().await;
        let token_file = self.store.load()?;
        let now = current_timestamp()?;
        if !token_file.token.access_token_is_stale(now) {
            return Ok(token_file.token.access_token);
        }
        if token_file.refresh_token_is_stale(now) {
            return Err(Error::AuthExpired);
        }
        let refreshed =
            refresh_token_file_with_client(&self.config, &token_file, &self.http).await?;
        let access_token = refreshed.token.access_token.clone();
        self.store.save(&refreshed)?;
        Ok(access_token)
    }

    /// Forces a refresh, persists the result, and returns the updated token file.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::AuthExpired`] if the refresh token has expired locally.
    /// Returns [`crate::Error::RefreshTokenInvalid`] if Schwab rejects the refresh token.
    /// Returns an [`Error`] if the token store cannot be read, the refresh request
    /// fails, or the refreshed token cannot be persisted.
    #[instrument(skip_all)]
    pub async fn refresh(&self) -> Result<TokenFile> {
        let _guard = self.refresh_lock.lock().await;
        let token_file = self.store.load()?;
        let now = current_timestamp()?;
        if token_file.refresh_token_is_stale(now) {
            return Err(Error::AuthExpired);
        }
        let refreshed =
            refresh_token_file_with_client(&self.config, &token_file, &self.http).await?;
        self.store.save(&refreshed)?;
        Ok(refreshed)
    }

    /// Builds a regular API config from the current access token snapshot.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the access token cannot be obtained.
    #[instrument(skip_all)]
    pub async fn config(&self) -> Result<Config> {
        Ok(Config::new().bearer_token(self.token().await?))
    }

    /// Builds a regular API client from the current access token snapshot.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the access token cannot be obtained.
    #[instrument(skip_all)]
    pub async fn client(&self) -> Result<Client> {
        Ok(Client::new(self.config().await?))
    }

    #[cfg(test)]
    pub(super) fn with_http_client(
        config: AuthConfig,
        store: Arc<dyn TokenStore>,
        http: reqwest::Client,
    ) -> Result<Self> {
        Self::from_shared_store(config, store, http)
    }
}
