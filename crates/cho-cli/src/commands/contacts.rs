//! Contact commands: list, get, search, create, update.

use std::path::PathBuf;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;
use cho_sdk::models::contact::Contact;

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
    /// Create a new contact from a JSON file.
    Create {
        /// Path to JSON file containing the contact data.
        #[arg(long)]
        file: PathBuf,
        /// Idempotency key for safe retries.
        #[arg(long)]
        idempotency_key: Option<String>,
    },
    /// Update an existing contact from a JSON file.
    Update {
        /// Contact ID (UUID) to update.
        id: Uuid,
        /// Path to JSON file containing the contact update data.
        #[arg(long)]
        file: PathBuf,
        /// Idempotency key for safe retries.
        #[arg(long)]
        idempotency_key: Option<String>,
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
            let output = ctx.format_paginated_output(&contacts)?;
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
            let output = ctx.format_paginated_output(&contacts)?;
            println!("{output}");
            Ok(())
        }
        ContactCommands::Create {
            file,
            idempotency_key,
        } => {
            ctx.require_writes_allowed()?;
            let contact: Contact = crate::commands::invoices::read_json_file(file)?;
            let result = ctx
                .client()
                .contacts()
                .create(&contact, idempotency_key.as_deref())
                .await?;
            let output = ctx.format_output(&result)?;
            println!("{output}");
            Ok(())
        }
        ContactCommands::Update {
            id,
            file,
            idempotency_key,
        } => {
            ctx.require_writes_allowed()?;
            let contact: Contact = crate::commands::invoices::read_json_file(file)?;
            let result = ctx
                .client()
                .contacts()
                .update(*id, &contact, idempotency_key.as_deref())
                .await?;
            let output = ctx.format_output(&result)?;
            println!("{output}");
            Ok(())
        }
    }
}
