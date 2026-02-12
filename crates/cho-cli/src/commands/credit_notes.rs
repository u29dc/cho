//! Credit note commands: list, get.

use std::time::Instant;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

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

/// Returns the tool name for a credit note subcommand.
pub fn tool_name(cmd: &CreditNoteCommands) -> &'static str {
    match cmd {
        CreditNoteCommands::List { .. } => "credit-notes.list",
        CreditNoteCommands::Get { .. } => "credit-notes.get",
    }
}

/// Runs a credit note subcommand.
pub async fn run(
    cmd: &CreditNoteCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        CreditNoteCommands::List { r#where, order } => {
            warn_if_suspicious_filter(r#where.as_ref());
            warn_if_suspicious_filter(order.as_ref());
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            if let Some(o) = order {
                params = params.with_order(o.clone());
            }
            let pagination = ctx.pagination_params();
            let items = ctx
                .client()
                .credit_notes()
                .list(&params, &pagination)
                .await?;
            ctx.emit_list("credit-notes.list", &items, start)?;
            Ok(())
        }
        CreditNoteCommands::Get { id } => {
            let item = ctx.client().credit_notes().get(*id).await?;
            ctx.emit_success("credit-notes.get", &item, start)?;
            Ok(())
        }
    }
}
