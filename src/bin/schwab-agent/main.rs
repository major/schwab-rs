//! Agent-oriented JSON CLI porcelain for the `schwab` crate.

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod account;
mod analyze;
mod auth;
mod cli;
mod completions;
mod config;
mod error;
mod market;
mod options;
mod order;
mod output;
mod raw;
mod shared;
mod ta;
mod verify;

use std::{
    io::{self, Write},
    panic::{AssertUnwindSafe, catch_unwind},
};

use clap::Parser;
use serde_json::Value;

use crate::cli::{Cli, Command, OptionCommand, TaCommand};
use crate::error::AppError;
use crate::output::ErrorBody;

/// Parses process arguments, runs the selected command, and writes JSON output.
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn run_from_env() -> i32 {
    run(Cli::parse()).await
}

/// Runs the CLI from process arguments and exits with the command status code.
#[tokio::main]
async fn main() {
    std::process::exit(run_from_env().await);
}

/// Runs a parsed CLI command and writes the structured JSON result.
pub async fn run(cli: Cli) -> i32 {
    if let Command::Completions(args) = &cli.command {
        let mut stdout = io::stdout().lock();
        let mut stderr = io::stderr().lock();
        return write_completions(args, &mut stdout, &mut stderr);
    }

    let result = execute(cli).await;
    let mut stdout = io::stdout().lock();
    match result {
        Ok(data) => write_json(&mut stdout, &data).unwrap_or(1),
        Err(error) => {
            let code = error.exit_code();
            let body = ErrorBody::from(&error);
            let write_code = write_json(&mut stdout, &body).unwrap_or(1);
            if write_code == 0 { code } else { write_code }
        }
    }
}

/// Executes a command and returns the data payload directly.
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn execute(cli: Cli) -> Result<Value, AppError> {
    // Order commands produce their own data values with dynamic command names.
    if let Command::Order(command) = &cli.command {
        return order::handle(&cli, command).await;
    }
    if let Command::Analyze(args) = &cli.command {
        let client = auth::provider()?.client().await?;
        return analyze::analyze(&client, args).await;
    }

    match &cli.command {
        Command::Auth(command) => auth::handle(&cli, command).await,
        Command::Market(command) => market::handle(&cli, command).await,
        Command::Option(command) => {
            let client = auth::provider()?.client().await?;
            match command {
                OptionCommand::Expirations(args) => {
                    options::expirations::handle(&client, &args.symbol).await
                }
                OptionCommand::Chain(args) => options::chain::handle(&client, args).await,
                OptionCommand::Screen(args) => options::screen::handle(&client, args).await,
                OptionCommand::Contract(args) => options::contract::handle(&client, args).await,
            }
        }
        Command::Analyze(_) => unreachable!("handled above"),
        Command::Completions(_) => unreachable!("handled above"),
        Command::Ta(ta_cmd) => {
            let client = auth::provider()?.client().await?;
            match ta_cmd {
                TaCommand::Dashboard(args) => ta::dashboard(&client, args).await,
                TaCommand::ExpectedMove(args) => {
                    ta::expected_move::expected_move(&client, args).await
                }
            }
        }
        Command::Order(_) => unreachable!("handled above"),
        Command::Account(command) => account::handle(&cli, command).await,
    }
}

/// Serializes `value` as JSON and writes it to `writer` followed by a newline.
///
/// Returns `Ok(0)` on success, or an `io::Error` if the write fails.
fn write_json<W, T>(writer: &mut W, value: &T) -> Result<i32, io::Error>
where
    W: Write,
    T: serde::Serialize,
{
    serde_json::to_writer(&mut *writer, value)?;
    writer.write_all(b"\n")?;
    Ok(0)
}

fn write_completions<W, E>(args: &cli::CompletionsArgs, stdout: &mut W, stderr: &mut E) -> i32
where
    W: Write,
    E: Write,
{
    let result = catch_unwind(AssertUnwindSafe(|| completions::write(args, stdout)));
    match result {
        Ok(Ok(code)) => code,
        Ok(Err(_)) | Err(_) => {
            let _ = writeln!(stderr, "failed to write shell completions");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, path::Path};

    use clap::Parser;

    use crate::cli::Cli;

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set_path(key: &'static str, value: &Path) -> Self {
            let previous = std::env::var_os(key);
            unsafe {
                std::env::set_var(key, value);
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
    fn write_json_writes_json_followed_by_newline() {
        let mut buf: Vec<u8> = Vec::new();
        let value = serde_json::json!({"ok": true});
        let result = super::write_json(&mut buf, &value);

        assert_eq!(result.unwrap(), 0);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.ends_with('\n'));
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed["ok"], true);
    }

    #[test]
    fn write_json_returns_error_on_write_failure() {
        struct FailWriter;
        impl std::io::Write for FailWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("write failed"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::other("flush failed"))
            }
        }

        let mut writer = FailWriter;
        let value = serde_json::json!({"ok": true});
        assert!(super::write_json(&mut writer, &value).is_err());
    }

    #[test]
    fn write_completions_reports_write_failure() {
        struct FailWriter;
        impl std::io::Write for FailWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("write failed"))
            }

            #[cfg_attr(coverage_nightly, coverage(off))]
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let args = crate::cli::CompletionsArgs {
            shell: clap_complete::Shell::Bash,
        };
        let mut stdout = FailWriter;
        let mut stderr = Vec::new();

        let code = super::write_completions(&args, &mut stdout, &mut stderr);

        assert_eq!(code, 1);
        assert!(
            String::from_utf8(stderr)
                .unwrap()
                .contains("failed to write shell completions")
        );
    }

    #[test]
    fn run_returns_nonzero_on_missing_token_file() {
        let _lock = crate::config::TEST_ENV_LOCK.lock().unwrap();
        let token_path = Path::new("/tmp/schwab-test-nonexistent-token-file");
        let _token_path = EnvVarGuard::set_path("SCHWAB_TOKEN_PATH", token_path);
        let cli = Cli::parse_from(["schwab-agent", "auth", "refresh"]);
        let code = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(super::run(cli));
        assert_eq!(code, 3);
    }
}
