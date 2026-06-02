//! Agent configuration loaded from the shared config file.
//!
//! The agent reads `~/.config/schwab-agent/config.json` (or
//! `$XDG_CONFIG_HOME/schwab-agent/config.json`) which is shared with the Go
//! CLI. The config file is optional; missing files or missing keys default to
//! safe values (mutable operations disabled). Credentials and token paths can
//! also be provided with environment variables.

use std::path::{Path, PathBuf};

#[cfg(test)]
use std::sync::{LazyLock, Mutex};

use serde::Deserialize;
use serde::Serialize;
use serde_json::{Value, to_value};

use crate::error::AppError;

/// Shared lock for tests that mutate process-wide environment variables.
#[cfg(test)]
pub(crate) static TEST_ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// The default OAuth callback URL used when no env var or config file provides one.
pub(crate) const DEFAULT_CALLBACK_URL: &str = "https://127.0.0.1:8182";

/// Subset of the shared agent config relevant to this CLI.
///
/// Unknown keys are silently ignored so the Go CLI can add fields without
/// breaking the Rust CLI.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct AgentConfig {
    /// Schwab app client ID, shared with the Go CLI.
    pub client_id: Option<String>,

    /// Schwab app client secret, shared with the Go CLI.
    pub client_secret: Option<String>,

    /// OAuth callback URL registered with Schwab.
    pub callback_url: Option<String>,

    /// When `true`, mutable order operations (place, replace, cancel) are
    /// allowed. Defaults to `false` when the key is absent or the config
    /// file does not exist.
    #[serde(
        default,
        rename = "i-also-like-to-live-dangerously",
        alias = "i_also_like_to_live_dangerously"
    )]
    pub i_also_like_to_live_dangerously: bool,
}

/// Returns the path to the shared agent config file.
///
/// Uses `$XDG_CONFIG_HOME/schwab-agent/config.json`, falling back to
/// `~/.config/schwab-agent/config.json` on platforms without `XDG_CONFIG_HOME`.
#[must_use]
pub(crate) fn config_path() -> PathBuf {
    xdg_config_home()
        .unwrap_or_else(|| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".config")))
        .join("schwab-agent")
        .join("config.json")
}

/// Returns `$XDG_CONFIG_HOME` as a [`PathBuf`] when the env var is set to a
/// non-empty value, regardless of platform. This lets tests inject a temp dir
/// on macOS where [`dirs::config_dir`] would otherwise resolve to
/// `~/Library/Application Support` and ignore `XDG_CONFIG_HOME`.
fn xdg_config_home() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// Returns the default OAuth token path.
#[must_use]
fn default_token_path() -> PathBuf {
    xdg_config_home()
        .unwrap_or_else(|| dirs::config_dir().unwrap_or_else(|| PathBuf::from(".config")))
        .join("schwab-agent-rs")
        .join("token.json")
}

/// Returns sanitized setup status for config, auth, path, and debug discovery.
pub(crate) fn status() -> Result<Value, AppError> {
    let config_path = config_path();
    let config = load_agent_config_from(&config_path)?;
    let token_path = token_path();
    let status = ConfigStatus::from_config(&config_path, &token_path, &config);
    Ok(to_value(status)?)
}

/// Returns the OAuth token path from `SCHWAB_TOKEN_PATH`, falling back to the
/// default path under the user's config directory.
#[must_use]
pub(crate) fn token_path() -> PathBuf {
    std::env::var_os("SCHWAB_TOKEN_PATH")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(default_token_path)
}

/// Loads the agent config from a specific path.
///
/// Returns `AgentConfig::default()` (all flags false) when the file is
/// missing, which makes "file not found" a safe no-op rather than an error.
pub(crate) fn load_agent_config_from(path: &std::path::Path) -> Result<AgentConfig, AppError> {
    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(serde_json::from_str(&contents)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AgentConfig::default()),
        Err(e) => Err(AppError::Io(e)),
    }
}

/// Loads the agent config from the default shared config path.
pub(crate) fn load_agent_config() -> Result<AgentConfig, AppError> {
    load_agent_config_from(&config_path())
}

/// Resolves Schwab API credentials from environment variables and the shared
/// agent config file.
///
/// Environment variables take precedence over the config file. The callback URL
/// falls back to [`DEFAULT_CALLBACK_URL`] when neither source provides one.
pub(crate) fn resolve_credentials() -> Result<(String, String, String), AppError> {
    resolve_credentials_from(&config_path())
}

