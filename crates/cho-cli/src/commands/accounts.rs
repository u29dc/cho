//! Account commands: list.

use std::time::Instant;

use clap::Subcommand;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Account subcommands.
#[derive(Debug, Subcommand)]
pub enum AccountCommands {
    /// List chart of accounts.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,
    },
}

/// Returns the tool name for an account subcommand.
pub fn tool_name(cmd: &AccountCommands) -> &'static str {
    match cmd {
        AccountCommands::List { .. } => "accounts.list",
    }
}

/// Runs an account subcommand.
pub async fn run(
    cmd: &AccountCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        AccountCommands::List { r#where } => {
            warn_if_suspicious_filter(r#where.as_ref());
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let accounts = ctx.client().accounts().list(&params).await?;
            ctx.emit_items("accounts.list", &accounts, start)?;
            Ok(())
        }
    }
}
