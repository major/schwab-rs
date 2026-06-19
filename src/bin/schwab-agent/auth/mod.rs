//! Authentication command handlers for login, token exchange, refresh, and status.

use std::fs::File;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use std::time::{Duration, Instant};

use schwab::auth::{
    AuthConfig, AuthContext, CallbackResult, FileTokenStore, Provider, TokenFile, authorize_url,
    exchange_redirect_url,
};
use serde::Serialize;
use serde_json::{Value, to_value};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};

use crate::cli::{AuthCommand, AuthExchangeArgs, Cli, LoginArgs, LoginUrlArgs};
use crate::error::AppError;

/// Maximum age of a Schwab refresh token in seconds (6.5 days, per Schwab's documented policy).
const REFRESH_TOKEN_MAX_AGE_SECONDS: i64 = 561_600;
const CALLBACK_READ_LIMIT: usize = 8192;

/// Dispatches an `auth` subcommand to the appropriate handler.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) async fn handle(_cli: &Cli, command: &AuthCommand) -> Result<Value, AppError> {
    match command {
        AuthCommand::Status => status(),
        AuthCommand::Login(args) => login(args).await,
        AuthCommand::LoginUrl(args) => login_url(args),
        AuthCommand::Exchange(args) => exchange(args).await,
        AuthCommand::Refresh => refresh().await,
    }
}

/// Builds an `AuthConfig` from environment variables and the shared config file
/// at `~/.config/schwab-agent/config.json`.
///
/// Resolution order for each credential: environment variable > config file.
/// The callback URL additionally falls back to the hardcoded default.
pub(crate) fn build_config() -> Result<AuthConfig, AppError> {
    let (client_id, client_secret, callback_url) = crate::config::resolve_credentials()?;
    Ok(AuthConfig::new(&client_id, &client_secret, &callback_url)?)
}

/// Testable variant of [`build_config`] that loads agent config from a specific path
/// instead of the default location, isolating tests from the real config file.
#[cfg(test)]
pub(crate) fn build_config_from(config_path: &std::path::Path) -> Result<AuthConfig, AppError> {
    let (client_id, client_secret, callback_url) =
        crate::config::resolve_credentials_from(config_path)?;
    Ok(AuthConfig::new(&client_id, &client_secret, &callback_url)?)
}

/// Returns a `Provider` backed by the saved token file, failing if the file does not exist.
pub(crate) fn provider() -> Result<Provider, AppError> {
    let token_path = crate::config::token_path();
    require_token_file(&token_path)?;
    Ok(Provider::from_token_file(build_config()?, token_path)?)
}

/// Reads the token file and returns a JSON summary of its current auth state.
///
/// Returns a "missing" status object when no token file exists, rather than an error,
/// so callers can inspect the state without special-casing the absent-file case.
fn status() -> Result<Value, AppError> {
    let token_path = crate::config::token_path();
    if !token_path.exists() {
        return Ok(to_value(AuthStatus::missing(&token_path))?);
    }

    let token_file: TokenFile = serde_json::from_reader(File::open(&token_path)?)?;
    Ok(to_value(AuthStatus::from_token_file(
        &token_path,
        &token_file,
    ))?)
}

/// Runs the full interactive OAuth login flow, blocking until Schwab redirects to the callback.
///
/// Optionally opens the authorization URL in the system browser. Writes the resulting
/// token to disk via `FileTokenStore` so subsequent commands can reuse it.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn login(args: &LoginArgs) -> Result<Value, AppError> {
    let token_path = crate::config::token_path();
    if let Some(parent) = token_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let config = build_config()?;
    let auth_context = authorize_url(&config)?;
    let callback_server = CallbackServer::start(&auth_context.callback_url)?;
    let url = auth_context.authorization_url.clone();
    let browser_opened = if args.no_browser {
        false
    } else {
        open::that(&url).is_ok()
    };
    // Block until Schwab redirects to the local callback server with a complete
    // OAuth callback. Browser certificate-warning probes are ignored by the
    // callback server so they do not consume the login attempt.
    let redirect_url =
        callback_server.wait(&auth_context, Some(Duration::from_secs(args.timeout)))?;
    let _provider = exchange_redirect_url(
        config,
        FileTokenStore::new(&token_path),
        &auth_context,
        &redirect_url,
    )
    .await?;
    Ok(to_value(LoginOutput {
        logged_in: true,
        token_path: token_path.display().to_string(),
        browser_opened,
    })?)
}

struct CallbackServer {
    listener: TcpListener,
    tls_config: Arc<ServerConfig>,
    callback_path: String,
}

