//! Quote commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::CliContext;

/// Quote subcommands.
#[derive(Debug, Subcommand)]
pub enum QuoteCommands {
    /// List quotes.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,

        /// Order expression (e.g. "Date DESC").
        #[arg(long)]
        order: Option<String>,
    },
    /// Get a single quote by ID.
    Get {
        /// Quote UUID.
        id: Uuid,
    },
}

/// Runs a quote subcommand.
pub async fn run(cmd: &QuoteCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        QuoteCommands::List { r#where, order } => {
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            if let Some(o) = order {
                params = params.with_order(o.clone());
            }
            let pagination = ctx.pagination_params();
            let items = ctx.client().quotes().list(&params, &pagination).await?;
            let output = ctx.format_list_output(&items)?;
            println!("{output}");
            Ok(())
        }
        QuoteCommands::Get { id } => {
            let item = ctx.client().quotes().get(*id).await?;
            let output = ctx.format_output(&item)?;
            println!("{output}");
            Ok(())
        }
    }
}
