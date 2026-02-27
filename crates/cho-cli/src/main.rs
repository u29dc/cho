#![forbid(unsafe_code)]

//! cho: FreeAgent CLI for agent-native accounting automation.

mod audit;
mod commands;
mod context;
mod envelope;
mod error;
mod output;
mod registry;

use std::io::IsTerminal;
use std::sync::Arc;
use std::time::Instant;

use clap::{Parser, Subcommand};
use secrecy::SecretString;
use tracing_subscriber::EnvFilter;

use cho_sdk::auth::AuthManager;
use cho_sdk::client::{FreeAgentClient, HttpObserver};

use crate::audit::AuditLogger;
use crate::commands::auth::AuthCommands;
use crate::commands::company::CompanyCommands;
use crate::commands::config::ConfigCommands;
use crate::commands::payroll::{PayrollCommands, PayrollProfileCommands};
use crate::commands::reports::ReportCommands;
use crate::commands::resources::{
    BankTransactionCommands, ContactCommands, ExpenseCommands, GetDeleteResourceCommands,
    InvoiceCommands, ReadOnlyResourceCommands, ResourceCommands,
};
use crate::commands::tax::{
    CorporationTaxReturnCommands, FinalAccountsReportCommands, SelfAssessmentReturnCommands,
    VatReturnCommands,
};
use crate::commands::utils::AppConfig;
use crate::context::CliContext;
use crate::output::OutputFormat;
use crate::output::json::JsonOptions;

/// `cho` CLI args.
#[derive(Debug, Parser)]
#[command(name = "cho", version, about, long_about = None)]
struct Cli {
    /// Output format.
    #[arg(long, value_enum, global = true)]
    format: Option<OutputFormat>,

    /// Shorthand for `--format json`.
    #[arg(long, global = true)]
    json: bool,

    /// Convert decimal-like numbers to strings in JSON output.
    #[arg(long, global = true)]
    precise: bool,

    /// Max list items to return.
    #[arg(long, global = true)]
    limit: Option<usize>,

    /// Fetch all pages.
    #[arg(long, global = true)]
    all: bool,

    /// Override OAuth client id.
    #[arg(long, global = true)]
    client_id: Option<String>,

    /// Override OAuth client secret.
    #[arg(long, global = true)]
    client_secret: Option<String>,

    /// Enable verbose tracing logs.
    #[arg(long, global = true)]
    verbose: bool,

    /// Command to run.
    #[command(subcommand)]
    command: Commands,
}

