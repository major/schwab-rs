/// Fetches quotes using a previously saved token file.
///
/// Assumes you have already run the `auth` example to create the token file.
///
/// Environment variables:
///   SCHWAB_CLIENT_ID       - Schwab app client ID
///   SCHWAB_CLIENT_SECRET   - Schwab app client secret
///   SCHWAB_CALLBACK_URL    - OAuth callback URL (default: https://127.0.0.1:8182/callback)
///   SCHWAB_TOKEN_PATH      - Token file path (default: schwab-token.json)
use std::env;
use std::path::PathBuf;

use schwab::auth::{AuthConfig, Provider};

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
    let provider = Provider::from_token_file(config, token_path)?;
    let client = provider.client().await?;

    let symbols = env::args().skip(1).collect::<Vec<_>>();
    let symbols = if symbols.is_empty() {
        vec!["AAPL".to_string(), "MSFT".to_string(), "$SPX".to_string()]
    } else {
        symbols
    };

    let quotes = client.get_quotes(&symbols).await?;

    for (symbol, quote) in &quotes {
        println!("{symbol}: {quote:#?}");
    }

    Ok(())
}
