//! Credit note commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::CliContext;

/// Credit note subcommands.
#[derive(Debug, Subcommand)]
pub enum CreditNoteCommands {
    /// List credit notes.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,

        /// Order expression (e.g. "Date DESC").
        #[arg(long)]
        order: Option<String>,
    },
    /// Get a single credit note by ID.
    Get {
        /// Credit note UUID.
        id: Uuid,
    },
}

/// Runs a credit note subcommand.
pub async fn run(cmd: &CreditNoteCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        CreditNoteCommands::List { r#where, order } => {
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            if let Some(o) = order {
                params = params.with_order(o.clone());
            }
            let pagination = ctx.pagination_params();
            let items = ctx.client().credit_notes().list(&params, &pagination).await?;
            let output = ctx.format_list_output(&items)?;
            println!("{output}");
            Ok(())
        }
        CreditNoteCommands::Get { id } => {
            let item = ctx.client().credit_notes().get(*id).await?;
            let output = ctx.format_output(&item)?;
            println!("{output}");
            Ok(())
        }
    }
}