impl CallbackServer {
    fn start(callback_url: &str) -> Result<Self, AppError> {
        let parsed = parse_callback_url(callback_url)?;
        let port = parsed.port().ok_or_else(|| {
            AppError::Schwab(schwab::Error::InvalidAuthConfig {
                field: "callback_url",
                message: "callback URL must include an explicit port".to_string(),
            })
        })?;
        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            tls_config: Arc::new(callback_tls_config()?),
            callback_path: callback_path(&parsed),
        })
    }

    fn wait(
        self,
        auth_context: &AuthContext,
        timeout: Option<Duration>,
    ) -> Result<String, AppError> {
        let deadline = timeout.map(|timeout| Instant::now() + timeout);
        loop {
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                return Err(callback_error("timed out waiting for callback"));
            }
            match self.listener.accept() {
                Ok((stream, _)) if deadline.is_some_and(|deadline| Instant::now() >= deadline) => {
                    drop(stream);
                    return Err(callback_error("timed out waiting for callback"));
                }
                Ok((stream, _)) => match handle_callback_stream(
                    stream,
                    self.tls_config.clone(),
                    &self.callback_path,
                    &auth_context.state,
                    stream_io_timeout(deadline)?,
                )? {
                    CallbackOutcome::Continue => continue,
                    CallbackOutcome::Fatal(message) => return Err(callback_error(message)),
                    CallbackOutcome::Complete(result) => {
                        if result.state != auth_context.state {
                            return Err(callback_error("state mismatch"));
                        }
                        return callback_redirect_url(auth_context, &result);
                    }
                },
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(AppError::Io(error)),
            }
        }
    }
}

enum CallbackOutcome {
    Continue,
    Fatal(String),
    Complete(CallbackResult),
}

fn handle_callback_stream(
    stream: TcpStream,
    tls_config: Arc<ServerConfig>,
    callback_path: &str,
    expected_state: &str,
    io_timeout: Duration,
) -> Result<CallbackOutcome, AppError> {
    stream.set_nonblocking(false)?;
    stream.set_read_timeout(Some(io_timeout))?;
    stream.set_write_timeout(Some(io_timeout))?;
    let connection = match ServerConnection::new(tls_config) {
        Ok(connection) => connection,
        Err(_) => return Ok(CallbackOutcome::Continue),
    };
    let mut stream = StreamOwned::new(connection, stream);
    let mut buffer = vec![0; CALLBACK_READ_LIMIT];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(bytes_read) => bytes_read,
        Err(_) => return Ok(CallbackOutcome::Continue),
    };
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let outcome = match parse_callback_request(&request, callback_path) {
        CallbackOutcome::Complete(result) if result.state != expected_state => {
            CallbackOutcome::Fatal("state mismatch".to_string())
        }
        outcome => outcome,
    };
    let response = match &outcome {
        CallbackOutcome::Continue => http_response(
            "400 Bad Request",
            "Waiting for the Schwab authorization callback. You can close this tab.",
        ),
        CallbackOutcome::Fatal(message) => http_response("400 Bad Request", message),
        CallbackOutcome::Complete(_) => {
            http_response("200 OK", "Login successful. You can close this tab.")
        }
    };
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
    Ok(outcome)
}

fn stream_io_timeout(deadline: Option<Instant>) -> Result<Duration, AppError> {
    let max_stream_timeout = Duration::from_secs(10);
    let Some(deadline) = deadline else {
        return Ok(max_stream_timeout);
    };
    let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
        return Err(callback_error("timed out waiting for callback"));
    };
    if remaining.is_zero() {
        return Err(callback_error("timed out waiting for callback"));
    }
    Ok(remaining.min(max_stream_timeout))
}

fn parse_callback_request(request: &str, callback_path: &str) -> CallbackOutcome {
    let Some(request_line) = request.lines().next() else {
        return CallbackOutcome::Continue;
    };
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    if method != "GET" {
        return CallbackOutcome::Continue;
    }
    let Ok(url) = reqwest::Url::parse(&format!("https://127.0.0.1{target}")) else {
        return CallbackOutcome::Continue;
    };
    if url.path() != callback_path {
        return CallbackOutcome::Continue;
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
            return CallbackOutcome::Fatal(format!("{error}: {description}"));
        }
        return CallbackOutcome::Fatal(error);
    }
    match (code, state) {
        (Some(code), Some(state)) => CallbackOutcome::Complete(CallbackResult { code, state }),
        _ => CallbackOutcome::Continue,
    }
}

