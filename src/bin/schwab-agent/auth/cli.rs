use clap::{Args, Subcommand};

/// Authentication commands.
#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Show local token state without printing secrets.
    Status,
    /// Full interactive login: open browser, wait for a complete callback, exchange and save token.
    Login(LoginArgs),
    /// Build a browser authorization URL and open it in the default browser.
    LoginUrl(LoginUrlArgs),
    /// Exchange a pasted browser redirect URL for a saved token file.
    Exchange(AuthExchangeArgs),
    /// Force-refresh the saved token file.
    Refresh,
}

/// Arguments for `auth login`.
#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Skip opening the authorization URL in the default browser.
    #[arg(long)]
    pub no_browser: bool,

    /// Seconds to wait for the callback before timing out.
    #[arg(long, default_value = "300")]
    pub timeout: u64,
}

/// Arguments for `auth login-url`.
#[derive(Debug, Args)]
pub struct LoginUrlArgs {
    /// Skip opening the authorization URL in the default browser.
    #[arg(long)]
    pub no_browser: bool,
}

/// Arguments for `auth exchange`.
#[derive(Debug, Args)]
pub struct AuthExchangeArgs {
    /// CSRF state returned by `auth login-url`.
    #[arg(long)]
    pub state: String,

    /// Full redirect URL copied from the browser address bar.
    #[arg(long)]
    pub redirect_url: String,
}
