//! Tax rate commands: list.

use clap::Subcommand;

use cho_sdk::http::request::ListParams;

use crate::context::CliContext;

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

/// Runs a tax rate subcommand.
pub async fn run(cmd: &TaxRateCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        TaxRateCommands::List { r#where } => {
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let items = ctx.client().tax_rates().list(&params).await?;
            let output = ctx.format_list_output(&items)?;
            println!("{output}");
            Ok(())
        }
    }
}