fn callback_tls_config() -> Result<ServerConfig, AppError> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let certificate =
        rcgen::generate_simple_self_signed(vec!["127.0.0.1".to_string(), "localhost".to_string()])
            .map_err(|error| {
                callback_error(format!("failed to generate callback TLS cert: {error}"))
            })?;
    let cert_der = certificate.cert.der().clone();
    let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        certificate.signing_key.serialize_der(),
    ));
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .map_err(|error| callback_error(format!("invalid callback TLS cert: {error}")))
}

fn parse_callback_url(callback_url: &str) -> Result<reqwest::Url, AppError> {
    let url = reqwest::Url::parse(callback_url).map_err(|error| {
        AppError::Schwab(schwab::Error::InvalidAuthConfig {
            field: "callback_url",
            message: error.to_string(),
        })
    })?;
    if url.scheme() != "https" {
        return Err(AppError::Schwab(schwab::Error::InvalidAuthConfig {
            field: "callback_url",
            message: "callback URL must use https".to_string(),
        }));
    }
    if url.host_str() != Some("127.0.0.1") {
        return Err(AppError::Schwab(schwab::Error::InvalidAuthConfig {
            field: "callback_url",
            message: "callback URL host must be exactly 127.0.0.1".to_string(),
        }));
    }
    Ok(url)
}

fn callback_path(url: &reqwest::Url) -> String {
    let path = url.path();
    if path.is_empty() {
        "/".to_string()
    } else {
        path.to_string()
    }
}

fn callback_redirect_url(
    auth_context: &AuthContext,
    result: &CallbackResult,
) -> Result<String, AppError> {
    let mut url = reqwest::Url::parse(&auth_context.callback_url).map_err(|error| {
        AppError::Schwab(schwab::Error::InvalidAuthConfig {
            field: "callback_url",
            message: error.to_string(),
        })
    })?;
    url.query_pairs_mut()
        .clear()
        .append_pair("code", &result.code)
        .append_pair("state", &result.state);
    Ok(url.into())
}

fn callback_error(message: impl Into<String>) -> AppError {
    AppError::Schwab(schwab::Error::AuthCallback(message.into()))
}

fn http_response(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\ncontent-type: text/plain; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// Generates the Schwab authorization URL without starting a local callback server.
///
/// Useful for headless or scripted flows where the caller handles the redirect manually
/// and will later call `exchange` with the redirect URL.
fn login_url(args: &LoginUrlArgs) -> Result<Value, AppError> {
    let token_path = crate::config::token_path();
    let context = authorize_url(&build_config()?)?;
    let browser_opened = if args.no_browser {
        false
    } else {
        open::that(&context.authorization_url).is_ok()
    };
    Ok(to_value(LoginUrlOutput {
        authorization_url: context.authorization_url,
        callback_url: context.callback_url,
        state: context.state,
        token_path: token_path.display().to_string(),
        browser_opened,
    })?)
}

/// Exchanges a Schwab redirect URL for an access/refresh token pair and saves it to disk.
///
/// This is the second step of the manual login flow started by `login_url`.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn exchange(args: &AuthExchangeArgs) -> Result<Value, AppError> {
    let (_client_id, _client_secret, callback_url) = crate::config::resolve_credentials()?;
    let context = AuthContext {
        callback_url,
        authorization_url: String::new(),
        state: args.state.clone(),
    };
    let token_path = crate::config::token_path();
    exchange_redirect_url(
        build_config()?,
        FileTokenStore::new(&token_path),
        &context,
        &args.redirect_url,
    )
    .await?;
    Ok(to_value(TokenSavedOutput {
        token_saved: true,
        token_path: token_path.display().to_string(),
    })?)
}

/// Uses the saved refresh token to obtain a new access token and overwrites the token file.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn refresh() -> Result<Value, AppError> {
    let token_path = crate::config::token_path();
    let token_file = provider()?.refresh().await?;
    Ok(to_value(RefreshOutput {
        refreshed: true,
        token_path: token_path.display().to_string(),
        access_expires_at: token_file.token.expires_at.and_then(format_epoch),
    })?)
}

/// Returns an error if the token file at `path` does not exist.
///
/// Centralizes the missing-token check so callers don't repeat the same path-exists guard.
fn require_token_file(path: &Path) -> Result<(), AppError> {
    if path.exists() {
        Ok(())
    } else {
        Err(AppError::TokenFileMissing(path.display().to_string()))
    }
}

/// Returns the current UTC time as a Unix timestamp in seconds.
fn now_epoch() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