/// Top-level commands.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Tool discovery metadata.
    Tools {
        /// Optional tool detail name.
        name: Option<String>,
    },
    /// Readiness checks.
    Health,
    /// Config operations.
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Authentication operations.
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Company information.
    Company {
        #[command(subcommand)]
        command: CompanyCommands,
    },
    /// Financial reports.
    Reports {
        #[command(subcommand)]
        command: ReportCommands,
    },

    /// Contacts.
    Contacts {
        #[command(subcommand)]
        command: ContactCommands,
    },
    /// Invoices.
    Invoices {
        #[command(subcommand)]
        command: InvoiceCommands,
    },
    /// Bank accounts.
    #[command(name = "bank-accounts")]
    BankAccounts {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Bank transactions.
    #[command(name = "bank-transactions")]
    BankTransactions {
        #[command(subcommand)]
        command: BankTransactionCommands,
    },
    /// Bank transaction explanations.
    #[command(name = "bank-transaction-explanations")]
    BankTransactionExplanations {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Bills.
    Bills {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Expenses.
    Expenses {
        #[command(subcommand)]
        command: ExpenseCommands,
    },
    /// Categories.
    Categories {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Accounting transactions.
    Transactions {
        #[command(subcommand)]
        command: ReadOnlyResourceCommands,
    },

    /// Corporation tax returns.
    #[command(name = "corporation-tax-returns")]
    CorporationTaxReturns {
        #[command(subcommand)]
        command: CorporationTaxReturnCommands,
    },
    /// Self-assessment returns.
    #[command(name = "self-assessment-returns")]
    SelfAssessmentReturns {
        #[command(subcommand)]
        command: SelfAssessmentReturnCommands,
    },
    /// VAT returns.
    #[command(name = "vat-returns")]
    VatReturns {
        #[command(subcommand)]
        command: VatReturnCommands,
    },
    /// Final accounts reports.
    #[command(name = "final-accounts-reports")]
    FinalAccountsReports {
        #[command(subcommand)]
        command: FinalAccountsReportCommands,
    },

    /// Payroll data.
    Payroll {
        #[command(subcommand)]
        command: PayrollCommands,
    },
    /// Payroll profiles.
    #[command(name = "payroll-profiles")]
    PayrollProfiles {
        #[command(subcommand)]
        command: PayrollProfileCommands,
    },

    /// Sales tax periods.
    #[command(name = "sales-tax-periods")]
    SalesTaxPeriods {
        #[command(subcommand)]
        command: ResourceCommands,
    },

    /// Credit notes.
    #[command(name = "credit-notes")]
    CreditNotes {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Estimates.
    Estimates {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Recurring invoices.
    #[command(name = "recurring-invoices")]
    RecurringInvoices {
        #[command(subcommand)]
        command: ReadOnlyResourceCommands,
    },
    /// Journal sets.
    #[command(name = "journal-sets")]
    JournalSets {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Users.
    Users {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Capital assets.
    #[command(name = "capital-assets")]
    CapitalAssets {
        #[command(subcommand)]
        command: ReadOnlyResourceCommands,
    },
    /// Stock items.
    #[command(name = "stock-items")]
    StockItems {
        #[command(subcommand)]
        command: ReadOnlyResourceCommands,
    },
    /// Projects.
    Projects {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Timeslips.
    Timeslips {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Attachments.
    Attachments {
        #[command(subcommand)]
        command: GetDeleteResourceCommands,
    },
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let cli = Cli::parse();

    if cli.verbose {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }

    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(err) => {
            emit_bootstrap_error(&err, cli.json, "config.load", start, 2, None);
            return;
        }
    };

    let format = resolve_output_format(&cli, &config);
    let json_mode = format == OutputFormat::Json;
    let limit = cli.limit.or(config.defaults.limit).unwrap_or(100);

    let run_id = uuid::Uuid::new_v4().to_string();
    let audit = match AuditLogger::new(run_id) {
        Ok(audit) => audit,
        Err(err) => {
            let wrapped = cho_sdk::error::ChoSdkError::Config {
                message: format!("AUDIT_LOG_UNAVAILABLE: {err}"),
            };
            emit_bootstrap_error(&wrapped, json_mode, "bootstrap.audit", start, 2, None);
            return;
        }
    };

    let tool_name = top_level_tool_name(&cli.command);
    let argv = std::env::args().collect::<Vec<_>>();
    let _ = audit.log_command_start(tool_name, &argv);
    let input_payload = serde_json::json!({ "tool": tool_name });
    let _ = audit.log_command_input(tool_name, &input_payload.to_string());

    // Early commands that do not require API client.
    match &cli.command {
        Commands::Tools { name } => {
            commands::tools::run(name.as_deref(), json_mode, start, &audit);
            let _ = audit.log_command_end(tool_name, 0, start.elapsed().as_millis() as u64);
            return;
        }
        Commands::Health => {
            let exit_code = commands::health::run(json_mode, start, &audit).await;
            let _ = audit.log_command_end(tool_name, exit_code, start.elapsed().as_millis() as u64);
            std::process::exit(exit_code);
        }
        Commands::Config { command } => {
            match commands::config::run(command, json_mode, start, &audit) {
                Ok(()) => {
                    let _ = audit.log_command_end(tool_name, 0, start.elapsed().as_millis() as u64);
                    return;
                }
                Err(err) => {
                    emit_runtime_error(
                        &err,
                        json_mode,
                        commands::config::tool_name(command),
                        start,
                        Some(&audit),
                    );
                    let code = error::exit_code(&err);
                    let _ =
                        audit.log_command_end(tool_name, code, start.elapsed().as_millis() as u64);
                    std::process::exit(code);
                }
            }
        }
        _ => {}
    }

    let client_id = match cli.client_id.or_else(|| config.resolve_client_id()) {
        Some(value) => value,
        None => {
            let err = cho_sdk::error::ChoSdkError::AuthRequired {
                message: "Missing client_id (set CHO_CLIENT_ID or auth.client_id)".to_string(),
            };
            emit_runtime_error(&err, json_mode, tool_name, start, Some(&audit));
            let code = error::exit_code(&err);
            let _ = audit.log_command_end(tool_name, code, start.elapsed().as_millis() as u64);
            std::process::exit(code);
        }
    };

    let client_secret = match cli.client_secret.or_else(|| config.resolve_client_secret()) {
        Some(value) => value,
        None => {
            let err = cho_sdk::error::ChoSdkError::AuthRequired {
                message: "Missing client_secret (set CHO_CLIENT_SECRET or auth.client_secret)"
                    .to_string(),
            };
            emit_runtime_error(&err, json_mode, tool_name, start, Some(&audit));
            let code = error::exit_code(&err);
            let _ = audit.log_command_end(tool_name, code, start.elapsed().as_millis() as u64);
            std::process::exit(code);
        }
    };

    let sdk_config = config.sdk_config();
    let allow_writes = sdk_config.allow_writes;

    let auth = match AuthManager::new(
        client_id,
        SecretString::new(client_secret.into()),
        sdk_config.clone(),
    ) {
        Ok(auth) => auth,
        Err(err) => {
            emit_runtime_error(&err, json_mode, tool_name, start, Some(&audit));
            let code = error::exit_code(&err);
            let _ = audit.log_command_end(tool_name, code, start.elapsed().as_millis() as u64);
            std::process::exit(code);
        }
    };

    if let Err(err) = auth.load_stored_tokens().await {
        emit_runtime_error(&err, json_mode, tool_name, start, Some(&audit));
        let code = error::exit_code(&err);
        let _ = audit.log_command_end(tool_name, code, start.elapsed().as_millis() as u64);
        std::process::exit(code);
    }

    let observer: Arc<dyn HttpObserver> = Arc::new(audit.clone());
    let client = match FreeAgentClient::builder()
        .config(sdk_config)
        .auth_manager(auth)
        .observer(observer)
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            emit_runtime_error(&err, json_mode, tool_name, start, Some(&audit));
            let code = error::exit_code(&err);
            let _ = audit.log_command_end(tool_name, code, start.elapsed().as_millis() as u64);
            std::process::exit(code);
        }
    };

    let context = CliContext::new(
        client,
        format,
        JsonOptions {
            precise: cli.precise,
        },
        limit,
        cli.all,
        allow_writes,
        audit.clone(),
    );

    let (tool, result) = dispatch_command(&cli.command, &context, start).await;

    match result {
        Ok(()) => {
            let _ = audit.log_command_end(&tool, 0, start.elapsed().as_millis() as u64);
        }
        Err(err) => {
            emit_runtime_error(&err, json_mode, &tool, start, Some(&audit));
            let code = error::exit_code(&err);
            let _ = audit.log_command_end(&tool, code, start.elapsed().as_millis() as u64);
            std::process::exit(code);
        }
    }
}

async fn dispatch_command(
    command: &Commands,
    ctx: &CliContext,
    start: Instant,
) -> (String, cho_sdk::error::Result<()>) {
    match command {
        Commands::Tools { .. } | Commands::Health | Commands::Config { .. } => {
            unreachable!("Early-dispatch command reached runtime dispatch")
        }
        Commands::Auth { command } => (
            commands::auth::tool_name(command).to_string(),
            commands::auth::run(command, ctx, start).await,
        ),
        Commands::Company { command } => (
            commands::company::tool_name(command).to_string(),
            commands::company::run(command, ctx, start).await,
        ),
        Commands::Reports { command } => (
            commands::reports::tool_name(command).to_string(),
            commands::reports::run(command, ctx, start).await,
        ),
        Commands::Contacts { command } => (
            commands::resources::contacts_tool_name(command),
            commands::resources::run_contacts(command, ctx, start).await,
        ),
        Commands::Invoices { command } => (
            commands::resources::invoices_tool_name(command),
            commands::resources::run_invoices(command, ctx, start).await,
        ),
        Commands::BankAccounts { command } => (
            commands::resources::tool_name("bank-accounts", command),
            commands::resources::run_resource("bank-accounts", command, ctx, start).await,
        ),
        Commands::BankTransactions { command } => (
            commands::resources::bank_transactions_tool_name(command),
            commands::resources::run_bank_transactions(command, ctx, start).await,
        ),
        Commands::BankTransactionExplanations { command } => (
            commands::resources::tool_name("bank-transaction-explanations", command),
            commands::resources::run_resource("bank-transaction-explanations", command, ctx, start)
                .await,
        ),
        Commands::Bills { command } => (
            commands::resources::tool_name("bills", command),
            commands::resources::run_resource("bills", command, ctx, start).await,
        ),
        Commands::Expenses { command } => (
            commands::resources::expenses_tool_name(command),
            commands::resources::run_expenses(command, ctx, start).await,
        ),
        Commands::Categories { command } => (
            commands::resources::tool_name("categories", command),
            commands::resources::run_resource("categories", command, ctx, start).await,
        ),
        Commands::Transactions { command } => (
            commands::resources::tool_name_read_only("transactions", command),
            commands::resources::run_read_only_resource("transactions", command, ctx, start).await,
        ),
        Commands::CorporationTaxReturns { command } => (
            commands::tax::corporation_tool_name(command).to_string(),
            commands::tax::run_corporation_tax(command, ctx, start).await,
        ),
        Commands::SelfAssessmentReturns { command } => (
            commands::tax::self_assessment_tool_name(command).to_string(),
            commands::tax::run_self_assessment(command, ctx, start).await,
        ),
        Commands::VatReturns { command } => (
            commands::tax::vat_tool_name(command).to_string(),
            commands::tax::run_vat(command, ctx, start).await,
        ),
        Commands::FinalAccountsReports { command } => (
            commands::tax::final_accounts_tool_name(command).to_string(),
            commands::tax::run_final_accounts(command, ctx, start).await,
        ),
        Commands::Payroll { command } => (
            commands::payroll::payroll_tool_name(command).to_string(),
            commands::payroll::run_payroll(command, ctx, start).await,
        ),
        Commands::PayrollProfiles { command } => (
            commands::payroll::payroll_profile_tool_name(command).to_string(),
            commands::payroll::run_payroll_profiles(command, ctx, start).await,
        ),
        Commands::SalesTaxPeriods { command } => (
            commands::resources::tool_name("sales-tax-periods", command),
            commands::resources::run_resource("sales-tax-periods", command, ctx, start).await,
        ),
        Commands::CreditNotes { command } => (
            commands::resources::tool_name("credit-notes", command),
            commands::resources::run_resource("credit-notes", command, ctx, start).await,
        ),
        Commands::Estimates { command } => (
            commands::resources::tool_name("estimates", command),
            commands::resources::run_resource("estimates", command, ctx, start).await,
        ),
        Commands::RecurringInvoices { command } => (
            commands::resources::tool_name_read_only("recurring-invoices", command),
            commands::resources::run_read_only_resource("recurring-invoices", command, ctx, start)
                .await,
        ),
        Commands::JournalSets { command } => (
            commands::resources::tool_name("journal-sets", command),
            commands::resources::run_resource("journal-sets", command, ctx, start).await,
        ),
        Commands::Users { command } => (
            commands::resources::tool_name("users", command),
            commands::resources::run_resource("users", command, ctx, start).await,
        ),
        Commands::CapitalAssets { command } => (
            commands::resources::tool_name_read_only("capital-assets", command),
            commands::resources::run_read_only_resource("capital-assets", command, ctx, start)
                .await,
        ),
        Commands::StockItems { command } => (
            commands::resources::tool_name_read_only("stock-items", command),
            commands::resources::run_read_only_resource("stock-items", command, ctx, start).await,
        ),
        Commands::Projects { command } => (
            commands::resources::tool_name("projects", command),
            commands::resources::run_resource("projects", command, ctx, start).await,
        ),
        Commands::Timeslips { command } => (
            commands::resources::tool_name("timeslips", command),
            commands::resources::run_resource("timeslips", command, ctx, start).await,
        ),
        Commands::Attachments { command } => (
            commands::resources::tool_name_get_delete("attachments", command),
            commands::resources::run_get_delete_resource("attachments", command, ctx, start).await,
        ),
    }
}

fn resolve_output_format(cli: &Cli, config: &AppConfig) -> OutputFormat {
    if cli.json {
        return OutputFormat::Json;
    }

    if let Some(format) = cli.format {
        return format;
    }

    if let Some(default) = config.defaults.format.as_deref() {
        match default {
            "json" => return OutputFormat::Json,
            "table" => return OutputFormat::Table,
            "csv" => return OutputFormat::Csv,
            _ => {}
        }
    }

    if std::io::stdout().is_terminal() {
        OutputFormat::Table
    } else {
        OutputFormat::Json
    }
}

fn emit_runtime_error(
    err: &cho_sdk::error::ChoSdkError,
    json_mode: bool,
    tool: &str,
    start: Instant,
    audit: Option<&AuditLogger>,
) {
    let output = error::format_error(err, json_mode, tool, start);
    if json_mode {
        println!("{output}");
    } else {
        eprintln!("{output}");
    }
    if let Some(audit) = audit {
        let _ = audit.log_command_output(tool, &output);
    }
}

fn emit_bootstrap_error(
    err: &cho_sdk::error::ChoSdkError,
    json_mode: bool,
    tool: &str,
    start: Instant,
    exit_code: i32,
    audit: Option<&AuditLogger>,
) {
    let output = error::format_error(err, json_mode, tool, start);
    if json_mode {
        println!("{output}");
    } else {
        eprintln!("{output}");
    }
    if let Some(audit) = audit {
        let _ = audit.log_command_output(tool, &output);
    }
    std::process::exit(exit_code);
}

fn top_level_tool_name(command: &Commands) -> &'static str {
    match command {
        Commands::Tools { name } => {
            if name.is_some() {
                "tools.get"
            } else {
                "tools.list"
            }
        }
        Commands::Health => "health.check",
        Commands::Config { command } => commands::config::tool_name(command),
        Commands::Auth { command } => commands::auth::tool_name(command),
        Commands::Company { command } => commands::company::tool_name(command),
        Commands::Reports { command } => commands::reports::tool_name(command),
        Commands::Contacts { command } => match command {
            ContactCommands::List(_) => "contacts.list",
            ContactCommands::Get { .. } => "contacts.get",
            ContactCommands::Create { .. } => "contacts.create",
            ContactCommands::Update { .. } => "contacts.update",
            ContactCommands::Delete { .. } => "contacts.delete",
            ContactCommands::Search { .. } => "contacts.search",
        },
        Commands::Invoices { command } => match command {
            InvoiceCommands::List(_) => "invoices.list",
            InvoiceCommands::Get { .. } => "invoices.get",
            InvoiceCommands::Create { .. } => "invoices.create",
            InvoiceCommands::Update { .. } => "invoices.update",
            InvoiceCommands::Delete { .. } => "invoices.delete",
            InvoiceCommands::Transition { .. } => "invoices.transition",
            InvoiceCommands::SendEmail { .. } => "invoices.send-email",
        },
        Commands::BankAccounts { command } => match command {
            ResourceCommands::List(_) => "bank-accounts.list",
            ResourceCommands::Get { .. } => "bank-accounts.get",
            ResourceCommands::Create { .. } => "bank-accounts.create",
            ResourceCommands::Update { .. } => "bank-accounts.update",
            ResourceCommands::Delete { .. } => "bank-accounts.delete",
        },
        Commands::BankTransactions { command } => match command {
            BankTransactionCommands::List(_) => "bank-transactions.list",
            BankTransactionCommands::Get { .. } => "bank-transactions.get",
            BankTransactionCommands::UploadStatement { .. } => "bank-transactions.upload-statement",
        },
        Commands::BankTransactionExplanations { command } => match command {
            ResourceCommands::List(_) => "bank-transaction-explanations.list",
            ResourceCommands::Get { .. } => "bank-transaction-explanations.get",
            ResourceCommands::Create { .. } => "bank-transaction-explanations.create",
            ResourceCommands::Update { .. } => "bank-transaction-explanations.update",
            ResourceCommands::Delete { .. } => "bank-transaction-explanations.delete",
        },
        Commands::Bills { command } => match command {
            ResourceCommands::List(_) => "bills.list",
            ResourceCommands::Get { .. } => "bills.get",
            ResourceCommands::Create { .. } => "bills.create",
            ResourceCommands::Update { .. } => "bills.update",
            ResourceCommands::Delete { .. } => "bills.delete",
        },
        Commands::Expenses { command } => match command {
            ExpenseCommands::List(_) => "expenses.list",
            ExpenseCommands::Get { .. } => "expenses.get",
            ExpenseCommands::Create { .. } => "expenses.create",
            ExpenseCommands::Update { .. } => "expenses.update",
            ExpenseCommands::Delete { .. } => "expenses.delete",
            ExpenseCommands::MileageSettings => "expenses.mileage-settings",
        },
        Commands::Categories { command } => match command {
            ResourceCommands::List(_) => "categories.list",
            ResourceCommands::Get { .. } => "categories.get",
            ResourceCommands::Create { .. } => "categories.create",
            ResourceCommands::Update { .. } => "categories.update",
            ResourceCommands::Delete { .. } => "categories.delete",
        },
        Commands::Transactions { command } => match command {
            ReadOnlyResourceCommands::List(_) => "transactions.list",
            ReadOnlyResourceCommands::Get { .. } => "transactions.get",
        },
        Commands::CorporationTaxReturns { command } => {
            commands::tax::corporation_tool_name(command)
        }
        Commands::SelfAssessmentReturns { command } => {
            commands::tax::self_assessment_tool_name(command)
        }
        Commands::VatReturns { command } => commands::tax::vat_tool_name(command),
        Commands::FinalAccountsReports { command } => {
            commands::tax::final_accounts_tool_name(command)
        }
        Commands::Payroll { command } => commands::payroll::payroll_tool_name(command),
        Commands::PayrollProfiles { command } => {
            commands::payroll::payroll_profile_tool_name(command)
        }
        Commands::SalesTaxPeriods { command } => match command {
            ResourceCommands::List(_) => "sales-tax-periods.list",
            ResourceCommands::Get { .. } => "sales-tax-periods.get",
            ResourceCommands::Create { .. } => "sales-tax-periods.create",
            ResourceCommands::Update { .. } => "sales-tax-periods.update",
            ResourceCommands::Delete { .. } => "sales-tax-periods.delete",
        },
        Commands::CreditNotes { command } => match command {
            ResourceCommands::List(_) => "credit-notes.list",
            ResourceCommands::Get { .. } => "credit-notes.get",
            ResourceCommands::Create { .. } => "credit-notes.create",
            ResourceCommands::Update { .. } => "credit-notes.update",
            ResourceCommands::Delete { .. } => "credit-notes.delete",
        },
        Commands::Estimates { command } => match command {
            ResourceCommands::List(_) => "estimates.list",
            ResourceCommands::Get { .. } => "estimates.get",
            ResourceCommands::Create { .. } => "estimates.create",
            ResourceCommands::Update { .. } => "estimates.update",
            ResourceCommands::Delete { .. } => "estimates.delete",
        },
        Commands::RecurringInvoices { command } => match command {
            ReadOnlyResourceCommands::List(_) => "recurring-invoices.list",
            ReadOnlyResourceCommands::Get { .. } => "recurring-invoices.get",
        },
        Commands::JournalSets { command } => match command {
            ResourceCommands::List(_) => "journal-sets.list",
            ResourceCommands::Get { .. } => "journal-sets.get",
            ResourceCommands::Create { .. } => "journal-sets.create",
            ResourceCommands::Update { .. } => "journal-sets.update",
            ResourceCommands::Delete { .. } => "journal-sets.delete",
        },
        Commands::Users { command } => match command {
            ResourceCommands::List(_) => "users.list",
            ResourceCommands::Get { .. } => "users.get",
            ResourceCommands::Create { .. } => "users.create",
            ResourceCommands::Update { .. } => "users.update",
            ResourceCommands::Delete { .. } => "users.delete",
        },
        Commands::CapitalAssets { command } => match command {
            ReadOnlyResourceCommands::List(_) => "capital-assets.list",
            ReadOnlyResourceCommands::Get { .. } => "capital-assets.get",
        },
        Commands::StockItems { command } => match command {
            ReadOnlyResourceCommands::List(_) => "stock-items.list",
            ReadOnlyResourceCommands::Get { .. } => "stock-items.get",
        },
        Commands::Projects { command } => match command {
            ResourceCommands::List(_) => "projects.list",
            ResourceCommands::Get { .. } => "projects.get",
            ResourceCommands::Create { .. } => "projects.create",
            ResourceCommands::Update { .. } => "projects.update",
            ResourceCommands::Delete { .. } => "projects.delete",
        },
        Commands::Timeslips { command } => match command {
            ResourceCommands::List(_) => "timeslips.list",
            ResourceCommands::Get { .. } => "timeslips.get",
            ResourceCommands::Create { .. } => "timeslips.create",
            ResourceCommands::Update { .. } => "timeslips.update",
            ResourceCommands::Delete { .. } => "timeslips.delete",
        },
        Commands::Attachments { command } => match command {
            GetDeleteResourceCommands::Get { .. } => "attachments.get",
            GetDeleteResourceCommands::Delete { .. } => "attachments.delete",
        },
    }
}
