//! Offline discovery output for agents and humans.

use serde::Serialize;
use serde_json::{Value, to_value};

use crate::{config, error::AppError, market, options};

const DOCS_URL: &str = "https://github.com/major/schwab-rs#schwab-agent-cli";

/// Returns sanitized config and environment health for interactive inspection.
pub(crate) fn doctor() -> Result<Value, AppError> {
    let status = config::status()?;
    Ok(to_value(DoctorOutput {
        status: "ok",
        summary: "sanitized config, auth, token, and debug checks completed without reading account data",
        config: status,
        docs_url: DOCS_URL,
    })?)
}

/// Returns the machine-readable CLI discovery schema.
pub(crate) fn schema() -> Result<Value, AppError> {
    Ok(to_value(SchemaOutput {
        name: "schwab-agent",
        version: env!("CARGO_PKG_VERSION"),
        docs_url: DOCS_URL,
        output_formats: output_formats(),
        environment_variables: environment_variables(),
        commands: commands(),
        exit_codes: exit_codes(),
        field_selectors: field_selectors(),
    })?)
}

#[derive(Debug, Serialize)]
struct DoctorOutput {
    status: &'static str,
    summary: &'static str,
    config: Value,
    docs_url: &'static str,
}

#[derive(Debug, Serialize)]
struct SchemaOutput {
    name: &'static str,
    version: &'static str,
    docs_url: &'static str,
    output_formats: Vec<OutputFormat>,
    environment_variables: Vec<EnvironmentVariable>,
    commands: Vec<CommandInfo>,
    exit_codes: Vec<ExitCodeInfo>,
    field_selectors: Vec<FieldSelectorInfo>,
}

