//! OAuth helpers for Schwab's browser login and localhost callback flow.
//!
//! The library owns the security-sensitive pieces of the flow: callback URL
//! validation, CSRF state generation and checking, token exchange, refresh, and
//! private token-file persistence. Browser launching is intentionally adapter
//! driven so callers can choose whether to open a browser or print the URL.
//!
//! # Headless environments
//!
//! When a localhost callback server is impractical (SSH sessions, containers,
//! CI), use [`authorize_url`] to get the login URL, then
//! [`exchange_redirect_url`] with the URL the browser was redirected to after
//! the user authenticates. See [`exchange_redirect_url`] for a full example.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use rand::Rng;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue};
use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as AsyncMutex;

use tracing::instrument;

use crate::{Client, Config, Error, Result};

/// Schwab's production OAuth base URL.
pub const DEFAULT_OAUTH_BASE_URL: &str = "https://api.schwabapi.com/v1/oauth";
const ACCESS_TOKEN_EXPIRY_BUFFER_SECONDS: i64 = 5 * 60;
const REFRESH_TOKEN_MAX_AGE_SECONDS: i64 = 6 * 24 * 60 * 60 + 12 * 60 * 60;
const OAUTH_STATE_BYTES: usize = 32;
const CALLBACK_READ_LIMIT: usize = 8192;
const OAUTH_ERROR_BODY_LIMIT: usize = 1024 * 1024;

/// Schwab OAuth configuration used by the browser login flow.
#[derive(Clone, Eq, PartialEq)]
pub struct AuthConfig {
    client_id: String,
    client_secret: String,
    callback_url: String,
    oauth_base_url: String,
}

impl std::fmt::Debug for AuthConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AuthConfig")
            .field("client_id", &redacted(&self.client_id))
            .field("client_secret", &"<redacted>")
            .field("callback_url", &self.callback_url)
            .field("oauth_base_url", &self.oauth_base_url)
            .finish()
    }
}

impl AuthConfig {
    /// Creates OAuth configuration for Schwab's production OAuth base URL.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::InvalidAuthConfig`] if any field is empty or the callback URL
    /// is not on `127.0.0.1`.
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        callback_url: impl Into<String>,
    ) -> Result<Self> {
        let config = Self {
            client_id: client_id.into().trim().to_string(),
            client_secret: client_secret.into().trim().to_string(),
            callback_url: callback_url.into().trim().to_string(),
            oauth_base_url: DEFAULT_OAUTH_BASE_URL.to_string(),
        };
        config.validate()?;
        Ok(config)
    }

    /// Overrides the OAuth base URL, primarily for tests and Schwab-compatible environments.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::InvalidAuthConfig`] if the resulting configuration is invalid.
    pub fn oauth_base_url(mut self, oauth_base_url: impl Into<String>) -> Result<Self> {
        self.oauth_base_url = oauth_base_url
            .into()
            .trim()
            .trim_end_matches('/')
            .to_string();
        self.validate()?;
        Ok(self)
    }

    /// Returns the Schwab app key used as the OAuth client ID.
    #[must_use]
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Returns the configured callback URL.
    #[must_use]
    pub fn callback_url(&self) -> &str {
        &self.callback_url
    }

    fn validate(&self) -> Result<()> {
        required_auth_text("client_id", &self.client_id)?;
        required_auth_text("client_secret", &self.client_secret)?;
        let callback_url = parse_url("callback_url", &self.callback_url)?;
        validate_callback_url(&callback_url)?;
        let oauth_base_url = parse_url("oauth_base_url", &self.oauth_base_url)?;
        validate_oauth_base_url(&oauth_base_url)?;
        Ok(())
    }

    fn endpoint(&self, path: &str) -> Result<reqwest::Url> {
        let mut url = parse_url("oauth_base_url", &self.oauth_base_url)?;
        url.path_segments_mut()
            .map_err(|()| Error::InvalidAuthConfig {
                field: "oauth_base_url",
                message: "OAuth base URL cannot contain opaque path segments".to_string(),
            })?
            .pop_if_empty()
            .push(path);
        Ok(url)
    }
}

/// Authorization context callers must keep until the callback is received.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthContext {
    /// Callback URL registered with Schwab.
    pub callback_url: String,
    /// Full Schwab authorization URL for the user's browser.
    pub authorization_url: String,
    /// CSRF state that must match the callback.
    pub state: String,
}

/// OAuth token payload returned by Schwab.
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct TokenData {
    /// Current access token used as a bearer token.
    pub access_token: String,
    /// OAuth token type, usually `Bearer`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    /// Access-token lifetime in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<i64>,
    /// Refresh token used to renew access.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Granted OAuth scopes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Epoch seconds when the access token expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
}

impl std::fmt::Debug for TokenData {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TokenData")
            .field("access_token", &redacted(&self.access_token))
            .field("token_type", &self.token_type)
            .field("expires_in", &self.expires_in)
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|value| redacted(value)),
            )
            .field("scope", &self.scope)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

impl TokenData {
    fn with_expires_at(mut self, now: i64) -> Self {
        if self.expires_at.is_none()
            && let Some(expires_in) = self.expires_in
            && expires_in > 0
        {
            self.expires_at = Some(now + expires_in);
        }
        self
    }

    fn access_token_is_stale(&self, now: i64) -> bool {
        self.expires_at
            .map(|expires_at| expires_at <= now + ACCESS_TOKEN_EXPIRY_BUFFER_SECONDS)
            .unwrap_or(false)
    }
}

/// Persisted token wrapper that tracks original refresh-token creation time.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct TokenFile {
    /// Epoch seconds when the refresh token was originally created.
    pub creation_timestamp: i64,
    /// OAuth token payload.
    pub token: TokenData,
}

impl TokenFile {
    fn refresh_token_is_stale(&self, now: i64) -> bool {
        now >= self.creation_timestamp + REFRESH_TOKEN_MAX_AGE_SECONDS
    }
}

/// Storage backend for persisted Schwab OAuth tokens.
pub trait TokenStore: Send + Sync {
    /// Saves a token file.
    fn save(&self, token_file: &TokenFile) -> Result<()>;

    /// Loads a token file.
    fn load(&self) -> Result<TokenFile>;
}

/// JSON token store backed by a file with owner-only permissions on Unix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileTokenStore {
    path: PathBuf,
}

