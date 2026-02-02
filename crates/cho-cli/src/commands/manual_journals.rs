//! Manual journal commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::CliContext;

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

/// Runs a manual journal subcommand.
pub async fn run(cmd: &ManualJournalCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        ManualJournalCommands::List { r#where, order } => {
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
            let output = ctx.format_paginated_output(&items)?;
            println!("{output}");
            Ok(())
        }
        ManualJournalCommands::Get { id } => {
            let item = ctx.client().manual_journals().get(*id).await?;
            let output = ctx.format_output(&item)?;
            println!("{output}");
            Ok(())
        }
    }
}
