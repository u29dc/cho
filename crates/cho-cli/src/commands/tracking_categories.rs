//! Tracking category commands: list, get.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::{CliContext, warn_if_suspicious_filter};

/// Tracking category subcommands.
#[derive(Debug, Subcommand)]
pub enum TrackingCategoryCommands {
    /// List tracking categories.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,
    },
    /// Get a single tracking category by ID.
    Get {
        /// Tracking category UUID.
        id: Uuid,
    },
}

/// Runs a tracking category subcommand.
pub async fn run(cmd: &TrackingCategoryCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        TrackingCategoryCommands::List { r#where } => {
            warn_if_suspicious_filter(r#where.as_ref());
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let items = ctx.client().tracking_categories().list(&params).await?;
            let output = ctx.format_list_output(&items)?;
            println!("{output}");
            Ok(())
        }
        TrackingCategoryCommands::Get { id } => {
            let item = ctx.client().tracking_categories().get(*id).await?;
            let output = ctx.format_output(&item)?;
            println!("{output}");
            Ok(())
        }
    }
}
