//! Contact commands: list, get, search.

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;

use crate::context::CliContext;

/// Contact subcommands.
#[derive(Debug, Subcommand)]
pub enum ContactCommands {
    /// List contacts.
    List {
        /// OData where filter expression.
        #[arg(long)]
        r#where: Option<String>,
    },
    /// Get a single contact by ID.
    Get {
        /// Contact ID (UUID).
        id: Uuid,
    },
    /// Search contacts by name, email, etc.
    Search {
        /// Search term.
        term: String,
    },
}

/// Runs a contact subcommand.
pub async fn run(cmd: &ContactCommands, ctx: &CliContext) -> cho_sdk::error::Result<()> {
    match cmd {
        ContactCommands::List { r#where } => {
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let pagination = ctx.pagination_params();
            let contacts = ctx.client().contacts().list(&params, &pagination).await?;
            let output = ctx.format_list_output(&contacts)?;
            println!("{output}");
            Ok(())
        }
        ContactCommands::Get { id } => {
            let contact = ctx.client().contacts().get(*id).await?;
            let output = ctx.format_output(&contact)?;
            println!("{output}");
            Ok(())
        }
        ContactCommands::Search { term } => {
            let pagination = ctx.pagination_params();
            let contacts = ctx.client().contacts().search(term, &pagination).await?;
            let output = ctx.format_list_output(&contacts)?;
            println!("{output}");
            Ok(())
        }
    }
}
