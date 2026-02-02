//! Organisation commands: get.

use clap::Subcommand;

use crate::context::CliContext;

/// Organisation subcommands.
#[derive(Debug, Subcommand)]
pub enum OrganisationCommands {
    /// Get the current organisation details.
    Get,
}

/// Runs an organisation subcommand.
pub async fn run(cmd: &OrganisationCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        OrganisationCommands::Get => {
            let org = ctx.client().organisations().get().await?;
            let output = ctx.format_output(&org)?;
            println!("{output}");
            Ok(())
        }
    }
}
