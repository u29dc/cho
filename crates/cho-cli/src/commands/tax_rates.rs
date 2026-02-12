//! Tax rate commands: list.

use std::time::Instant;

use clap::Subcommand;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Tax rate subcommands.
#[derive(Debug, Subcommand)]
pub enum TaxRateCommands {
    /// List tax rates.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,
    },
}

/// Returns the tool name for a tax rate subcommand.
pub fn tool_name(cmd: &TaxRateCommands) -> &'static str {
    match cmd {
        TaxRateCommands::List { .. } => "tax-rates.list",
    }
}

/// Runs a tax rate subcommand.
pub async fn run(
    cmd: &TaxRateCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        TaxRateCommands::List { r#where } => {
            warn_if_suspicious_filter(r#where.as_ref());
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let items = ctx.client().tax_rates().list(&params).await?;
            ctx.emit_items("tax-rates.list", &items, start)?;
            Ok(())
        }
    }
}