/// Converts a Unix timestamp in seconds to an RFC 3339 string, returning `None` on overflow.
fn format_epoch(epoch: i64) -> Option<String> {
    OffsetDateTime::from_unix_timestamp(epoch)
        .ok()
        .and_then(|timestamp| timestamp.format(&Rfc3339).ok())
}

/// JSON output for the `auth status` command, describing the current token state.
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
struct AuthStatus {
    /// Whether a token file exists on disk.
    token_present: bool,
    /// Absolute path to the token file.
    token_path: String,
    /// RFC 3339 timestamp when the access token expires, if known.
    access_expires_at: Option<String>,
    /// Whether the access token has already expired, if an expiry time is recorded.
    access_expired: Option<bool>,
    /// RFC 3339 timestamp when the refresh token was created (derived from the token file).
    refresh_created_at: Option<String>,
    /// RFC 3339 timestamp when the refresh token will expire, based on `REFRESH_TOKEN_MAX_AGE_SECONDS`.
    refresh_expires_at: Option<String>,
    /// Whether the refresh token has already expired.
    refresh_expired: Option<bool>,
    /// Whether a token refresh can be attempted right now (refresh token present and not expired).
    refresh_possible: bool,
}

impl AuthStatus {
    /// Returns an `AuthStatus` representing the state when no token file exists.
    #[must_use]
    fn missing(token_path: &Path) -> Self {
        Self {
            token_present: false,
            token_path: token_path.display().to_string(),
            access_expires_at: None,
            access_expired: None,
            refresh_created_at: None,
            refresh_expires_at: None,
            refresh_expired: None,
            refresh_possible: false,
        }
    }

    /// Builds an `AuthStatus` by inspecting a loaded `TokenFile` against the current time.
    ///
    /// Refresh token expiry is computed from `token_file.creation_timestamp` plus
    /// `REFRESH_TOKEN_MAX_AGE_SECONDS`, since Schwab does not embed that expiry in the file.
    #[must_use]
    fn from_token_file(token_path: &Path, token_file: &TokenFile) -> Self {
        let now = now_epoch();
        let refresh_expires_at_epoch =
            token_file.creation_timestamp + REFRESH_TOKEN_MAX_AGE_SECONDS;
        Self {
            token_present: true,
            token_path: token_path.display().to_string(),
            access_expires_at: token_file.token.expires_at.and_then(format_epoch),
            access_expired: token_file
                .token
                .expires_at
                .map(|expires_at| expires_at <= now),
            refresh_created_at: format_epoch(token_file.creation_timestamp),
            refresh_expires_at: format_epoch(refresh_expires_at_epoch),
            refresh_expired: Some(refresh_expires_at_epoch <= now),
            refresh_possible: token_file.token.refresh_token.is_some()
                && refresh_expires_at_epoch > now,
        }
    }
}

/// JSON output for the `auth login` command after a successful interactive login.
#[derive(Debug, Serialize)]
struct LoginOutput {
    /// Always `true`; present so callers can confirm success without inspecting exit code.
    logged_in: bool,
    /// Path where the token file was written.
    token_path: String,
    /// Whether the authorization URL was successfully opened in the system browser.
    browser_opened: bool,
}

/// JSON output for the `auth login-url` command, containing everything needed to complete a manual login.
#[derive(Debug, Serialize)]
struct LoginUrlOutput {
    /// The Schwab authorization URL the user must visit to grant access.
    authorization_url: String,
    /// The local callback URL Schwab will redirect to after the user approves.
    callback_url: String,
    /// CSRF state token; must be passed to `auth exchange` to validate the redirect.
    state: String,
    /// Path where the token file will be written after a successful exchange.
    token_path: String,
    /// Whether the authorization URL was successfully opened in the system browser.
    browser_opened: bool,
}

/// JSON output for the `auth exchange` command after the token has been written to disk.
#[derive(Debug, Serialize)]
struct TokenSavedOutput {
    /// Always `true`; present so callers can confirm success without inspecting exit code.
    token_saved: bool,
    /// Path where the token file was written.
    token_path: String,
}

/// JSON output for the `auth refresh` command after a successful token refresh.
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
struct RefreshOutput {
    /// Always `true`; present so callers can confirm success without inspecting exit code.
    refreshed: bool,
    /// Path to the token file that was updated.
    token_path: String,
    /// RFC 3339 timestamp when the new access token expires, if the API returned one.
    access_expires_at: Option<String>,
}

#[cfg(test)]
mod tests;

pub mod cli;
