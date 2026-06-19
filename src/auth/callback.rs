#![cfg_attr(coverage_nightly, coverage(off))]
use super::oauth::exchange_code_with_client;
use super::*;

/// Starts the full login flow and calls `url_handler` with the authorization URL.
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> schwab::Result<()> {
/// use schwab::auth::{self, AuthConfig, FileTokenStore};
///
/// let config = AuthConfig::new(
///     "my-app-key",
///     "my-app-secret",
///     "https://127.0.0.1:8182/callback",
/// )?;
/// let provider = auth::login(config, FileTokenStore::new("token.json"), |url| {
///     println!("Open this URL in your browser: {url}");
///     Ok(())
/// })
/// .await?;
/// # Ok(())
/// # }
/// ```
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
/// # Examples
///
/// ```no_run
/// # async fn example() -> schwab::Result<()> {
/// use schwab::auth::{self, AuthConfig, FileTokenStore};
///
/// let config = AuthConfig::new(
///     "my-app-key",
///     "my-app-secret",
///     "https://127.0.0.1:8182/callback",
/// )?;
/// let session = auth::start_login(config, FileTokenStore::new("token.json"))?;
/// println!("Open: {}", session.auth_context().authorization_url);
/// let provider = session.wait().await?;
/// # Ok(())
/// # }
/// ```
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
    pub(super) timeout: Option<Duration>,
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

pub(super) struct CallbackServer {
    result_rx: mpsc::Receiver<Result<CallbackResult>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl CallbackServer {
    pub(super) fn start(callback_url: &str) -> Result<Self> {
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

    pub(super) fn wait(mut self, timeout: Option<Duration>) -> Result<CallbackResult> {
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

pub(super) fn parse_callback_request(request: &str, callback_path: &str) -> Result<CallbackResult> {
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

fn http_response(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\ncontent-type: text/plain; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
}