/// Testable variant of [`resolve_credentials`] that loads the agent config from
/// a specific path instead of the default location.
pub(crate) fn resolve_credentials_from(path: &Path) -> Result<(String, String, String), AppError> {
    let config = load_agent_config_from(path)?;
    let client_id = std::env::var("SCHWAB_CLIENT_ID")
        .ok()
        .or(config.client_id)
        .ok_or(AppError::MissingAuthConfig("client_id"))?;
    let client_secret = std::env::var("SCHWAB_CLIENT_SECRET")
        .ok()
        .or(config.client_secret)
        .ok_or(AppError::MissingAuthConfig("client_secret"))?;
    let callback_url = std::env::var("SCHWAB_CALLBACK_URL")
        .ok()
        .or(config.callback_url)
        .unwrap_or_else(|| DEFAULT_CALLBACK_URL.to_string());

    Ok((client_id, client_secret, callback_url))
}

/// Checks that mutable operations are enabled, loading config from the
/// given path. Used by tests to avoid depending on the real config file.
#[cfg(test)]
fn require_mutable_enabled_from(path: &std::path::Path) -> Result<(), AppError> {
    let config = load_agent_config_from(path)?;
    if config.i_also_like_to_live_dangerously {
        Ok(())
    } else {
        Err(AppError::MutableDisabled)
    }
}