impl FileTokenStore {
    /// Creates a file token store at the provided path.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Returns the token file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl TokenStore for FileTokenStore {
    fn save(&self, token_file: &TokenFile) -> Result<()> {
        if let Some(parent) = real_parent(&self.path) {
            fs::create_dir_all(parent).map_err(Error::Io)?;
            set_private_dir_permissions(parent)?;
        }

        let temp_path = self.path.with_extension("tmp");
        let _ = fs::remove_file(&temp_path);
        let encoded = serde_json::to_vec_pretty(token_file).map_err(Error::Encode)?;
        let mut temp_file = private_file(&temp_path)?;
        temp_file.write_all(&encoded).map_err(Error::Io)?;
        temp_file.write_all(b"\n").map_err(Error::Io)?;
        temp_file.sync_all().map_err(Error::Io)?;
        drop(temp_file);
        fs::rename(&temp_path, &self.path).map_err(Error::Io)?;
        sync_parent_dir(&self.path)?;
        Ok(())
    }

    fn load(&self) -> Result<TokenFile> {
        let contents = fs::read_to_string(&self.path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                Error::AuthRequired
            } else {
                Error::Io(error)
            }
        })?;
        serde_json::from_str(&contents).map_err(Error::Json)
    }
}

/// In-memory token store useful for tests and short-lived tools.
#[derive(Debug, Default)]
pub struct MemoryTokenStore {
    token_file: Mutex<Option<TokenFile>>,
}

impl MemoryTokenStore {
    /// Creates an empty memory token store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl TokenStore for MemoryTokenStore {
    fn save(&self, token_file: &TokenFile) -> Result<()> {
        let mut guard = self
            .token_file
            .lock()
            .map_err(|_| Error::AuthCallback("memory token store lock poisoned".to_string()))?;
        *guard = Some(token_file.clone());
        Ok(())
    }

    fn load(&self) -> Result<TokenFile> {
        self.token_file
            .lock()
            .map_err(|_| Error::AuthCallback("memory token store lock poisoned".to_string()))?
            .clone()
            .ok_or(Error::AuthRequired)
    }
}

/// Refresh-capable OAuth token provider.
pub struct Provider {
    config: AuthConfig,
    store: Arc<dyn TokenStore>,
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

    fn from_shared_store(
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
    /// Returns [`crate::Error::AuthExpired`] if the refresh token has expired.
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
    /// Returns [`crate::Error::AuthExpired`] if the refresh token has expired.
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
    fn with_http_client(
        config: AuthConfig,
        store: Arc<dyn TokenStore>,
        http: reqwest::Client,
    ) -> Result<Self> {
        Self::from_shared_store(config, store, http)
    }
}

/// Builds a Schwab authorization URL and CSRF state for the browser flow.
///
/// # Errors
///
/// Returns [`crate::Error::InvalidAuthConfig`] if the auth configuration is invalid.
pub fn authorize_url(config: &AuthConfig) -> Result<AuthContext> {
    authorize_url_with_state(config, &random_oauth_state()?)
}

fn authorize_url_with_state(config: &AuthConfig, state: &str) -> Result<AuthContext> {
    config.validate()?;
    let mut url = config.endpoint("authorize")?;
    url.query_pairs_mut()
        .clear()
        .append_pair("response_type", "code")
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", &config.callback_url)
        .append_pair("state", state);
    Ok(AuthContext {
        callback_url: config.callback_url.clone(),
        authorization_url: url.to_string(),
        state: state.to_string(),
    })
}

/// Exchanges an authorization code for an initial token file.
///
/// # Errors
///
/// Returns an [`Error`] if the code is empty, the token request fails, or the
/// response cannot be decoded.
#[instrument(skip_all)]
pub async fn exchange_code(config: &AuthConfig, code: &str) -> Result<TokenFile> {
    exchange_code_with_client(config, code, &reqwest::Client::new()).await
}

async fn exchange_code_with_client(
    config: &AuthConfig,
    code: &str,
    http: &reqwest::Client,
) -> Result<TokenFile> {
    required_auth_text("code", code)?;
    let now = current_timestamp()?;
    let token = token_request(
        config,
        &[
            ("grant_type", "authorization_code"),
            ("code", code.trim()),
            ("redirect_uri", &config.callback_url),
        ],
        http,
    )
    .await?
    .with_expires_at(now);
    Ok(TokenFile {
        creation_timestamp: now,
        token,
    })
}

/// Extracts an authorization code and CSRF state from a redirect URL.
///
/// This is the headless counterpart to `CallbackServer`. After calling
/// [`authorize_url`], direct the user to open the authorization URL in a
/// browser. When Schwab redirects, the localhost callback server will not be
/// running, so the browser shows an error, but the address bar contains the
/// code and state parameters. The user copies that URL and passes it here.
///
/// Returns an error if the URL is malformed, carries an OAuth error, is
/// missing the `code` or `state` parameters, or the `state` does not match
/// the [`AuthContext`] (CSRF check).
pub fn parse_redirect_url(
    auth_context: &AuthContext,
    redirect_url: &str,
) -> Result<CallbackResult> {
    let url = reqwest::Url::parse(redirect_url.trim())
        .map_err(|error| Error::AuthCallback(format!("invalid redirect URL: {error}")))?;
    let mut code = None;
    let mut state = None;
    let mut oauth_error = None;
    let mut oauth_error_description = None;
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            "error" => oauth_error = Some(value.into_owned()),
            "error_description" => oauth_error_description = Some(value.into_owned()),
            _ => {}
        }
    }
    if let Some(error) = oauth_error {
        if let Some(description) = oauth_error_description {
            return Err(Error::AuthCallback(format!("{error}: {description}")));
        }
        return Err(Error::AuthCallback(error));
    }
    let result = CallbackResult {
        code: code.ok_or_else(|| {
            Error::AuthCallback("missing authorization code in redirect URL".to_string())
        })?,
        state: state
            .ok_or_else(|| Error::AuthCallback("missing state in redirect URL".to_string()))?,
    };
    if result.state != auth_context.state {
        return Err(Error::AuthCallback(
            "state mismatch in redirect URL".to_string(),
        ));
    }
    Ok(result)
}

