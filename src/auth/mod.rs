#![cfg_attr(coverage_nightly, coverage(off))]
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
///
/// # Examples
///
/// ```
/// use schwab::auth::AuthConfig;
///
/// let config = AuthConfig::new(
///     "my-app-key",
///     "my-app-secret",
///     "https://127.0.0.1:8182/callback",
/// )
/// .unwrap();
///
/// assert_eq!(config.client_id(), "my-app-key");
/// assert_eq!(config.callback_url(), "https://127.0.0.1:8182/callback");
/// ```
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

#[derive(Deserialize)]
struct OAuthErrorBody {
    error: Option<String>,
}

/// Storage backend for persisted Schwab OAuth tokens.
pub trait TokenStore: Send + Sync {
    /// Saves a token file.
    fn save(&self, token_file: &TokenFile) -> Result<()>;

    /// Loads a token file.
    fn load(&self) -> Result<TokenFile>;
}

mod callback;
mod oauth;
mod provider;
mod store;

pub use callback::{CallbackResult, LoginSession, login, start_login};
pub use oauth::{
    authorize_url, exchange_code, exchange_redirect_url, parse_redirect_url, refresh_token_file,
};
pub use provider::Provider;
pub use store::{FileTokenStore, MemoryTokenStore};

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

fn current_timestamp() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            Error::AuthCallback(format!("system time is before UNIX epoch: {error}"))
        })?;
    i64::try_from(duration.as_secs())
        .map_err(|error| Error::AuthCallback(format!("timestamp overflow: {error}")))
}

fn redacted(value: &str) -> &'static str {
    if value.is_empty() { "" } else { "<redacted>" }
}

#[cfg(test)]
mod tests;
