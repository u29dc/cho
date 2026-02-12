//! Organisation commands: get.

use std::time::Instant;

use clap::Subcommand;

use crate::context::CliContext;

/// Organisation subcommands.
#[derive(Debug, Subcommand)]
pub enum OrganisationCommands {
    /// Get the current organisation details.
    Get,
}

/// Returns the tool name for the given subcommand.
pub fn tool_name(cmd: &OrganisationCommands) -> &'static str {
    match cmd {
        OrganisationCommands::Get => "organisation.get",
    }
}

/// Runs an organisation subcommand.
pub async fn run(
    cmd: &OrganisationCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        OrganisationCommands::Get => {
            let org = ctx.client().organisations().get().await?;
            ctx.emit_success("organisation.get", &org, start)?;
            Ok(())
        }
    }
}