/// Completes the login flow from a pasted redirect URL.
///
/// This is the headless alternative to [`login`]. Call [`authorize_url`]
/// first, present the authorization URL to the user (print it, email it,
/// etc.), then call this function with the full redirect URL the user copies
/// from their browser's address bar after authenticating.
///
/// # Example
///
/// ```no_run
/// use schwab::auth::{self, AuthConfig, FileTokenStore};
///
/// # async fn run() -> schwab::Result<()> {
/// let config = AuthConfig::new("APP_KEY", "APP_SECRET", "https://127.0.0.1:8182/callback")?;
/// let context = auth::authorize_url(&config)?;
///
/// // In a real application, print the URL and read stdin.
/// println!("Open this URL in a browser:\n{}", context.authorization_url);
/// let redirect_url = "https://127.0.0.1:8182/callback?code=CODE&state=...";
///
/// let provider = auth::exchange_redirect_url(
///     config,
///     FileTokenStore::new("token.json"),
///     &context,
///     redirect_url,
/// )
/// .await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`crate::Error::AuthCallback`] if the redirect URL is malformed or fails
/// the CSRF state check. Returns an [`Error`] if the token exchange fails.
#[instrument(skip_all)]
pub async fn exchange_redirect_url<S>(
    config: AuthConfig,
    store: S,
    auth_context: &AuthContext,
    redirect_url: &str,
) -> Result<Provider>
where
    S: TokenStore + 'static,
{
    let result = parse_redirect_url(auth_context, redirect_url)?;
    let store = Arc::new(store);
    let http = reqwest::Client::new();
    let token_file = exchange_code_with_client(&config, &result.code, &http).await?;
    store.save(&token_file)?;
    Provider::from_shared_store(config, store, http)
}

/// Refreshes a token file while preserving the original creation timestamp.
///
/// # Errors
///
/// Returns [`crate::Error::AuthExpired`] if the refresh token is missing or has expired.
/// Returns an [`Error`] if the refresh request fails or the response cannot be decoded.
#[instrument(skip_all)]
pub async fn refresh_token_file(config: &AuthConfig, token_file: &TokenFile) -> Result<TokenFile> {
    refresh_token_file_with_client(config, token_file, &reqwest::Client::new()).await
}

async fn refresh_token_file_with_client(
    config: &AuthConfig,
    token_file: &TokenFile,
    http: &reqwest::Client,
) -> Result<TokenFile> {
    let refresh_token = token_file
        .token
        .refresh_token
        .as_deref()
        .ok_or(Error::AuthExpired)?;
    required_auth_text("refresh_token", refresh_token)?;
    let now = current_timestamp()?;
    if token_file.refresh_token_is_stale(now) {
        return Err(Error::AuthExpired);
    }
    let mut token = token_request(
        config,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ],
        http,
    )
    .await?
    .with_expires_at(now);
    if token.refresh_token.is_none() {
        token.refresh_token = token_file.token.refresh_token.clone();
    }
    Ok(TokenFile {
        creation_timestamp: token_file.creation_timestamp,
        token,
    })
}

async fn token_request(
    config: &AuthConfig,
    form: &[(&str, &str)],
    http: &reqwest::Client,
) -> Result<TokenData> {
    config.validate()?;
    let response = http
        .post(config.endpoint("token")?)
        .header(AUTHORIZATION, basic_auth_header(config)?)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(form)
        .send()
        .await
        .map_err(Error::Request)?;

    let status = response.status();
    if !status.is_success() {
        let bytes = response.bytes().await.map_err(Error::Request)?;
        let body =
            String::from_utf8_lossy(&bytes[..bytes.len().min(OAUTH_ERROR_BODY_LIMIT)]).into_owned();
        if status.as_u16() == 400 && body.contains("invalid_grant") {
            return Err(Error::AuthExpired);
        }
        return Err(Error::HttpStatus {
            status: status.as_u16(),
            body,
        });
    }
    let text = response.text().await.map_err(Error::Request)?;
    serde_json::from_str::<TokenData>(&text).map_err(|source| Error::Decode { source, body: text })
}

/// Starts the full login flow and calls `url_handler` with the authorization URL.
///
/// # Errors
///
/// Returns an [`Error`] if the callback listener fails to start, `url_handler`
/// returns an error, or the token exchange fails.
#[instrument(skip_all)]
pub async fn login<S, F>(config: AuthConfig, store: S, url_handler: F) -> Result<Provider>
where
    S: TokenStore + 'static,
    F: FnOnce(&str) -> Result<()>,
{
    let session = start_login(config, store)?;
    url_handler(&session.auth_context.authorization_url)?;
    session.wait().await
}

/// Starts the callback listener and returns a one-shot login session.
///
/// # Errors
///
/// Returns [`crate::Error::InvalidAuthConfig`] if the auth configuration is invalid.
/// Returns [`crate::Error::AuthCallback`] if the callback listener fails to bind.
pub fn start_login<S>(config: AuthConfig, store: S) -> Result<LoginSession>
where
    S: TokenStore + 'static,
{
    let store = Arc::new(store);
    let auth_context = authorize_url(&config)?;
    let callback_server = CallbackServer::start(&config.callback_url)?;
    Ok(LoginSession {
        config,
        store,
        auth_context,
        callback_server,
        http: reqwest::Client::new(),
        timeout: Some(Duration::from_mins(5)),
    })
}

/// One-shot browser login session returned by [`start_login`].
pub struct LoginSession {
    config: AuthConfig,
    store: Arc<dyn TokenStore>,
    auth_context: AuthContext,
    callback_server: CallbackServer,
    http: reqwest::Client,
    timeout: Option<Duration>,
}

impl LoginSession {
    /// Returns the authorization context for this session.
    #[must_use]
    pub fn auth_context(&self) -> &AuthContext {
        &self.auth_context
    }

