//! Currency commands: list.

use std::time::Instant;

use clap::Subcommand;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Currency subcommands.
#[derive(Debug, Subcommand)]
pub enum CurrencyCommands {
    /// List currencies.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,
    },
}

/// Returns the tool name for a currency subcommand.
pub fn tool_name(cmd: &CurrencyCommands) -> &'static str {
    match cmd {
        CurrencyCommands::List { .. } => "currencies.list",
    }
}

/// Runs a currency subcommand.
pub async fn run(
    cmd: &CurrencyCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        CurrencyCommands::List { r#where } => {
            warn_if_suspicious_filter(r#where.as_ref());
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let items = ctx.client().currencies().list(&params).await?;
            ctx.emit_items("currencies.list", &items, start)?;
            Ok(())
        }
    }
}