/// Checks that mutable operations are enabled in the agent config.
///
/// Call this guard at the top of every mutable command handler (place,
/// replace, cancel). Returns `Ok(())` when the flag is set, or
/// `Err(AppError::MutableDisabled)` otherwise.
pub(crate) fn require_mutable_enabled() -> Result<(), AppError> {
    let config = load_agent_config()?;
    if config.i_also_like_to_live_dangerously {
        Ok(())
    } else {
        Err(AppError::MutableDisabled)
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
struct ConfigStatus {
    config_path: String,
    config_present: bool,
    token_path: String,
    token_present: bool,
    credential_sources: CredentialSources,
    callback_url_source: ConfigSource,
    callback_url_default: &'static str,
    token_path_source: ConfigSource,
    mutable_operations_enabled: bool,
    debug: DebugStatus,
    precedence: [&'static str; 4],
    environment_variables: [&'static str; 7],
}

impl ConfigStatus {
    fn from_config(config_path: &Path, token_path: &Path, config: &AgentConfig) -> Self {
        Self {
            config_path: config_path.display().to_string(),
            config_present: config_path.exists(),
            token_path: token_path.display().to_string(),
            token_present: token_path.exists(),
            credential_sources: CredentialSources {
                client_id: credential_source("SCHWAB_CLIENT_ID", config.client_id.as_deref()),
                client_secret: credential_source(
                    "SCHWAB_CLIENT_SECRET",
                    config.client_secret.as_deref(),
                ),
            },
            callback_url_source: callback_url_source(config),
            callback_url_default: DEFAULT_CALLBACK_URL,
            token_path_source: token_path_source(),
            mutable_operations_enabled: config.i_also_like_to_live_dangerously,
            debug: DebugStatus {
                rust_log_enabled: std::env::var_os("RUST_LOG")
                    .is_some_and(|value| !value.is_empty()),
                stderr_only: true,
            },
            precedence: ["command flags", "environment", "config file", "defaults"],
            environment_variables: [
                "SCHWAB_CLIENT_ID",
                "SCHWAB_CLIENT_SECRET",
                "SCHWAB_CALLBACK_URL",
                "SCHWAB_TOKEN_PATH",
                "XDG_CONFIG_HOME",
                "XDG_STATE_HOME",
                "RUST_LOG",
            ],
        }
    }
}

#[derive(Debug, Serialize)]
struct CredentialSources {
    client_id: ConfigSource,
    client_secret: ConfigSource,
}

#[derive(Debug, Serialize)]
struct DebugStatus {
    rust_log_enabled: bool,
    stderr_only: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum ConfigSource {
    Environment,
    ConfigFile,
    Default,
    Missing,
}

fn credential_source(env_var: &str, config_value: Option<&str>) -> ConfigSource {
    if std::env::var_os(env_var).is_some() {
        ConfigSource::Environment
    } else if config_value.is_some() {
        ConfigSource::ConfigFile
    } else {
        ConfigSource::Missing
    }
}

fn callback_url_source(config: &AgentConfig) -> ConfigSource {
    if std::env::var_os("SCHWAB_CALLBACK_URL").is_some() {
        ConfigSource::Environment
    } else if config.callback_url.is_some() {
        ConfigSource::ConfigFile
    } else {
        ConfigSource::Default
    }
}

fn token_path_source() -> ConfigSource {
    if std::env::var_os("SCHWAB_TOKEN_PATH").is_some_and(|value| !value.is_empty()) {
        ConfigSource::Environment
    } else {
        ConfigSource::Default
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, io::Write, path::Path};

    use super::*;

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, previous }
        }

        fn set_path(key: &'static str, value: &Path) -> Self {
            let previous = std::env::var_os(key);
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = std::env::var_os(key);
            unsafe {
                std::env::remove_var(key);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match self.previous.as_ref() {
                Some(value) => unsafe { std::env::set_var(self.key, value) },
                None => unsafe { std::env::remove_var(self.key) },
            }
        }
    }

    #[test]
    fn loads_config_with_flag_true() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"i-also-like-to-live-dangerously": true}}"#).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let config: AgentConfig = serde_json::from_str(&contents).unwrap();
        assert!(config.i_also_like_to_live_dangerously);
    }

    #[test]
    fn loads_config_with_flag_false() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"i-also-like-to-live-dangerously": false}}"#).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let config: AgentConfig = serde_json::from_str(&contents).unwrap();
        assert!(!config.i_also_like_to_live_dangerously);
    }

    #[test]
    fn loads_config_with_flag_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"client_id": "test"}}"#).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let config: AgentConfig = serde_json::from_str(&contents).unwrap();
        assert!(!config.i_also_like_to_live_dangerously);
    }

    #[test]
    fn default_config_has_flag_false() {
        let config = AgentConfig::default();
        assert!(!config.i_also_like_to_live_dangerously);
    }

    #[test]
    fn deserialize_ignores_unknown_keys() {
        let json = r#"{"client_id": "x", "callback_url": "https://localhost", "i-also-like-to-live-dangerously": true}"#;
        let config: AgentConfig = serde_json::from_str(json).unwrap();
        assert!(config.i_also_like_to_live_dangerously);
    }

    #[test]
    fn config_path_ends_with_expected_suffix() {
        let path = config_path();
        assert!(
            path.ends_with("schwab-agent/config.json"),
            "unexpected config path: {path:?}"
        );
    }

    #[test]
    fn token_path_from_env() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("token.json");
        let _guard = EnvVarGuard::set_path("SCHWAB_TOKEN_PATH", &path);

        assert_eq!(token_path(), path);
    }

    #[test]
    fn token_path_default_fallback() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let _guard = EnvVarGuard::remove("SCHWAB_TOKEN_PATH");

        let path = token_path();

        assert!(path.ends_with("schwab-agent-rs/token.json"));
    }

    #[test]
    fn token_path_empty_env_falls_back_to_xdg_config_home() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let _token_path = EnvVarGuard::set("SCHWAB_TOKEN_PATH", "");
        let _xdg_config_home = EnvVarGuard::set_path("XDG_CONFIG_HOME", dir.path());

        assert_eq!(
            token_path(),
            dir.path().join("schwab-agent-rs").join("token.json")
        );
    }

    #[test]
    fn require_mutable_returns_error_when_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"i-also-like-to-live-dangerously": false}}"#).unwrap();

        let result = require_mutable_enabled_from(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code(), "config.mutable_disabled");
        assert_eq!(err.exit_code(), 10);
        assert!(err.hint().is_some());
    }

    #[test]
    fn require_mutable_returns_error_when_config_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");

        let result = require_mutable_enabled_from(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code(), "config.mutable_disabled");
    }

    #[test]
    fn require_mutable_returns_ok_when_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"i-also-like-to-live-dangerously": true}}"#).unwrap();

        let result = require_mutable_enabled_from(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn load_agent_config_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.json");

        let config = load_agent_config_from(&path).expect("missing config should be safe default");

        assert!(config.client_id.is_none());
        assert!(config.client_secret.is_none());
        assert!(config.callback_url.is_none());
        assert!(!config.i_also_like_to_live_dangerously);
    }

    #[test]
    fn load_agent_config_rejects_malformed_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, "{not json").unwrap();

        let err = load_agent_config_from(&path).unwrap_err();

        assert_eq!(err.code(), "json.error");
        assert_eq!(err.exit_code(), 20);
    }

    #[test]
    fn deserializes_credential_fields() {
        let json = r#"{
            "client_id": "my_id",
            "client_secret": "my_secret",
            "callback_url": "https://localhost:9999"
        }"#;
        let config: AgentConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.client_id.as_deref(), Some("my_id"));
        assert_eq!(config.client_secret.as_deref(), Some("my_secret"));
        assert_eq!(
            config.callback_url.as_deref(),
            Some("https://localhost:9999")
        );
    }

    #[test]
    fn credential_fields_default_to_none() {
        let config = AgentConfig::default();
        assert!(config.client_id.is_none());
        assert!(config.client_secret.is_none());
        assert!(config.callback_url.is_none());
    }

    #[test]
    fn resolve_credentials_from_env() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let _client_id = EnvVarGuard::set("SCHWAB_CLIENT_ID", "env-id");
        let _client_secret = EnvVarGuard::set("SCHWAB_CLIENT_SECRET", "env-secret");
        let _callback_url = EnvVarGuard::set("SCHWAB_CALLBACK_URL", "https://env.example");
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        let (client_id, client_secret, callback_url) =
            resolve_credentials_from(&config_path).unwrap();

        assert_eq!(client_id, "env-id");
        assert_eq!(client_secret, "env-secret");
        assert_eq!(callback_url, "https://env.example");
    }

    #[test]
    fn resolve_credentials_env_overrides_config() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let _client_id = EnvVarGuard::set("SCHWAB_CLIENT_ID", "env-id");
        let _client_secret = EnvVarGuard::set("SCHWAB_CLIENT_SECRET", "env-secret");
        let _callback_url = EnvVarGuard::set("SCHWAB_CALLBACK_URL", "https://env.example");
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        std::fs::write(
            &config_path,
            r#"{"client_id":"file-id","client_secret":"file-secret","callback_url":"https://file.example"}"#,
        )
        .unwrap();

        let (client_id, client_secret, callback_url) =
            resolve_credentials_from(&config_path).unwrap();

        assert_eq!(client_id, "env-id");
        assert_eq!(client_secret, "env-secret");
        assert_eq!(callback_url, "https://env.example");
    }

    #[test]
    fn resolve_credentials_from_config_file() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let _client_id = EnvVarGuard::remove("SCHWAB_CLIENT_ID");
        let _client_secret = EnvVarGuard::remove("SCHWAB_CLIENT_SECRET");
        let _callback_url = EnvVarGuard::remove("SCHWAB_CALLBACK_URL");
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        std::fs::write(
            &config_path,
            r#"{"client_id":"file-id","client_secret":"file-secret","callback_url":"https://file.example"}"#,
        )
        .unwrap();

        let (client_id, client_secret, callback_url) =
            resolve_credentials_from(&config_path).unwrap();

        assert_eq!(client_id, "file-id");
        assert_eq!(client_secret, "file-secret");
        assert_eq!(callback_url, "https://file.example");
    }

    #[test]
    fn resolve_credentials_missing_client_id() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let _client_id = EnvVarGuard::remove("SCHWAB_CLIENT_ID");
        let _client_secret = EnvVarGuard::remove("SCHWAB_CLIENT_SECRET");
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        std::fs::write(&config_path, "{}").unwrap();

        let err = resolve_credentials_from(&config_path).unwrap_err();

        match err {
            AppError::MissingAuthConfig(field) => assert_eq!(field, "client_id"),
            other => panic!("expected MissingAuthConfig, got {other:?}"),
        }
    }

    #[test]
    fn resolve_credentials_missing_client_secret() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let _client_id = EnvVarGuard::set("SCHWAB_CLIENT_ID", "env-id");
        let _client_secret = EnvVarGuard::remove("SCHWAB_CLIENT_SECRET");
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        std::fs::write(&config_path, "{}").unwrap();

        let err = resolve_credentials_from(&config_path).unwrap_err();

        match err {
            AppError::MissingAuthConfig(field) => assert_eq!(field, "client_secret"),
            other => panic!("expected MissingAuthConfig, got {other:?}"),
        }
    }

    #[test]
    fn resolve_credentials_callback_url_default() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let _client_id = EnvVarGuard::remove("SCHWAB_CLIENT_ID");
        let _client_secret = EnvVarGuard::remove("SCHWAB_CLIENT_SECRET");
        let _callback_url = EnvVarGuard::remove("SCHWAB_CALLBACK_URL");
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        std::fs::write(
            &config_path,
            r#"{"client_id":"file-id","client_secret":"file-secret"}"#,
        )
        .unwrap();

        let (_, _, callback_url) = resolve_credentials_from(&config_path).unwrap();

        assert_eq!(callback_url, DEFAULT_CALLBACK_URL);
    }

    #[test]
    fn status_reports_sources_without_secret_values() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join("schwab-agent");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.json"),
            r#"{
                "client_id": "file-client-id-secret",
                "client_secret": "file-client-secret-secret",
                "callback_url": "https://127.0.0.1:8182/callback",
                "i-also-like-to-live-dangerously": true
            }"#,
        )
        .unwrap();
        let token_path = dir.path().join("token.json");
        std::fs::write(&token_path, "{}").unwrap();
        let _xdg_config_home = EnvVarGuard::set_path("XDG_CONFIG_HOME", dir.path());
        let _token_path = EnvVarGuard::set_path("SCHWAB_TOKEN_PATH", &token_path);
        let _client_id = EnvVarGuard::set("SCHWAB_CLIENT_ID", "env-client-id-secret");
        let _client_secret = EnvVarGuard::remove("SCHWAB_CLIENT_SECRET");
        let _callback_url = EnvVarGuard::remove("SCHWAB_CALLBACK_URL");
        let _rust_log = EnvVarGuard::set("RUST_LOG", "schwab=debug");

        let value = status().unwrap();
        let output = serde_json::to_string(&value).unwrap();

        assert_eq!(value["config_present"], true);
        assert_eq!(value["token_present"], true);
        assert_eq!(value["credential_sources"]["client_id"], "environment");
        assert_eq!(value["credential_sources"]["client_secret"], "config_file");
        assert_eq!(value["callback_url_source"], "config_file");
        assert_eq!(value["token_path_source"], "environment");
        assert_eq!(value["mutable_operations_enabled"], true);
        assert_eq!(value["debug"]["rust_log_enabled"], true);
        assert!(!output.contains("env-client-id-secret"));
        assert!(!output.contains("file-client-id-secret"));
        assert!(!output.contains("file-client-secret-secret"));
        assert!(!output.contains("127.0.0.1:8182/callback"));
    }

    #[test]
    fn status_reports_callback_url_environment_source_without_value() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let _xdg_config_home = EnvVarGuard::set_path("XDG_CONFIG_HOME", dir.path());
        let _token_path = EnvVarGuard::remove("SCHWAB_TOKEN_PATH");
        let _client_id = EnvVarGuard::remove("SCHWAB_CLIENT_ID");
        let _client_secret = EnvVarGuard::remove("SCHWAB_CLIENT_SECRET");
        let _callback_url =
            EnvVarGuard::set("SCHWAB_CALLBACK_URL", "https://127.0.0.1:9443/callback");
        let _rust_log = EnvVarGuard::remove("RUST_LOG");

        let value = status().unwrap();
        let output = serde_json::to_string(&value).unwrap();

        assert_eq!(value["callback_url_source"], "environment");
        assert!(!output.contains("127.0.0.1:9443/callback"));
    }

    #[test]
    fn status_reports_missing_credentials_and_default_paths() {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let _xdg_config_home = EnvVarGuard::set_path("XDG_CONFIG_HOME", dir.path());
        let _token_path = EnvVarGuard::remove("SCHWAB_TOKEN_PATH");
        let _client_id = EnvVarGuard::remove("SCHWAB_CLIENT_ID");
        let _client_secret = EnvVarGuard::remove("SCHWAB_CLIENT_SECRET");
        let _callback_url = EnvVarGuard::remove("SCHWAB_CALLBACK_URL");
        let _rust_log = EnvVarGuard::remove("RUST_LOG");

        let value = status().unwrap();

        assert_eq!(value["config_present"], false);
        assert_eq!(value["token_present"], false);
        assert_eq!(value["credential_sources"]["client_id"], "missing");
        assert_eq!(value["credential_sources"]["client_secret"], "missing");
        assert_eq!(value["callback_url_source"], "default");
        assert_eq!(value["token_path_source"], "default");
        assert_eq!(value["debug"]["rust_log_enabled"], false);
        assert!(
            value["token_path"]
                .as_str()
                .unwrap()
                .ends_with("schwab-agent-rs/token.json")
        );
    }
}