    /// Sets the maximum time to wait for the localhost callback.
    #[must_use]
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Waits for the callback, exchanges the code, saves the token, and returns a provider.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::AuthCallback`] if the callback times out, the state does not
    /// match, or the listener thread panics. Returns an [`Error`] if the token
    /// exchange or persistence fails.
    #[instrument(skip_all)]
    pub async fn wait(self) -> Result<Provider> {
        let LoginSession {
            config,
            store,
            auth_context,
            callback_server,
            http,
            timeout,
        } = self;
        let expected_state = auth_context.state;
        let result = tokio::task::spawn_blocking(move || callback_server.wait(timeout))
            .await
            .map_err(|error| {
                Error::AuthCallback(format!("callback wait task failed: {error}"))
            })??;
        if result.state != expected_state {
            return Err(Error::AuthCallback("state mismatch".to_string()));
        }
        let token_file = exchange_code_with_client(&config, &result.code, &http).await?;
        store.save(&token_file)?;
        Provider::from_shared_store(config, store, http)
    }
}

/// Authorization response extracted from the localhost callback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallbackResult {
    /// Authorization code returned by Schwab.
    pub code: String,
    /// State returned by Schwab.
    pub state: String,
}

struct CallbackServer {
    result_rx: mpsc::Receiver<Result<CallbackResult>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl CallbackServer {
    fn start(callback_url: &str) -> Result<Self> {
        let parsed = parse_url("callback_url", callback_url)?;
        validate_callback_url(&parsed)?;
        let port = parsed.port().ok_or_else(|| Error::InvalidAuthConfig {
            field: "callback_url",
            message: "callback URL must include an explicit port".to_string(),
        })?;
        let path = callback_path(&parsed);
        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
            .map_err(Error::Io)?;
        listener.set_nonblocking(true).map_err(Error::Io)?;
        let tls_config = Arc::new(callback_tls_config()?);
        let (result_tx, result_rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            callback_loop(listener, tls_config, path, result_tx, shutdown_rx);
        });

        Ok(Self {
            result_rx,
            shutdown_tx: Some(shutdown_tx),
            handle: Some(handle),
        })
    }

    fn wait(mut self, timeout: Option<Duration>) -> Result<CallbackResult> {
        let result = match timeout {
            Some(timeout) => self
                .result_rx
                .recv_timeout(timeout)
                .map_err(|_| Error::AuthCallback("timed out waiting for callback".to_string()))?,
            None => self
                .result_rx
                .recv()
                .map_err(|_| Error::AuthCallback("callback server exited".to_string()))?,
        };
        self.shutdown();
        result
    }

    fn shutdown(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for CallbackServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn callback_loop(
    listener: TcpListener,
    tls_config: Arc<ServerConfig>,
    callback_path: String,
    result_tx: mpsc::Sender<Result<CallbackResult>>,
    shutdown_rx: mpsc::Receiver<()>,
) {
    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }
        match listener.accept() {
            Ok((stream, _)) => {
                let result = handle_callback_stream(stream, tls_config.clone(), &callback_path);
                let _ = result_tx.send(result);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => {
                let _ = result_tx.send(Err(Error::Io(error)));
                break;
            }
        }
    }
}

fn handle_callback_stream(
    stream: TcpStream,
    tls_config: Arc<ServerConfig>,
    callback_path: &str,
) -> Result<CallbackResult> {
    // The listener is non-blocking for cancellation support, but on macOS the
    // accepted stream inherits that mode. TLS requires blocking I/O.
    stream.set_nonblocking(false).map_err(Error::Io)?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(Error::Io)?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(Error::Io)?;
    let connection = ServerConnection::new(tls_config).map_err(|error| {
        Error::AuthCallback(format!("failed to create TLS connection: {error}"))
    })?;
    let mut stream = StreamOwned::new(connection, stream);
    let mut buffer = vec![0; CALLBACK_READ_LIMIT];
    let bytes_read = stream.read(&mut buffer).map_err(Error::Io)?;
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let result = parse_callback_request(&request, callback_path);
    let response = match &result {
        Ok(_) => http_response("200 OK", "Login successful. You can close this tab."),
        Err(error) => http_response("400 Bad Request", &error.to_string()),
    };
    stream.write_all(response.as_bytes()).map_err(Error::Io)?;
    stream.flush().map_err(Error::Io)?;
    result
}

fn parse_callback_request(request: &str, callback_path: &str) -> Result<CallbackResult> {
    let request_line = request
        .lines()
        .next()
        .ok_or_else(|| Error::AuthCallback("empty callback request".to_string()))?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    if method != "GET" {
        return Err(Error::AuthCallback(
            "callback request must use GET".to_string(),
        ));
    }
    let url = reqwest::Url::parse(&format!("https://127.0.0.1{target}")).map_err(|error| {
        Error::AuthCallback(format!("invalid callback request target: {error}"))
    })?;
    if url.path() != callback_path {
        return Err(Error::AuthCallback(format!(
            "unexpected callback path {:?}",
            url.path()
        )));
    }
    let mut code = None;
    let mut state = None;
    let mut oauth_error = None;
    let mut oauth_error_description = None;
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            "error" => oauth_error = Some(value.into_owned()),
            "error_description" => oauth_error_description = Some(value.into_owned()),
            _ => {}
        }
    }
    if let Some(error) = oauth_error {
        if let Some(description) = oauth_error_description {
            return Err(Error::AuthCallback(format!("{error}: {description}")));
        }
        return Err(Error::AuthCallback(error));
    }
    Ok(CallbackResult {
        code: code.ok_or_else(|| Error::AuthCallback("missing authorization code".to_string()))?,
        state: state.ok_or_else(|| Error::AuthCallback("missing state".to_string()))?,
    })
}

fn callback_tls_config() -> Result<ServerConfig> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let certificate =
        rcgen::generate_simple_self_signed(vec!["127.0.0.1".to_string(), "localhost".to_string()])
            .map_err(|error| {
                Error::AuthCallback(format!("failed to generate callback TLS cert: {error}"))
            })?;
    let cert_der = certificate.cert.der().clone();
    let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        certificate.signing_key.serialize_der(),
    ));
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .map_err(|error| Error::AuthCallback(format!("invalid callback TLS cert: {error}")))
}

fn validate_callback_url(url: &reqwest::Url) -> Result<()> {
    if url.scheme() != "https" {
        return Err(Error::InvalidAuthConfig {
            field: "callback_url",
            message: "callback URL must use https".to_string(),
        });
    }
    if url.host_str() != Some("127.0.0.1") {
        return Err(Error::InvalidAuthConfig {
            field: "callback_url",
            message: "callback URL host must be exactly 127.0.0.1".to_string(),
        });
    }
    if url.port().is_none() {
        return Err(Error::InvalidAuthConfig {
            field: "callback_url",
            message: "callback URL must include an explicit port".to_string(),
        });
    }
    Ok(())
}

fn validate_oauth_base_url(url: &reqwest::Url) -> Result<()> {
    if url.scheme() == "https" {
        return Ok(());
    }
    if cfg!(test) && url.scheme() == "http" && url.host_str() == Some("127.0.0.1") {
        return Ok(());
    }
    Err(Error::InvalidAuthConfig {
        field: "oauth_base_url",
        message: "OAuth base URL must use https".to_string(),
    })
}

fn callback_path(url: &reqwest::Url) -> String {
    let path = url.path();
    if path.is_empty() {
        "/".to_string()
    } else {
        path.to_string()
    }
}

fn required_auth_text<'a>(field: &'static str, value: &'a str) -> Result<&'a str> {
    let value = value.trim();
    if value.is_empty() {
        return Err(Error::InvalidAuthConfig {
            field,
            message: "value cannot be empty".to_string(),
        });
    }
    Ok(value)
}

fn parse_url(field: &'static str, value: &str) -> Result<reqwest::Url> {
    reqwest::Url::parse(required_auth_text(field, value)?).map_err(|error| {
        Error::InvalidAuthConfig {
            field,
            message: error.to_string(),
        }
    })
}

fn random_oauth_state() -> Result<String> {
    use std::fmt::Write;
    let mut bytes = [0_u8; OAUTH_STATE_BYTES];
    rand::rng().fill_bytes(&mut bytes);
    let mut hex = String::with_capacity(OAUTH_STATE_BYTES * 2);
    for byte in bytes {
        write!(hex, "{byte:02x}").expect("writing to String never fails");
    }
    Ok(hex)
}

fn basic_auth_header(config: &AuthConfig) -> Result<HeaderValue> {
    let username = urlencoding::encode(&config.client_id);
    let password = urlencoding::encode(&config.client_secret);
    let encoded =
        base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
    HeaderValue::from_str(&format!("Basic {encoded}"))
        .map_err(|error| Error::AuthCallback(error.to_string()))
}

fn current_timestamp() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            Error::AuthCallback(format!("system time is before UNIX epoch: {error}"))
        })?;
    i64::try_from(duration.as_secs())
        .map_err(|error| Error::AuthCallback(format!("timestamp overflow: {error}")))
}

fn http_response(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\ncontent-type: text/plain; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn redacted(value: &str) -> &'static str {
    if value.is_empty() { "" } else { "<redacted>" }
}

fn private_file(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).map_err(Error::Io)
}

fn real_parent(path: &Path) -> Option<&Path> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
}

fn set_private_dir_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(Error::Io)?;
    }
    Ok(())
}

fn sync_parent_dir(path: &Path) -> Result<()> {
    #[cfg(not(windows))]
    {
        if let Some(parent) = real_parent(path) {
            File::open(parent)
                .and_then(|file| file.sync_all())
                .map_err(Error::Io)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use mockito::Matcher;

    use super::*;
    use crate::test_support::fixture;

    #[test]
    fn auth_config_rejects_insecure_or_non_loopback_callbacks() {
        assert!(matches!(
            AuthConfig::new("client", "secret", "http://127.0.0.1:8182/callback"),
            Err(Error::InvalidAuthConfig {
                field: "callback_url",
                ..
            })
        ));
        assert!(matches!(
            AuthConfig::new("client", "secret", "https://localhost:8182/callback"),
            Err(Error::InvalidAuthConfig {
                field: "callback_url",
                ..
            })
        ));
        assert!(matches!(
            AuthConfig::new("client", "secret", "https://127.0.0.1/callback"),
            Err(Error::InvalidAuthConfig {
                field: "callback_url",
                ..
            })
        ));
    }

    #[test]
    fn authorize_url_contains_schwab_oauth_parameters() {
        let config =
            AuthConfig::new("client-id", "secret", "https://127.0.0.1:8182/callback").unwrap();
        let context = authorize_url(&config).unwrap();

        assert_eq!(context.callback_url, "https://127.0.0.1:8182/callback");
        assert_eq!(context.state.len(), OAUTH_STATE_BYTES * 2);
        let parsed = reqwest::Url::parse(&context.authorization_url).unwrap();
        let pairs: Vec<_> = parsed.query_pairs().collect();
        assert!(pairs.contains(&("response_type".into(), "code".into())));
        assert!(pairs.contains(&("client_id".into(), "client-id".into())));
        assert!(pairs.contains(&(
            "redirect_uri".into(),
            "https://127.0.0.1:8182/callback".into()
        )));
        assert!(
            pairs
                .iter()
                .any(|(key, value)| key == "state" && value == &context.state)
        );
    }

    #[test]
    fn file_token_store_round_trips_metadata_with_private_permissions() {
        let path = unique_test_path("tokens.json");
        let store = FileTokenStore::new(&path);
        let token_file = token_file("ACCESS", "REFRESH", current_timestamp().unwrap() + 3600);

        store.save(&token_file).unwrap();
        assert_eq!(store.load().unwrap(), token_file);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let file_mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            let dir_mode = fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(file_mode, 0o600);
            assert_eq!(dir_mode, 0o700);
        }
    }

    #[test]
    fn file_token_store_supports_bare_relative_paths() {
        let current_dir = std::env::current_dir().unwrap();
        let test_dir = std::env::temp_dir().join(format!(
            "schwab-rs-auth-relative-{}",
            current_timestamp().unwrap()
        ));
        fs::create_dir_all(&test_dir).unwrap();
        std::env::set_current_dir(&test_dir).unwrap();

        let result = (|| {
            let store = FileTokenStore::new("schwab-token.json");
            let token_file = token_file("ACCESS", "REFRESH", current_timestamp().unwrap() + 3600);

            store.save(&token_file)?;
            store.load()
        })();

        std::env::set_current_dir(current_dir).unwrap();
        fs::remove_dir_all(&test_dir).unwrap();
        assert_eq!(result.unwrap().token.access_token, "ACCESS");
    }

    #[test]
    fn memory_token_store_reports_auth_required_when_empty() {
        let store = MemoryTokenStore::new();

        assert!(matches!(store.load(), Err(Error::AuthRequired)));
    }

    #[tokio::test]
    async fn refresh_token_file_preserves_timestamp_and_sends_expected_request() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/token")
            .match_header("authorization", Matcher::Regex("^Basic .+$".into()))
            .match_body(Matcher::AllOf(vec![
                Matcher::Regex("grant_type=refresh_token".into()),
                Matcher::Regex("refresh_token=REFRESH1".into()),
            ]))
            .with_status(200)
            .with_body(fixture("token_response.json"))
            .create_async()
            .await;

        let url = server.url();
        let config = test_config(&url);
        let original_timestamp = current_timestamp().unwrap() - 60;
        let token_file = TokenFile {
            creation_timestamp: original_timestamp,
            token: TokenData {
                access_token: "OLD".to_string(),
                token_type: Some("Bearer".to_string()),
                expires_in: Some(1800),
                refresh_token: Some("REFRESH1".to_string()),
                scope: None,
                expires_at: Some(current_timestamp().unwrap() - 1),
            },
        };

        let refreshed = refresh_token_file(&config, &token_file).await.unwrap();

        mock.assert_async().await;
        assert_eq!(refreshed.creation_timestamp, original_timestamp);
        assert_eq!(refreshed.token.access_token, "NEW");
        assert!(refreshed.token.expires_at.unwrap() > current_timestamp().unwrap());
    }

    #[tokio::test]
    async fn refresh_token_file_preserves_refresh_token_when_response_omits_it() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/token")
            .with_status(200)
            .with_body(fixture("token_response_no_refresh.json"))
            .create_async()
            .await;

        let url = server.url();
        let config = test_config(&url);
        let original = token_file("OLD", "REFRESH1", current_timestamp().unwrap() - 1);

        let refreshed = refresh_token_file(&config, &original).await.unwrap();

        assert_eq!(refreshed.token.access_token, "NEW");
        assert_eq!(refreshed.token.refresh_token.as_deref(), Some("REFRESH1"));
    }

    #[tokio::test]
    async fn provider_refreshes_expired_token_and_saves_result() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/token")
            .match_body(Matcher::AllOf(vec![
                Matcher::Regex("grant_type=refresh_token".into()),
                Matcher::Regex("refresh_token=REFRESH1".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"access_token":"NEW","refresh_token":"REFRESH2","expires_in":1800}"#)
            .create_async()
            .await;

        let url = server.url();
        let config = test_config(&url);
        let store: Arc<dyn TokenStore> = Arc::new(MemoryTokenStore::new());
        store
            .save(&token_file(
                "OLD",
                "REFRESH1",
                current_timestamp().unwrap() - 10,
            ))
            .unwrap();
        let provider =
            Provider::with_http_client(config, store.clone(), reqwest::Client::new()).unwrap();

        let access_token = provider.token().await.unwrap();
        let saved = store.load().unwrap();

        mock.assert_async().await;
        assert_eq!(access_token, "NEW");
        assert_eq!(saved.token.access_token, "NEW");
    }

    #[tokio::test]
    async fn callback_server_receives_https_code_and_state() {
        let port = unused_loopback_port();
        let server = CallbackServer::start(&format!("https://127.0.0.1:{port}/callback")).unwrap();
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        let response = client
            .get(format!(
                "https://127.0.0.1:{port}/callback?code=CODE&state=STATE"
            ))
            .send()
            .await
            .unwrap();
        let result = server.wait(Some(Duration::from_secs(2))).unwrap();

        assert!(response.status().is_success());
        assert_eq!(
            result,
            CallbackResult {
                code: "CODE".to_string(),
                state: "STATE".to_string(),
            }
        );
    }

    #[test]
    fn callback_error_includes_oauth_error_description() {
        let request =
            "GET /callback?error=access_denied&error_description=User%20cancelled HTTP/1.1\r\n\r\n";

        let error = parse_callback_request(request, "/callback").unwrap_err();

        assert_eq!(
            error.to_string(),
            "Schwab auth callback failed: access_denied: User cancelled"
        );
    }

    fn test_config(oauth_base_url: &str) -> AuthConfig {
        AuthConfig {
            client_id: "client id".to_string(),
            client_secret: "secret/value".to_string(),
            callback_url: "https://127.0.0.1:8182/callback".to_string(),
            oauth_base_url: oauth_base_url.to_string(),
        }
    }

    fn token_file(access_token: &str, refresh_token: &str, expires_at: i64) -> TokenFile {
        TokenFile {
            creation_timestamp: current_timestamp().unwrap() - 60,
            token: TokenData {
                access_token: access_token.to_string(),
                token_type: Some("Bearer".to_string()),
                expires_in: Some(1800),
                refresh_token: Some(refresh_token.to_string()),
                scope: Some("readonly".to_string()),
                expires_at: Some(expires_at),
            },
        }
    }

    fn unique_test_path(filename: &str) -> PathBuf {
        std::env::temp_dir()
            .join("schwab-rs-auth-tests")
            .join(format!("{}-{filename}", current_timestamp().unwrap()))
    }

    #[test]
    fn parse_redirect_url_extracts_code_and_state() {
        let context = auth_context("STATE42");
        let url = "https://127.0.0.1:8182/callback?code=AUTH_CODE&state=STATE42";

        let result = parse_redirect_url(&context, url).unwrap();

        assert_eq!(result.code, "AUTH_CODE");
        assert_eq!(result.state, "STATE42");
    }

    #[test]
    fn parse_redirect_url_rejects_state_mismatch() {
        let context = auth_context("EXPECTED");
        let url = "https://127.0.0.1:8182/callback?code=AUTH_CODE&state=WRONG";

        let error = parse_redirect_url(&context, url).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Schwab auth callback failed: state mismatch in redirect URL"
        );
    }

    #[test]
    fn parse_redirect_url_rejects_missing_code() {
        let context = auth_context("STATE42");
        let url = "https://127.0.0.1:8182/callback?state=STATE42";

        let error = parse_redirect_url(&context, url).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Schwab auth callback failed: missing authorization code in redirect URL"
        );
    }

    #[test]
    fn parse_redirect_url_surfaces_oauth_error() {
        let context = auth_context("STATE42");
        let url = "https://127.0.0.1:8182/callback?error=access_denied&error_description=User%20cancelled&state=STATE42";

        let error = parse_redirect_url(&context, url).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Schwab auth callback failed: access_denied: User cancelled"
        );
    }

    #[tokio::test]
    async fn exchange_redirect_url_completes_headless_login() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/token")
            .match_body(Matcher::AllOf(vec![
                Matcher::Regex("grant_type=authorization_code".into()),
                Matcher::Regex("code=AUTH_CODE".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{"access_token":"HEADLESS_TOKEN","refresh_token":"REFRESH","expires_in":1800,"token_type":"Bearer"}"#,
            )
            .create_async()
            .await;

        let url = server.url();
        let config = test_config(&url);
        let context = authorize_url_with_state(&config, "STATE42").unwrap();
        let redirect_url = "https://127.0.0.1:8182/callback?code=AUTH_CODE&state=STATE42";

        let provider =
            exchange_redirect_url(config, MemoryTokenStore::new(), &context, redirect_url)
                .await
                .unwrap();

        mock.assert_async().await;
        assert_eq!(
            provider.store.load().unwrap().token.access_token,
            "HEADLESS_TOKEN"
        );
    }

    fn auth_context(state: &str) -> AuthContext {
        AuthContext {
            callback_url: "https://127.0.0.1:8182/callback".to_string(),
            authorization_url: format!(
                "https://api.schwabapi.com/v1/oauth/authorize?response_type=code&client_id=test&redirect_uri=https%3A%2F%2F127.0.0.1%3A8182%2Fcallback&state={state}"
            ),
            state: state.to_string(),
        }
    }

    fn unused_loopback_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    }

    #[test]
    fn auth_config_debug_redacts_credentials() {
        let config = AuthConfig::new(
            "my-app-key",
            "super-secret",
            "https://127.0.0.1:8182/callback",
        )
        .unwrap();
        let debug = format!("{config:?}");

        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("super-secret"));
        assert!(!debug.contains("my-app-key"));
        assert!(debug.contains("https://127.0.0.1:8182/callback"));
    }

    #[test]
    fn auth_config_oauth_base_url_setter_overrides_default() {
        let config =
            AuthConfig::new("client", "secret", "https://127.0.0.1:8182/callback").unwrap();
        assert_eq!(config.oauth_base_url, DEFAULT_OAUTH_BASE_URL);

        let custom = config
            .oauth_base_url("https://custom.example.com/oauth/")
            .unwrap();
        assert_eq!(custom.oauth_base_url, "https://custom.example.com/oauth");
    }

    #[test]
    fn auth_config_accessors_return_expected_values() {
        let config =
            AuthConfig::new("my-client-id", "secret", "https://127.0.0.1:8182/callback").unwrap();

        assert_eq!(config.client_id(), "my-client-id");
        assert_eq!(config.callback_url(), "https://127.0.0.1:8182/callback");
    }

    #[test]
    fn token_data_debug_redacts_tokens() {
        let token = TokenData {
            access_token: "secret-access".to_string(),
            token_type: Some("Bearer".to_string()),
            expires_in: Some(1800),
            refresh_token: Some("secret-refresh".to_string()),
            scope: Some("readonly".to_string()),
            expires_at: Some(999_999),
        };
        let debug = format!("{token:?}");

        assert!(!debug.contains("secret-access"));
        assert!(!debug.contains("secret-refresh"));
        assert!(debug.contains("<redacted>"));
        assert!(debug.contains("Bearer"));
    }

    #[test]
    fn file_token_store_path_accessor() {
        let store = FileTokenStore::new("/tmp/schwab-token.json");
        assert_eq!(store.path(), Path::new("/tmp/schwab-token.json"));
    }

    #[test]
    fn file_token_store_load_missing_file_returns_auth_required() {
        let store = FileTokenStore::new("/tmp/schwab-rs-nonexistent-token.json");
        assert!(matches!(store.load(), Err(Error::AuthRequired)));
    }

    #[test]
    fn redacted_returns_placeholder_or_empty() {
        assert_eq!(redacted("something"), "<redacted>");
        assert_eq!(redacted(""), "");
    }

    #[test]
    fn parse_redirect_url_surfaces_oauth_error_without_description() {
        let context = auth_context("STATE42");
        let url = "https://127.0.0.1:8182/callback?error=server_error&state=STATE42";

        let error = parse_redirect_url(&context, url).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Schwab auth callback failed: server_error"
        );
    }

    #[tokio::test]
    async fn provider_debug_redacts_internals() {
        let config = test_config("http://127.0.0.1:9999");
        let store: Arc<dyn TokenStore> = Arc::new(MemoryTokenStore::new());
        let provider = Provider::with_http_client(config, store, reqwest::Client::new()).unwrap();
        let debug = format!("{provider:?}");

        assert!(debug.contains("Provider"));
        assert!(debug.contains("<TokenStore>"));
        assert!(debug.contains("<reqwest::Client>"));
    }

    #[tokio::test]
    async fn provider_new_and_from_token_file_create_valid_instances() {
        let config =
            AuthConfig::new("client", "secret", "https://127.0.0.1:8182/callback").unwrap();
        let provider = Provider::new(config.clone(), MemoryTokenStore::new()).unwrap();
        let debug = format!("{provider:?}");
        assert!(debug.contains("Provider"));

        let path = unique_test_path("provider-token.json");
        let provider2 = Provider::from_token_file(config, &path).unwrap();
        let debug2 = format!("{provider2:?}");
        assert!(debug2.contains("Provider"));
    }

    #[tokio::test]
    async fn provider_refresh_returns_updated_token() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/token")
            .match_body(Matcher::AllOf(vec![
                Matcher::Regex("grant_type=refresh_token".into()),
                Matcher::Regex("refresh_token=REFRESH1".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{"access_token":"REFRESHED","refresh_token":"REFRESH2","expires_in":1800}"#,
            )
            .create_async()
            .await;

        let url = server.url();
        let config = test_config(&url);
        let store: Arc<dyn TokenStore> = Arc::new(MemoryTokenStore::new());
        store
            .save(&token_file(
                "OLD",
                "REFRESH1",
                current_timestamp().unwrap() - 10,
            ))
            .unwrap();
        let provider =
            Provider::with_http_client(config, store.clone(), reqwest::Client::new()).unwrap();

        let refreshed = provider.refresh().await.unwrap();

        mock.assert_async().await;
        assert_eq!(refreshed.token.access_token, "REFRESHED");
        assert_eq!(store.load().unwrap().token.access_token, "REFRESHED");
    }

    #[tokio::test]
    async fn provider_config_and_client_return_valid_objects() {
        let url = "http://127.0.0.1:9999";
        let config = test_config(url);
        let store: Arc<dyn TokenStore> = Arc::new(MemoryTokenStore::new());
        store
            .save(&token_file(
                "FRESH",
                "REFRESH",
                current_timestamp().unwrap() + 3600,
            ))
            .unwrap();
        let provider = Provider::with_http_client(config, store, reqwest::Client::new()).unwrap();

        let api_config = provider.config().await.unwrap();
        let _client = Client::new(api_config);

        let client = provider.client().await.unwrap();
        let debug = format!("{client:?}");
        assert!(debug.contains("Client"));
    }

    #[tokio::test]
    async fn exchange_code_with_client_exchanges_auth_code_for_token() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/token")
            .match_body(Matcher::AllOf(vec![
                Matcher::Regex("grant_type=authorization_code".into()),
                Matcher::Regex("code=TEST_CODE".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{"access_token":"NEW_TOKEN","refresh_token":"NEW_REFRESH","expires_in":1800}"#,
            )
            .create_async()
            .await;

        let url = server.url();
        let config = test_config(&url);
        let result = exchange_code_with_client(&config, "TEST_CODE", &reqwest::Client::new())
            .await
            .unwrap();

        mock.assert_async().await;
        assert_eq!(result.token.access_token, "NEW_TOKEN");
        assert!(result.token.expires_at.is_some());
    }

    #[tokio::test]
    async fn token_request_maps_400_invalid_grant_to_auth_expired() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/token")
            .with_status(400)
            .with_body(r#"{"error":"invalid_grant"}"#)
            .create_async()
            .await;

        let url = server.url();
        let config = test_config(&url);
        let error = exchange_code_with_client(&config, "CODE", &reqwest::Client::new())
            .await
            .unwrap_err();

        assert!(matches!(error, Error::AuthExpired));
    }

    #[tokio::test]
    async fn token_request_maps_non_400_to_http_status() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/token")
            .with_status(500)
            .with_body("server error")
            .create_async()
            .await;

        let url = server.url();
        let config = test_config(&url);
        let error = exchange_code_with_client(&config, "CODE", &reqwest::Client::new())
            .await
            .unwrap_err();

        assert!(matches!(error, Error::HttpStatus { status: 500, .. }));
    }

    #[tokio::test]
    async fn token_request_decode_error() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/token")
            .with_status(200)
            .with_body("not json")
            .create_async()
            .await;

        let url = server.url();
        let config = test_config(&url);
        let error = exchange_code_with_client(&config, "CODE", &reqwest::Client::new())
            .await
            .unwrap_err();

        assert!(matches!(error, Error::Decode { .. }));
    }

    #[test]
    fn start_login_creates_session_with_callback_server() {
        let config =
            AuthConfig::new("client", "secret", "https://127.0.0.1:8182/callback").unwrap();
        // Use a random port to avoid conflicts with other tests.
        let port = unused_loopback_port();
        let config = AuthConfig {
            callback_url: format!("https://127.0.0.1:{port}/callback"),
            ..config
        };
        let session = start_login(config, MemoryTokenStore::new()).unwrap();

        assert!(!session.auth_context().authorization_url.is_empty());
        assert!(!session.auth_context().state.is_empty());

        // Exercise the timeout setter.
        let session = session.timeout(Some(Duration::from_secs(1)));
        assert!(session.timeout == Some(Duration::from_secs(1)));
    }

    #[tokio::test]
    async fn login_completes_full_flow_with_callback() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/token")
            .match_body(Matcher::AllOf(vec![
                Matcher::Regex("grant_type=authorization_code".into()),
                Matcher::Regex("code=LOGIN_CODE".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{"access_token":"LOGIN_TOKEN","refresh_token":"REFRESH","expires_in":1800}"#,
            )
            .create_async()
            .await;

        let port = unused_loopback_port();
        let oauth_url = server.url();
        let config = AuthConfig {
            client_id: "client".to_string(),
            client_secret: "secret".to_string(),
            callback_url: format!("https://127.0.0.1:{port}/callback"),
            oauth_base_url: oauth_url,
        };

        let store = MemoryTokenStore::new();
        let session = start_login(config, store).unwrap();
        let state = session.auth_context().state.clone();

        // Simulate browser sending the callback via HTTPS.
        let callback_port = port;
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            // Small delay to let the session.wait() start listening.
            tokio::time::sleep(Duration::from_millis(100)).await;
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap();
            client
                .get(format!(
                    "https://127.0.0.1:{callback_port}/callback?code=LOGIN_CODE&state={state_clone}"
                ))
                .send()
                .await
                .unwrap();
        });

        let provider = session
            .timeout(Some(Duration::from_secs(5)))
            .wait()
            .await
            .unwrap();

        handle.await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            provider.store.load().unwrap().token.access_token,
            "LOGIN_TOKEN"
        );
    }

    #[test]
    fn callback_path_returns_slash_for_empty_path() {
        let url = reqwest::Url::parse("https://127.0.0.1:8182").unwrap();
        // URL parsing normalizes to "/" so callback_path returns "/".
        assert_eq!(callback_path(&url), "/");
    }

    #[test]
    fn parse_callback_request_rejects_non_get() {
        let request = "POST /callback?code=C&state=S HTTP/1.1\r\n\r\n";
        let error = parse_callback_request(request, "/callback").unwrap_err();
        assert!(error.to_string().contains("must use GET"));
    }

    #[test]
    fn parse_callback_request_rejects_wrong_path() {
        let request = "GET /wrong?code=C&state=S HTTP/1.1\r\n\r\n";
        let error = parse_callback_request(request, "/callback").unwrap_err();
        assert!(error.to_string().contains("unexpected callback path"));
    }

    #[test]
    fn parse_callback_request_rejects_missing_code() {
        let request = "GET /callback?state=S HTTP/1.1\r\n\r\n";
        let error = parse_callback_request(request, "/callback").unwrap_err();
        assert!(error.to_string().contains("missing authorization code"));
    }

    #[test]
    fn parse_callback_request_rejects_missing_state() {
        let request = "GET /callback?code=C HTTP/1.1\r\n\r\n";
        let error = parse_callback_request(request, "/callback").unwrap_err();
        assert!(error.to_string().contains("missing state"));
    }

    #[test]
    fn parse_callback_request_surfaces_oauth_error_without_description() {
        let request = "GET /callback?error=access_denied HTTP/1.1\r\n\r\n";
        let error = parse_callback_request(request, "/callback").unwrap_err();
        assert_eq!(
            error.to_string(),
            "Schwab auth callback failed: access_denied"
        );
    }

    #[test]
    fn parse_callback_request_rejects_empty_request() {
        let error = parse_callback_request("", "/callback").unwrap_err();
        assert!(error.to_string().contains("empty callback request"));
    }

    #[tokio::test]
    async fn provider_token_returns_fresh_token_without_refresh() {
        let config = test_config("http://127.0.0.1:9999");
        let store: Arc<dyn TokenStore> = Arc::new(MemoryTokenStore::new());
        store
            .save(&token_file(
                "STILL_FRESH",
                "REFRESH",
                current_timestamp().unwrap() + 3600,
            ))
            .unwrap();
        let provider = Provider::with_http_client(config, store, reqwest::Client::new()).unwrap();

        let token = provider.token().await.unwrap();
        assert_eq!(token, "STILL_FRESH");
    }

    #[tokio::test]
    async fn provider_token_returns_auth_expired_for_stale_refresh() {
        let config = test_config("http://127.0.0.1:9999");
        let store: Arc<dyn TokenStore> = Arc::new(MemoryTokenStore::new());
        // creation_timestamp far in the past makes refresh token stale
        let mut tf = token_file("OLD", "REFRESH", current_timestamp().unwrap() - 10);
        tf.creation_timestamp = 0;
        store.save(&tf).unwrap();
        let provider = Provider::with_http_client(config, store, reqwest::Client::new()).unwrap();

        assert!(matches!(provider.token().await, Err(Error::AuthExpired)));
    }

    #[tokio::test]
    async fn provider_refresh_returns_auth_expired_for_stale_refresh() {
        let config = test_config("http://127.0.0.1:9999");
        let store: Arc<dyn TokenStore> = Arc::new(MemoryTokenStore::new());
        let mut tf = token_file("OLD", "REFRESH", current_timestamp().unwrap() - 10);
        tf.creation_timestamp = 0;
        store.save(&tf).unwrap();
        let provider = Provider::with_http_client(config, store, reqwest::Client::new()).unwrap();

        assert!(matches!(provider.refresh().await, Err(Error::AuthExpired)));
    }
}
