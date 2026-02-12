//! Manual journal commands: list, get.

use std::time::Instant;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Manual journal subcommands.
#[derive(Debug, Subcommand)]
pub enum ManualJournalCommands {
    /// List manual journals.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,

        /// Order expression (e.g. "Date DESC").
        #[arg(long)]
        order: Option<String>,
    },
    /// Get a single manual journal by ID.
    Get {
        /// Manual journal UUID.
        id: Uuid,
    },
}

/// Returns the tool name for the given subcommand.
pub fn tool_name(cmd: &ManualJournalCommands) -> &'static str {
    match cmd {
        ManualJournalCommands::List { .. } => "manual-journals.list",
        ManualJournalCommands::Get { .. } => "manual-journals.get",
    }
}

/// Runs a manual journal subcommand.
pub async fn run(
    cmd: &ManualJournalCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        ManualJournalCommands::List { r#where, order } => {
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
                .manual_journals()
                .list(&params, &pagination)
                .await?;
            ctx.emit_list("manual-journals.list", &items, start)?;
            Ok(())
        }
        ManualJournalCommands::Get { id } => {
            let item = ctx.client().manual_journals().get(*id).await?;
            ctx.emit_success("manual-journals.get", &item, start)?;
            Ok(())
        }
    }
}
