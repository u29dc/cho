//! Item commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::CliContext;

/// Item subcommands.
#[derive(Debug, Subcommand)]
pub enum ItemCommands {
    /// List items.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,
    },
    /// Get a single item by ID.
    Get {
        /// Item UUID.
        id: Uuid,
    },
}

/// Runs an item subcommand.
pub async fn run(cmd: &ItemCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        ItemCommands::List { r#where } => {
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let items = ctx.client().items().list(&params).await?;
            let output = ctx.format_list_output(&items)?;
            println!("{output}");
            Ok(())
        }
        ItemCommands::Get { id } => {
            let item = ctx.client().items().get(*id).await?;
            let output = ctx.format_output(&item)?;
            println!("{output}");
            Ok(())
        }
    }
}
