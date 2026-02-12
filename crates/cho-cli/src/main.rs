#![forbid(unsafe_code)]

//! cho: Xero accounting CLI for AI agents and humans.

mod commands;
mod context;
mod envelope;
mod error;
mod output;
mod registry;

use std::io::IsTerminal;
use std::time::Instant;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use cho_sdk::auth::AuthManager;
use cho_sdk::client::XeroClient;
use cho_sdk::config::SdkConfig;
use cho_sdk::http::rate_limit::RateLimitConfig;

use crate::context::CliContext;
use crate::output::OutputFormat;
use crate::output::json::JsonOptions;

/// cho: Xero accounting CLI for AI agents and humans.
#[derive(Debug, Parser)]
#[command(name = "cho", version, about, long_about = None)]
struct Cli {
    /// Output format.
    #[arg(long, value_enum, global = true, env = "CHO_FORMAT")]
    format: Option<OutputFormat>,

    /// Shorthand for --format json.
    #[arg(long, global = true)]
    json: bool,

    /// Deprecated: JSON output always uses envelope. Accepted for compatibility.
    #[arg(long, global = true, hide = true)]
    meta: bool,

    /// Preserve Xero-native PascalCase keys, skip snake_case normalization.
    #[arg(long, global = true)]
    raw: bool,

    /// Emit money as strings instead of numbers.
    #[arg(long, global = true)]
    precise: bool,

    /// Override default tenant ID.
    #[arg(long, global = true, env = "CHO_TENANT_ID")]
    tenant: Option<String>,

    /// Enable verbose tracing output.
    #[arg(long, global = true)]
    verbose: bool,

    /// Suppress non-essential output.
    #[arg(long, global = true)]
    quiet: bool,

    /// Disable terminal colors.
    #[arg(long, global = true)]
    no_color: bool,

    /// Maximum items for list commands.
    #[arg(long, global = true, default_value = "100")]
    limit: usize,

    /// Fetch all pages, no limit.
    #[arg(long, global = true)]
    all: bool,

    /// Subcommand to run.
    #[command(subcommand)]
    command: Commands,
}

