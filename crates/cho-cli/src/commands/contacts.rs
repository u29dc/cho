//! Contact commands: list, get, search, create, update.

use std::path::PathBuf;
use std::time::Instant;

use clap::Subcommand;
use uuid::Uuid;

use cho_sdk::http::request::ListParams;
use cho_sdk::models::contact::Contact;

use crate::context::{CliContext, warn_if_suspicious_filter};

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

/// Returns the tool name for a contact subcommand.
pub fn tool_name(cmd: &ContactCommands) -> &'static str {
    match cmd {
        ContactCommands::List { .. } => "contacts.list",
        ContactCommands::Get { .. } => "contacts.get",
        ContactCommands::Search { .. } => "contacts.search",
        ContactCommands::Create { .. } => "contacts.create",
        ContactCommands::Update { .. } => "contacts.update",
    }
}

/// Runs a contact subcommand.
pub async fn run(
    cmd: &ContactCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        ContactCommands::List { r#where } => {
            warn_if_suspicious_filter(r#where.as_ref());
            let mut params = ListParams::new();
            if let Some(w) = r#where {
                params = params.with_where(w.clone());
            }
            let pagination = ctx.pagination_params();
            let contacts = ctx.client().contacts().list(&params, &pagination).await?;
            ctx.emit_list("contacts.list", &contacts, start)?;
            Ok(())
        }
        ContactCommands::Get { id } => {
            let contact = ctx.client().contacts().get(*id).await?;
            ctx.emit_success("contacts.get", &contact, start)?;
            Ok(())
        }
        ContactCommands::Search { term } => {
            let pagination = ctx.pagination_params();
            let contacts = ctx.client().contacts().search(term, &pagination).await?;
            ctx.emit_list("contacts.search", &contacts, start)?;
            Ok(())
        }
        ContactCommands::Create {
            file,
            idempotency_key,
        } => {
            ctx.require_writes_allowed()?;
            let contact: Contact = crate::commands::utils::read_json_file(file)?;
            let result = ctx
                .client()
                .contacts()
                .create(&contact, idempotency_key.as_deref())
                .await?;
            ctx.emit_success("contacts.create", &result, start)?;
            Ok(())
        }
        ContactCommands::Update {
            id,
            file,
            idempotency_key,
        } => {
            ctx.require_writes_allowed()?;
            let contact: Contact = crate::commands::utils::read_json_file(file)?;
            let result = ctx
                .client()
                .contacts()
                .update(*id, &contact, idempotency_key.as_deref())
                .await?;
            ctx.emit_success("contacts.update", &result, start)?;
            Ok(())
        }
    }
}
