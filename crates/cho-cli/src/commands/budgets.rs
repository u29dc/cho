//! Budget commands: list, get.

use std::time::Instant;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

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

/// Returns the tool name for a budget subcommand.
pub fn tool_name(cmd: &BudgetCommands) -> &'static str {
    match cmd {
        BudgetCommands::List { .. } => "budgets.list",
        BudgetCommands::Get { .. } => "budgets.get",
    }
}

/// Runs a budget subcommand.
pub async fn run(
    cmd: &BudgetCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        BudgetCommands::List { r#where } => {
            warn_if_suspicious_filter(r#where.as_ref());
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let items = ctx.client().budgets().list(&params).await?;
            ctx.emit_items("budgets.list", &items, start)?;
            Ok(())
        }
        BudgetCommands::Get { id } => {
            let item = ctx.client().budgets().get(*id).await?;
            ctx.emit_success("budgets.get", &item, start)?;
            Ok(())
        }
    }
}
