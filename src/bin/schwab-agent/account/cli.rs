use clap::Args;

/// Arguments for `account`.
#[derive(Debug, Args)]
pub struct AccountArgs {
    /// Account hash or nickname to resolve. Omit to list account summaries.
    pub selector: Option<String>,

    /// Include individual positions in each account summary.
    #[arg(long)]
    pub positions: bool,
}

/// Arguments for the top-level `positions` alias.
#[derive(Debug, Args)]
pub struct PositionsArgs {
    /// Account hash or nickname to inspect. Omit to list positions for all accounts.
    pub selector: Option<String>,
}

impl From<&PositionsArgs> for AccountArgs {
    fn from(args: &PositionsArgs) -> Self {
        Self {
            selector: args.selector.clone(),
            positions: true,
        }
    }
}

impl AccountArgs {
    /// Whether position data should be fetched from the API.
    ///
    /// Returns `true` when position data is explicitly requested.
    #[must_use]
    pub fn include_positions(&self) -> bool {
        self.positions
    }

    /// Whether the selector should return an account summary instead of a hash resolution.
    #[must_use]
    pub fn requests_summary(&self) -> bool {
        self.include_positions()
    }
}
