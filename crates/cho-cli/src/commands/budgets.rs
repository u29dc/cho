//! Budget commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::CliContext;

/// Budget subcommands.
#[derive(Debug, Subcommand)]
pub enum BudgetCommands {
    /// List budgets.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,
    },
    /// Get a single budget by ID.
    Get {
        /// Budget UUID.
        id: Uuid,
    },
}

/// Runs a budget subcommand.
pub async fn run(cmd: &BudgetCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        BudgetCommands::List { r#where } => {
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let items = ctx.client().budgets().list(&params).await?;
            let output = ctx.format_list_output(&items)?;
            println!("{output}");
            Ok(())
        }
        BudgetCommands::Get { id } => {
            let item = ctx.client().budgets().get(*id).await?;
            let output = ctx.format_output(&item)?;
            println!("{output}");
            Ok(())
        }
    }
}
