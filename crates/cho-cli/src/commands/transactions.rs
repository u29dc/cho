//! Bank transaction commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::CliContext;

/// Transaction subcommands.
#[derive(Debug, Subcommand)]
pub enum TransactionCommands {
    /// List bank transactions.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,

        /// Filter from date (YYYY-MM-DD).
        #[arg(long)]
        from: Option<String>,

        /// Filter to date (YYYY-MM-DD).
        #[arg(long)]
        to: Option<String>,
    },
    /// Get a single bank transaction by ID.
    Get {
        /// Bank transaction ID (UUID).
        id: Uuid,
    },
}

/// Runs a transaction subcommand.
pub async fn run(cmd: &TransactionCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        TransactionCommands::List { r#where, from, to } => {
            let mut params = ListParams::new();
            let mut where_parts = Vec::new();
            if let Some(w) = r#where {
                where_parts.push(w.clone());
            }
            if let Some(from_date) = from {
                where_parts.push(format!("Date >= DateTime({from_date})"));
            }
            if let Some(to_date) = to {
                where_parts.push(format!("Date <= DateTime({to_date})"));
            }
            if !where_parts.is_empty() {
                params = params.with_where(where_parts.join(" AND "));
            }

            let pagination = ctx.pagination_params();
            let txns = ctx
                .client()
                .bank_transactions()
                .list(&params, &pagination)
                .await?;
            let output = ctx.format_list_output(&txns)?;
            println!("{output}");
            Ok(())
        }
        TransactionCommands::Get { id } => {
            let txn = ctx.client().bank_transactions().get(*id).await?;
            let output = ctx.format_output(&txn)?;
            println!("{output}");
            Ok(())
        }
    }
}