#[derive(Debug, Serialize)]
struct CommandInfo {
    name: &'static str,
    classification: CommandClassification,
    description: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum CommandClassification {
    ReadOnly,
    Mutating,
    LocalOnly,
}

#[derive(Debug, Serialize)]
struct EnvironmentVariable {
    name: &'static str,
    purpose: &'static str,
    sensitive: bool,
}

#[derive(Debug, Serialize)]
struct OutputFormat {
    name: &'static str,
    stdout: &'static str,
    when: &'static str,
}

#[derive(Debug, Serialize)]
struct ExitCodeInfo {
    code: i32,
    category: &'static str,
    description: &'static str,
}

#[derive(Debug, Serialize)]
struct FieldSelectorInfo {
    command: &'static str,
    default_fields: Vec<&'static str>,
    available_fields: Vec<&'static str>,
}

fn output_formats() -> Vec<OutputFormat> {
    vec![
        OutputFormat {
            name: "json",
            stdout: "compact JSON payload",
            when: "normal command output and application errors",
        },
        OutputFormat {
            name: "usage_json",
            stdout: "ErrorBody JSON with usage.* codes",
            when: "clap usage errors when SCHWAB_AGENT_JSON_ERRORS=1",
        },
        OutputFormat {
            name: "shell_completion",
            stdout: "raw shell completion script",
            when: "completions or completion command only",
        },
    ]
}

fn environment_variables() -> Vec<EnvironmentVariable> {
    vec![
        EnvironmentVariable {
            name: "SCHWAB_CLIENT_ID",
            purpose: "OAuth client ID; overrides config file",
            sensitive: true,
        },
        EnvironmentVariable {
            name: "SCHWAB_CLIENT_SECRET",
            purpose: "OAuth client secret; overrides config file",
            sensitive: true,
        },
        EnvironmentVariable {
            name: "SCHWAB_CALLBACK_URL",
            purpose: "OAuth callback URL; overrides config file and default",
            sensitive: false,
        },
        EnvironmentVariable {
            name: "SCHWAB_TOKEN_PATH",
            purpose: "Token file path override; empty values are ignored",
            sensitive: false,
        },
        EnvironmentVariable {
            name: "SCHWAB_AGENT_JSON_ERRORS",
            purpose: "Render clap usage errors as ErrorBody JSON on stdout when truthy",
            sensitive: false,
        },
        EnvironmentVariable {
            name: "XDG_CONFIG_HOME",
            purpose: "Base directory for config.json and compatibility token path",
            sensitive: false,
        },
        EnvironmentVariable {
            name: "XDG_STATE_HOME",
            purpose: "Base directory for saved order preview files",
            sensitive: false,
        },
        EnvironmentVariable {
            name: "RUST_LOG",
            purpose: "Enable tracing diagnostics on stderr without changing JSON stdout",
            sensitive: false,
        },
    ]
}

fn commands() -> Vec<CommandInfo> {
    use CommandClassification::{LocalOnly, Mutating, ReadOnly};

    vec![
        CommandInfo {
            name: "schema",
            classification: LocalOnly,
            description: "emit this machine-readable discovery schema",
        },
        CommandInfo {
            name: "doctor",
            classification: LocalOnly,
            description: "inspect sanitized config, auth, token, and debug health",
        },
        CommandInfo {
            name: "config status",
            classification: LocalOnly,
            description: "emit sanitized setup status as JSON",
        },
        CommandInfo {
            name: "config show",
            classification: LocalOnly,
            description: "emit the same sanitized setup status as config status",
        },
        CommandInfo {
            name: "completions",
            classification: LocalOnly,
            description: "generate shell completion scripts; completion is a singular alias",
        },
        CommandInfo {
            name: "completion",
            classification: LocalOnly,
            description: "singular alias for completions",
        },
        CommandInfo {
            name: "auth status",
            classification: LocalOnly,
            description: "inspect local token state without printing secrets",
        },
        CommandInfo {
            name: "auth login",
            classification: LocalOnly,
            description: "open browser, receive OAuth callback, and save a local token",
        },
        CommandInfo {
            name: "auth login-url",
            classification: LocalOnly,
            description: "build and optionally open the OAuth authorization URL",
        },
        CommandInfo {
            name: "auth exchange",
            classification: LocalOnly,
            description: "exchange a browser redirect URL for a saved token",
        },
        CommandInfo {
            name: "auth refresh",
            classification: LocalOnly,
            description: "refresh the saved token file",
        },
        CommandInfo {
            name: "market quote",
            classification: ReadOnly,
            description: "fetch quote data",
        },
        CommandInfo {
            name: "quote",
            classification: ReadOnly,
            description: "alias for market quote",
        },
        CommandInfo {
            name: "market history",
            classification: ReadOnly,
            description: "fetch price-history candles",
        },
        CommandInfo {
            name: "history",
            classification: ReadOnly,
            description: "alias for market history",
        },
        CommandInfo {
            name: "option expirations",
            classification: ReadOnly,
            description: "fetch option expiration dates",
        },
        CommandInfo {
            name: "option chain",
            classification: ReadOnly,
            description: "fetch and filter an option chain",
        },
        CommandInfo {
            name: "option screen",
            classification: ReadOnly,
            description: "screen option chains with liquidity and pricing filters",
        },
        CommandInfo {
            name: "option contract",
            classification: ReadOnly,
            description: "look up a single option contract",
        },
        CommandInfo {
            name: "ta dashboard",
            classification: ReadOnly,
            description: "fetch candles and compute technical indicators",
        },
        CommandInfo {
            name: "ta expected-move",
            classification: ReadOnly,
            description: "estimate expected move from option straddle pricing",
        },
        CommandInfo {
            name: "analyze",
            classification: ReadOnly,
            description: "combine quote and technical-analysis data for one or more symbols",
        },
        CommandInfo {
            name: "account",
            classification: ReadOnly,
            description: "list account summaries, positions, or resolve account selectors",
        },
        CommandInfo {
            name: "positions",
            classification: ReadOnly,
            description: "alias for account --positions",
        },
        CommandInfo {
            name: "transactions",
            classification: ReadOnly,
            description: "get recent account transactions",
        },
        CommandInfo {
            name: "order get",
            classification: ReadOnly,
            description: "inspect active, filtered, or specific orders",
        },
        CommandInfo {
            name: "orders",
            classification: ReadOnly,
            description: "alias for order get",
        },
        CommandInfo {
            name: "stock buy",
            classification: LocalOnly,
            description: "legacy migration stub; use order equity buy",
        },
        CommandInfo {
            name: "stock sell",
            classification: LocalOnly,
            description: "legacy migration stub; use order equity sell",
        },
        CommandInfo {
            name: "order equity",
            classification: Mutating,
            description: "build or place equity orders; --dry-run, --preview, and no-account modes are local-only",
        },
        CommandInfo {
            name: "order option",
            classification: Mutating,
            description: "build or place single-leg option orders; --dry-run, --preview, and no-account modes are local-only",
        },
        CommandInfo {
            name: "order preview-raw",
            classification: ReadOnly,
            description: "call Schwab previewOrder for raw JSON and optionally save a digest",
        },
        CommandInfo {
            name: "order place-from-preview",
            classification: Mutating,
            description: "place a saved preview payload by digest",
        },
        CommandInfo {
            name: "order place-raw",
            classification: Mutating,
            description: "place an arbitrary raw JSON order payload",
        },
        CommandInfo {
            name: "order replace",
            classification: Mutating,
            description: "replace an existing order",
        },
        CommandInfo {
            name: "order repeat",
            classification: Mutating,
            description: "rebuild and submit or preview a historical order",
        },
        CommandInfo {
            name: "order cancel",
            classification: Mutating,
            description: "cancel an existing order",
        },
    ]
}

fn exit_codes() -> Vec<ExitCodeInfo> {
    vec![
        ExitCodeInfo {
            code: 0,
            category: "success",
            description: "command completed successfully",
        },
        ExitCodeInfo {
            code: 1,
            category: "runtime",
            description: "network, decoding, completion-write, or unexpected runtime failure",
        },
        ExitCodeInfo {
            code: 2,
            category: "usage",
            description: "clap usage error; use SCHWAB_AGENT_JSON_ERRORS=1 for JSON usage errors",
        },
        ExitCodeInfo {
            code: 3,
            category: "auth",
            description: "missing, expired, or invalid authentication state",
        },
        ExitCodeInfo {
            code: 4,
            category: "schwab",
            description: "Schwab HTTP status error",
        },
        ExitCodeInfo {
            code: 10,
            category: "validation",
            description: "input, account, market, option, TA, or mutable-config validation failed",
        },
        ExitCodeInfo {
            code: 11,
            category: "order",
            description: "saved preview load, verification, TTL, or digest validation failed",
        },
        ExitCodeInfo {
            code: 20,
            category: "local",
            description: "local I/O, JSON, config, response-shape, or calculation error",
        },
    ]
}

fn field_selectors() -> Vec<FieldSelectorInfo> {
    vec![
        FieldSelectorInfo {
            command: "market quote",
            default_fields: market::DEFAULT_QUOTE_FIELDS.to_vec(),
            available_fields: market::available_quote_fields(),
        },
        FieldSelectorInfo {
            command: "market history",
            default_fields: market::DEFAULT_HISTORY_FIELDS.to_vec(),
            available_fields: market::available_history_fields(),
        },
        FieldSelectorInfo {
            command: "option chain",
            default_fields: options::types::CHAIN_FIELDS.to_vec(),
            available_fields: options::types::available_fields(),
        },
        FieldSelectorInfo {
            command: "option screen",
            default_fields: options::types::SCREEN_FIELDS.to_vec(),
            available_fields: options::types::available_fields(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_includes_required_discovery_sections() {
        let value = schema().expect("schema serializes");

        assert_eq!(value["name"], "schwab-agent");
        assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
        assert!(value["commands"].as_array().is_some_and(|commands| {
            commands.iter().any(|command| command["name"] == "schema")
                && commands
                    .iter()
                    .any(|command| command["classification"] == "mutating")
        }));
        assert!(
            value["environment_variables"]
                .as_array()
                .is_some_and(|vars| {
                    vars.iter()
                        .any(|var| var["name"] == "SCHWAB_AGENT_JSON_ERRORS")
                })
        );
        assert!(
            value["field_selectors"]
                .as_array()
                .is_some_and(|selectors| {
                    selectors
                        .iter()
                        .any(|selector| selector["command"] == "option chain")
                })
        );
        assert!(value["exit_codes"].as_array().is_some_and(|codes| {
            codes
                .iter()
                .any(|code| code["code"] == 2 && code["category"] == "usage")
        }));
    }

    #[test]
    fn doctor_wraps_sanitized_config_status() {
        let value = doctor().expect("doctor serializes");

        assert_eq!(value["status"], "ok");
        assert!(value["config"]["config_path"].as_str().is_some());
        assert!(
            value["summary"]
                .as_str()
                .is_some_and(|summary| { summary.contains("without reading account data") })
        );
    }
}
