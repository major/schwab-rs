#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;

/// Builds a Schwab authorization URL and CSRF state for the browser flow.
///
/// # Examples
///
/// ```
/// use schwab::auth::{self, AuthConfig};
///
/// let config = AuthConfig::new(
///     "my-app-key",
///     "my-app-secret",
///     "https://127.0.0.1:8182/callback",
/// )
/// .unwrap();
/// let context = auth::authorize_url(&config).unwrap();
///
/// assert!(context.authorization_url.contains("response_type=code"));
/// assert!(context.authorization_url.contains("client_id=my-app-key"));
/// ```
///
/// # Errors
///
/// Returns [`crate::Error::InvalidAuthConfig`] if the auth configuration is invalid.
pub fn authorize_url(config: &AuthConfig) -> Result<AuthContext> {
    authorize_url_with_state(config, &random_oauth_state()?)
}

pub(super) fn authorize_url_with_state(config: &AuthConfig, state: &str) -> Result<AuthContext> {
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
/// # Examples
///
/// ```no_run
/// # async fn example() -> schwab::Result<()> {
/// use schwab::auth::{self, AuthConfig};
///
/// let config = AuthConfig::new(
///     "my-app-key",
///     "my-app-secret",
///     "https://127.0.0.1:8182/callback",
/// )?;
/// let token_file = auth::exchange_code(&config, "authorization-code").await?;
/// println!("token expires at: {:?}", token_file.token.expires_at);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an [`Error`] if the code is empty, the token request fails, or the
/// response cannot be decoded.
#[instrument(skip_all)]
pub async fn exchange_code(config: &AuthConfig, code: &str) -> Result<TokenFile> {
    exchange_code_with_client(config, code, &reqwest::Client::new()).await
}

pub(super) async fn exchange_code_with_client(
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
///
/// # Examples
///
/// ```
/// use schwab::auth::{self, AuthContext};
///
/// let context = AuthContext {
///     callback_url: "https://127.0.0.1:8182/callback".to_string(),
///     authorization_url: String::new(),
///     state: "my-state".to_string(),
/// };
/// let redirect = "https://127.0.0.1:8182/callback?code=AUTH_CODE&state=my-state";
/// let result = auth::parse_redirect_url(&context, redirect).unwrap();
///
/// assert_eq!(result.code, "AUTH_CODE");
/// assert_eq!(result.state, "my-state");
/// ```
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
/// # Examples
///
/// ```no_run
/// # async fn example() -> schwab::Result<()> {
/// use schwab::auth::{self, AuthConfig, FileTokenStore, TokenStore};
///
/// let config = AuthConfig::new(
///     "my-app-key",
///     "my-app-secret",
///     "https://127.0.0.1:8182/callback",
/// )?;
/// let store = FileTokenStore::new("token.json");
/// let token_file = store.load()?;
/// let refreshed = auth::refresh_token_file(&config, &token_file).await?;
/// store.save(&refreshed)?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`crate::Error::AuthExpired`] if the refresh token is missing or has expired locally.
/// Returns [`crate::Error::RefreshTokenInvalid`] if Schwab rejects the refresh token.
/// Returns an [`Error`] if the refresh request fails or the response cannot be decoded.
#[instrument(skip_all)]
pub async fn refresh_token_file(config: &AuthConfig, token_file: &TokenFile) -> Result<TokenFile> {
    refresh_token_file_with_client(config, token_file, &reqwest::Client::new()).await
}

pub(super) async fn refresh_token_file_with_client(
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
        if status.as_u16() == 400 && is_invalid_grant_error(&body) {
            return Err(Error::RefreshTokenInvalid);
        }
        return Err(Error::HttpStatus {
            status: status.as_u16(),
            body,
        });
    }
    let text = response.text().await.map_err(Error::Request)?;
    serde_json::from_str::<TokenData>(&text).map_err(|source| Error::Decode { source, body: text })
}

fn is_invalid_grant_error(body: &str) -> bool {
    serde_json::from_str::<OAuthErrorBody>(body)
        .ok()
        .and_then(|error| error.error)
        .is_some_and(|error| error == "invalid_grant")
        || body.contains("invalid_grant")
}

fn basic_auth_header(config: &AuthConfig) -> Result<HeaderValue> {
    let username = urlencoding::encode(&config.client_id);
    let password = urlencoding::encode(&config.client_secret);
    let encoded =
        base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
    HeaderValue::from_str(&format!("Basic {encoded}"))
        .map_err(|error| Error::AuthCallback(error.to_string()))
}
