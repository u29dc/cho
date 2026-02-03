//! Currency commands: list.

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

/// Runs a currency subcommand.
pub async fn run(cmd: &CurrencyCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        CurrencyCommands::List { r#where } => {
            warn_if_suspicious_filter(r#where.as_ref());
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let items = ctx.client().currencies().list(&params).await?;
            let output = ctx.format_list_output(&items)?;
            println!("{output}");
            Ok(())
        }
    }
}
