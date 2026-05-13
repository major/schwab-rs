use std::env;
use std::path::PathBuf;
use std::process::Command;

use schwab::auth::{AuthConfig, FileTokenStore, login};

#[tokio::main]
async fn main() -> schwab::Result<()> {
    let client_id = env::var("SCHWAB_CLIENT_ID").expect("set SCHWAB_CLIENT_ID");
    let client_secret = env::var("SCHWAB_CLIENT_SECRET").expect("set SCHWAB_CLIENT_SECRET");
    let callback_url = env::var("SCHWAB_CALLBACK_URL")
        .unwrap_or_else(|_| "https://127.0.0.1:8182/callback".to_string());
    let token_path = env::var_os("SCHWAB_TOKEN_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("schwab-token.json"));

    let config = AuthConfig::new(client_id, client_secret, callback_url)?;
    let store = FileTokenStore::new(token_path);
    let provider = login(config, store, |url| {
        println!("Open this URL if your browser does not start automatically:\n{url}");
        open_browser(url);
        Ok(())
    })
    .await?;

    let token = provider.token().await?;
    println!(
        "Token saved. Current access token has {} characters.",
        token.len()
    );
    Ok(())
}

fn open_browser(url: &str) {
    let status = browser_command(url).status().ok();

    if !matches!(status, Some(status) if status.success()) {
        println!("Browser launch failed. Copy the URL above into your browser.");
    }
}

fn browser_command(url: &str) -> Command {
    if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg(url);
        command
    } else if cfg!(target_os = "windows") {
        let mut command = Command::new("rundll32.exe");
        command.args(["url.dll,FileProtocolHandler", url]);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(url);
        command
    }
}
