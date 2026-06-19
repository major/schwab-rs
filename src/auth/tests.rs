use std::{assert_matches, net::TcpListener, path::PathBuf};

use mockito::Matcher;

use super::callback::{CallbackServer, parse_callback_request};
use super::oauth::{authorize_url_with_state, exchange_code_with_client};
use super::*;
use crate::test_support::fixture;

struct CurrentDirGuard(PathBuf);

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

#[test]
fn auth_config_rejects_insecure_or_non_loopback_callbacks() {
    assert_matches!(
        AuthConfig::new("client", "secret", "http://127.0.0.1:8182/callback"),
        Err(Error::InvalidAuthConfig {
            field: "callback_url",
            ..
        })
    );
    assert_matches!(
        AuthConfig::new("client", "secret", "https://localhost:8182/callback"),
        Err(Error::InvalidAuthConfig {
            field: "callback_url",
            ..
        })
    );
    assert_matches!(
        AuthConfig::new("client", "secret", "https://127.0.0.1/callback"),
        Err(Error::InvalidAuthConfig {
            field: "callback_url",
            ..
        })
    );
}

#[test]
fn authorize_url_contains_schwab_oauth_parameters() {
    let config = AuthConfig::new("client-id", "secret", "https://127.0.0.1:8182/callback").unwrap();
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

    let result = {
        let _current_dir_guard = CurrentDirGuard(current_dir);
        std::env::set_current_dir(&test_dir).unwrap();

        let store = FileTokenStore::new("schwab-token.json");
        let token_file = token_file("ACCESS", "REFRESH", current_timestamp().unwrap() + 3600);

        store.save(&token_file).and_then(|()| store.load())
    };

    fs::remove_dir_all(&test_dir).unwrap();
    assert_eq!(result.unwrap().token.access_token, "ACCESS");
}

#[test]
fn memory_token_store_reports_auth_required_when_empty() {
    let store = MemoryTokenStore::new();

    assert_matches!(store.load(), Err(Error::AuthRequired));
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
async fn refresh_token_file_maps_invalid_grant_to_reauth_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/token")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":"invalid_grant","error_description":"Refresh token expired"}"#)
        .create_async()
        .await;

    let url = server.url();
    let config = test_config(&url);
    let original = token_file("OLD", "REFRESH1", current_timestamp().unwrap() - 1);

    let error = refresh_token_file(&config, &original).await.unwrap_err();

    assert_matches!(error, Error::RefreshTokenInvalid);
}

#[tokio::test]
async fn refresh_token_file_maps_text_invalid_grant_to_reauth_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("POST", "/token")
        .with_status(400)
        .with_body("invalid_grant: refresh token expired")
        .create_async()
        .await;

    let url = server.url();
    let config = test_config(&url);
    let original = token_file("OLD", "REFRESH1", current_timestamp().unwrap() - 1);

    let error = refresh_token_file(&config, &original).await.unwrap_err();

    assert_matches!(error, Error::RefreshTokenInvalid);
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

    let provider = exchange_redirect_url(config, MemoryTokenStore::new(), &context, redirect_url)
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
    let config = AuthConfig::new("client", "secret", "https://127.0.0.1:8182/callback").unwrap();
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
    assert_matches!(store.load(), Err(Error::AuthRequired));
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
    let config = AuthConfig::new("client", "secret", "https://127.0.0.1:8182/callback").unwrap();
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
        .with_body(r#"{"access_token":"REFRESHED","refresh_token":"REFRESH2","expires_in":1800}"#)
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
async fn token_request_maps_400_invalid_grant_to_refresh_token_invalid() {
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

    assert_matches!(error, Error::RefreshTokenInvalid);
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

    assert_matches!(error, Error::HttpStatus { status: 500, .. });
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

    assert_matches!(error, Error::Decode { .. });
}

#[test]
fn start_login_creates_session_with_callback_server() {
    let config = AuthConfig::new("client", "secret", "https://127.0.0.1:8182/callback").unwrap();
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
        .with_body(r#"{"access_token":"LOGIN_TOKEN","refresh_token":"REFRESH","expires_in":1800}"#)
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

    assert_matches!(provider.token().await, Err(Error::AuthExpired));
}

#[tokio::test]
async fn provider_refresh_returns_auth_expired_for_stale_refresh() {
    let config = test_config("http://127.0.0.1:9999");
    let store: Arc<dyn TokenStore> = Arc::new(MemoryTokenStore::new());
    let mut tf = token_file("OLD", "REFRESH", current_timestamp().unwrap() - 10);
    tf.creation_timestamp = 0;
    store.save(&tf).unwrap();
    let provider = Provider::with_http_client(config, store, reqwest::Client::new()).unwrap();

    assert_matches!(provider.refresh().await, Err(Error::AuthExpired));
}