/// Top-level CLI commands.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Interactive first-time setup: authenticate and select a tenant.
    Init(commands::init::InitArgs),

    /// Show tool catalog for agent discovery.
    Tools {
        /// Tool name for detail view (e.g. "invoices.list").
        name: Option<String>,
    },

    /// Check CLI readiness (auth, config, connectivity).
    Health,

    /// Authentication management.
    Auth {
        #[command(subcommand)]
        command: commands::auth::AuthCommands,
    },
    /// Invoice operations.
    Invoices {
        #[command(subcommand)]
        command: commands::invoices::InvoiceCommands,
    },
    /// Contact operations.
    Contacts {
        #[command(subcommand)]
        command: commands::contacts::ContactCommands,
    },
    /// Payment operations.
    Payments {
        #[command(subcommand)]
        command: commands::payments::PaymentCommands,
    },
    /// Bank transaction operations.
    Transactions {
        #[command(subcommand)]
        command: commands::transactions::TransactionCommands,
    },
    /// Chart of accounts operations.
    Accounts {
        #[command(subcommand)]
        command: commands::accounts::AccountCommands,
    },
    /// Financial reports.
    Reports {
        #[command(subcommand)]
        command: commands::reports::ReportCommands,
    },
    /// Configuration management.
    Config {
        #[command(subcommand)]
        command: commands::config::ConfigCommands,
    },
    /// Credit note operations.
    #[command(name = "credit-notes")]
    CreditNotes {
        #[command(subcommand)]
        command: commands::credit_notes::CreditNoteCommands,
    },
    /// Quote operations.
    Quotes {
        #[command(subcommand)]
        command: commands::quotes::QuoteCommands,
    },
    /// Purchase order operations.
    #[command(name = "purchase-orders")]
    PurchaseOrders {
        #[command(subcommand)]
        command: commands::purchase_orders::PurchaseOrderCommands,
    },
    /// Item operations.
    Items {
        #[command(subcommand)]
        command: commands::items::ItemCommands,
    },
    /// Tax rate operations.
    #[command(name = "tax-rates")]
    TaxRates {
        #[command(subcommand)]
        command: commands::tax_rates::TaxRateCommands,
    },
    /// Currency operations.
    Currencies {
        #[command(subcommand)]
        command: commands::currencies::CurrencyCommands,
    },
    /// Tracking category operations.
    #[command(name = "tracking-categories")]
    TrackingCategories {
        #[command(subcommand)]
        command: commands::tracking_categories::TrackingCategoryCommands,
    },
    /// Organisation operations.
    #[command(name = "organisation")]
    Organisation {
        #[command(subcommand)]
        command: commands::organisations::OrganisationCommands,
    },
    /// Manual journal operations.
    #[command(name = "manual-journals")]
    ManualJournals {
        #[command(subcommand)]
        command: commands::manual_journals::ManualJournalCommands,
    },
    /// Prepayment operations.
    Prepayments {
        #[command(subcommand)]
        command: commands::prepayments::PrepaymentCommands,
    },
    /// Overpayment operations.
    Overpayments {
        #[command(subcommand)]
        command: commands::overpayments::OverpaymentCommands,
    },
    /// Linked transaction operations.
    #[command(name = "linked-transactions")]
    LinkedTransactions {
        #[command(subcommand)]
        command: commands::linked_transactions::LinkedTransactionCommands,
    },
    /// Budget operations.
    Budgets {
        #[command(subcommand)]
        command: commands::budgets::BudgetCommands,
    },
    /// Repeating invoice operations.
    #[command(name = "repeating-invoices")]
    RepeatingInvoices {
        #[command(subcommand)]
        command: commands::repeating_invoices::RepeatingInvoiceCommands,
    },
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let cli = Cli::parse();

    // Initialize tracing
    if cli.verbose {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }

    // Determine output format: --json flag > --format flag > auto-detect
    let format = if cli.json {
        OutputFormat::Json
    } else {
        cli.format.unwrap_or_else(|| {
            if std::io::stdout().is_terminal() {
                OutputFormat::Table
            } else {
                OutputFormat::Json
            }
        })
    };

    // Deprecation warning for --meta
    if cli.meta {
        eprintln!(
            "Warning: --meta is deprecated. JSON output now always uses the envelope format."
        );
    }

    let json_options = JsonOptions {
        raw: cli.raw,
        precise: cli.precise,
    };

    let is_json = format == OutputFormat::Json;

    // --- Early dispatch: commands that don't need a client ---

    // Init
    if let Commands::Init(ref args) = cli.command {
        if let Err(e) = commands::init::run(args).await {
            let msg = error::format_error(&e, is_json, "init", start);
            if is_json {
                println!("{msg}");
            } else {
                eprintln!("{msg}");
            }
            std::process::exit(error::exit_code(&e));
        }
        return;
    }

    // Tools
    if let Commands::Tools { ref name } = cli.command {
        commands::tools::run(name.as_deref(), is_json, start);
        return;
    }

    // Health
    if let Commands::Health = cli.command {
        let exit = commands::health::run(is_json, start).await;
        std::process::exit(exit);
    }

    // --- Build SDK client ---

    let client_id = std::env::var("CHO_CLIENT_ID").unwrap_or_default();
    let base_url =
        std::env::var("CHO_BASE_URL").unwrap_or_else(|_| SdkConfig::default().base_url.clone());

    let tenant_id = cli
        .tenant
        .or_else(load_config_tenant_id)
        .unwrap_or_default();

    let config = SdkConfig::default().with_base_url(base_url);

    let auth = AuthManager::new(client_id);
    // Try to load stored tokens
    let _ = auth.load_stored_tokens().await;

    let client = match XeroClient::builder()
        .config(config)
        .tenant_id(tenant_id)
        .auth_manager(auth)
        .rate_limit(RateLimitConfig::default())
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            let msg = error::format_error(&e, is_json, "unknown", start);
            if is_json {
                println!("{msg}");
            } else {
                eprintln!("{msg}");
            }
            std::process::exit(error::exit_code(&e));
        }
    };

    let ctx = CliContext::new(client, format, json_options, cli.limit, cli.all);

    // --- Dispatch command ---

    let (tool_name, result) = match &cli.command {
        Commands::Init(_) | Commands::Tools { .. } | Commands::Health => {
            unreachable!("handled by early dispatch")
        }
        Commands::Auth { command } => (
            commands::auth::tool_name(command),
            commands::auth::run(command, &ctx, start).await,
        ),
        Commands::Invoices { command } => (
            commands::invoices::tool_name(command),
            commands::invoices::run(command, &ctx, start).await,
        ),
        Commands::Contacts { command } => (
            commands::contacts::tool_name(command),
            commands::contacts::run(command, &ctx, start).await,
        ),
        Commands::Payments { command } => (
            commands::payments::tool_name(command),
            commands::payments::run(command, &ctx, start).await,
        ),
        Commands::Transactions { command } => (
            commands::transactions::tool_name(command),
            commands::transactions::run(command, &ctx, start).await,
        ),
        Commands::Accounts { command } => (
            commands::accounts::tool_name(command),
            commands::accounts::run(command, &ctx, start).await,
        ),
        Commands::Reports { command } => (
            commands::reports::tool_name(command),
            commands::reports::run(command, &ctx, start).await,
        ),
        Commands::Config { command } => (
            commands::config::tool_name(command),
            commands::config::run(command, &ctx, start).await,
        ),
        Commands::CreditNotes { command } => (
            commands::credit_notes::tool_name(command),
            commands::credit_notes::run(command, &ctx, start).await,
        ),
        Commands::Quotes { command } => (
            commands::quotes::tool_name(command),
            commands::quotes::run(command, &ctx, start).await,
        ),
        Commands::PurchaseOrders { command } => (
            commands::purchase_orders::tool_name(command),
            commands::purchase_orders::run(command, &ctx, start).await,
        ),
        Commands::Items { command } => (
            commands::items::tool_name(command),
            commands::items::run(command, &ctx, start).await,
        ),
        Commands::TaxRates { command } => (
            commands::tax_rates::tool_name(command),
            commands::tax_rates::run(command, &ctx, start).await,
        ),
        Commands::Currencies { command } => (
            commands::currencies::tool_name(command),
            commands::currencies::run(command, &ctx, start).await,
        ),
        Commands::TrackingCategories { command } => (
            commands::tracking_categories::tool_name(command),
            commands::tracking_categories::run(command, &ctx, start).await,
        ),
        Commands::Organisation { command } => (
            commands::organisations::tool_name(command),
            commands::organisations::run(command, &ctx, start).await,
        ),
        Commands::ManualJournals { command } => (
            commands::manual_journals::tool_name(command),
            commands::manual_journals::run(command, &ctx, start).await,
        ),
        Commands::Prepayments { command } => (
            commands::prepayments::tool_name(command),
            commands::prepayments::run(command, &ctx, start).await,
        ),
        Commands::Overpayments { command } => (
            commands::overpayments::tool_name(command),
            commands::overpayments::run(command, &ctx, start).await,
        ),
        Commands::LinkedTransactions { command } => (
            commands::linked_transactions::tool_name(command),
            commands::linked_transactions::run(command, &ctx, start).await,
        ),
        Commands::Budgets { command } => (
            commands::budgets::tool_name(command),
            commands::budgets::run(command, &ctx, start).await,
        ),
        Commands::RepeatingInvoices { command } => (
            commands::repeating_invoices::tool_name(command),
            commands::repeating_invoices::run(command, &ctx, start).await,
        ),
    };

    if let Err(e) = result {
        let msg = error::format_error(&e, ctx.is_json(), tool_name, start);
        if ctx.is_json() {
            println!("{msg}");
        } else {
            eprintln!("{msg}");
        }
        std::process::exit(error::exit_code(&e));
    }
}

/// Attempts to load the default tenant ID from the config file.
fn load_config_tenant_id() -> Option<String> {
    let config_dir = cho_sdk::auth::storage::config_dir().ok()?;
    let config_path = config_dir.join("config.toml");

    if !config_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(config_path).ok()?;
    let table: toml::Table = content.parse().ok()?;

    table
        .get("auth")?
        .as_table()?
        .get("tenant_id")?
        .as_str()
        .map(|s| s.to_string())
}
