//! Shell completion generation for the `schwab-agent` command tree.

use std::io;

use clap::CommandFactory;
use clap_complete::generate;

use crate::cli::{Cli, CompletionsArgs};

/// Writes a shell completion script for the requested shell.
pub fn write<W>(args: &CompletionsArgs, writer: &mut W) -> io::Result<i32>
where
    W: io::Write,
{
    let mut command = Cli::command();
    generate(args.shell, &mut command, "schwab-agent", writer);
    Ok(0)
}
