//! Payment commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::CliContext;

/// Payment subcommands.
#[derive(Debug, Subcommand)]
pub enum PaymentCommands {
    /// List payments.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,
    },
    /// Get a single payment by ID.
    Get {
        /// Payment ID (UUID).
        id: Uuid,
    },
}

/// Runs a payment subcommand.
pub async fn run(cmd: &PaymentCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        PaymentCommands::List { r#where } => {
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let pagination = ctx.pagination_params();
            let payments = ctx.client().payments().list(&params, &pagination).await?;
            let output = ctx.format_list_output(&payments)?;
            println!("{output}");
            Ok(())
        }
        PaymentCommands::Get { id } => {
            let payment = ctx.client().payments().get(*id).await?;
            let output = ctx.format_output(&payment)?;
            println!("{output}");
            Ok(())
        }
    }
}
