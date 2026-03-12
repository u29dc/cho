#![forbid(unsafe_code)]

//! cho: FreeAgent CLI for agent-native accounting automation.

mod audit;
mod commands;
mod context;
mod envelope;
mod error;
mod output;
mod registry;

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
    BankTransactionCommands, ContactCommands, CreditNoteCommands, EstimateCommands,
    ExpenseCommands, GetDeleteResourceCommands, InvoiceCommands, JournalSetCommands,
    ListOnlyResourceCommands, ReadOnlyResourceCommands, ResourceCommands, TimeslipCommands,
    UserCommands, WriteOnlyResourceCommands,
};
use crate::commands::tax::{
    CorporationTaxReturnCommands, FinalAccountsReportCommands, SelfAssessmentReturnCommands,
    VatReturnCommands,
};
use crate::commands::utils::AppConfig;
use crate::context::CliContext;
use crate::output::json::JsonOptions;
use crate::output::{OutputFormat, OutputMode};

/// `cho` CLI args.
#[derive(Debug, Parser)]
#[command(name = "cho", version, about, long_about = None)]
struct Cli {
    /// Human-readable output format.
    #[arg(long, value_enum, global = true)]
    format: Option<OutputFormat>,

    /// Emit human-readable text on stdout instead of the default JSON envelope.
    #[arg(long, global = true, conflicts_with = "format")]
    text: bool,

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
    /// Launch the terminal UI.
    Start,
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
    /// Bank feeds.
    #[command(name = "bank-feeds")]
    BankFeeds {
        #[command(subcommand)]
        command: ReadOnlyResourceCommands,
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
    /// CIS bands.
    #[command(name = "cis-bands")]
    CisBands {
        #[command(subcommand)]
        command: ListOnlyResourceCommands,
    },
    /// Email addresses.
    #[command(name = "email-addresses")]
    EmailAddresses {
        #[command(subcommand)]
        command: ListOnlyResourceCommands,
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
    /// Sales tax rates (EC MOSS).
    #[command(name = "sales-tax-rates")]
    SalesTaxRates {
        #[command(subcommand)]
        command: ListOnlyResourceCommands,
    },

    /// Credit notes.
    #[command(name = "credit-notes")]
    CreditNotes {
        #[command(subcommand)]
        command: CreditNoteCommands,
    },
    /// Credit note reconciliations.
    #[command(name = "credit-note-reconciliations")]
    CreditNoteReconciliations {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Estimates.
    Estimates {
        #[command(subcommand)]
        command: EstimateCommands,
    },
    /// Estimate items.
    #[command(name = "estimate-items")]
    EstimateItems {
        #[command(subcommand)]
        command: WriteOnlyResourceCommands,
    },
    /// Recurring invoices.
    #[command(name = "recurring-invoices")]
    RecurringInvoices {
        #[command(subcommand)]
        command: ReadOnlyResourceCommands,
    },
    /// Price list items.
    #[command(name = "price-list-items")]
    PriceListItems {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Journal sets.
    #[command(name = "journal-sets")]
    JournalSets {
        #[command(subcommand)]
        command: JournalSetCommands,
    },
    /// Users.
    Users {
        #[command(subcommand)]
        command: UserCommands,
    },
    /// Capital assets.
    #[command(name = "capital-assets")]
    CapitalAssets {
        #[command(subcommand)]
        command: ReadOnlyResourceCommands,
    },
    /// Capital asset types.
    #[command(name = "capital-asset-types")]
    CapitalAssetTypes {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Hire purchases.
    #[command(name = "hire-purchases")]
    HirePurchases {
        #[command(subcommand)]
        command: ReadOnlyResourceCommands,
    },
    /// Notes.
    Notes {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Properties.
    Properties {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Stock items.
    #[command(name = "stock-items")]
    StockItems {
        #[command(subcommand)]
        command: ReadOnlyResourceCommands,
    },
    /// Tasks.
    Tasks {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Projects.
    Projects {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// Timeslips.
    Timeslips {
        #[command(subcommand)]
        command: TimeslipCommands,
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
    let output_mode = resolve_output_mode(&cli);
    let json_mode = output_mode.is_json();

    if cli.verbose {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }

    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(err) => {
            emit_bootstrap_error(&err, json_mode, "config.load", start, 2, None);
            return;
        }
    };

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
    let _ = audit.log_command_start(&tool_name, &argv);
    let input_payload = serde_json::json!({ "tool": &tool_name });
    let _ = audit.log_command_input(&tool_name, &input_payload.to_string());

    // Early commands that do not require API client.
    match &cli.command {
        Commands::Start => match commands::start::run() {
            Ok(exit_code) => {
                let _ = audit.log_command_end(
                    &tool_name,
                    exit_code,
                    start.elapsed().as_millis() as u64,
                );
                std::process::exit(exit_code);
            }
            Err(err) => {
                emit_runtime_error(&err, json_mode, &tool_name, start, Some(&audit));
                let code = error::exit_code(&err);
                let _ = audit.log_command_end(&tool_name, code, start.elapsed().as_millis() as u64);
                std::process::exit(code);
            }
        },
        Commands::Tools { name } => {
            commands::tools::run(name.as_deref(), output_mode, start, &audit);
            let _ = audit.log_command_end(&tool_name, 0, start.elapsed().as_millis() as u64);
            return;
        }
        Commands::Health => {
            let exit_code = commands::health::run(output_mode, start, &audit).await;
            let _ =
                audit.log_command_end(&tool_name, exit_code, start.elapsed().as_millis() as u64);
            std::process::exit(exit_code);
        }
        Commands::Config { command } => {
            match commands::config::run(command, output_mode, start, &audit) {
                Ok(()) => {
                    let _ =
                        audit.log_command_end(&tool_name, 0, start.elapsed().as_millis() as u64);
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
                        audit.log_command_end(&tool_name, code, start.elapsed().as_millis() as u64);
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
            emit_runtime_error(&err, json_mode, &tool_name, start, Some(&audit));
            let code = error::exit_code(&err);
            let _ = audit.log_command_end(&tool_name, code, start.elapsed().as_millis() as u64);
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
            emit_runtime_error(&err, json_mode, &tool_name, start, Some(&audit));
            let code = error::exit_code(&err);
            let _ = audit.log_command_end(&tool_name, code, start.elapsed().as_millis() as u64);
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
            emit_runtime_error(&err, json_mode, &tool_name, start, Some(&audit));
            let code = error::exit_code(&err);
            let _ = audit.log_command_end(&tool_name, code, start.elapsed().as_millis() as u64);
            std::process::exit(code);
        }
    };

    if let Err(err) = auth.load_stored_tokens().await {
        emit_runtime_error(&err, json_mode, &tool_name, start, Some(&audit));
        let code = error::exit_code(&err);
        let _ = audit.log_command_end(&tool_name, code, start.elapsed().as_millis() as u64);
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
            emit_runtime_error(&err, json_mode, &tool_name, start, Some(&audit));
            let code = error::exit_code(&err);
            let _ = audit.log_command_end(&tool_name, code, start.elapsed().as_millis() as u64);
            std::process::exit(code);
        }
    };

    let context = CliContext::new(
        client,
        output_mode,
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
        Commands::Start | Commands::Tools { .. } | Commands::Health | Commands::Config { .. } => {
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
        Commands::BankFeeds { command } => (
            commands::resources::tool_name_read_only("bank-feeds", command),
            commands::resources::run_read_only_resource("bank-feeds", command, ctx, start).await,
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
        Commands::CisBands { command } => (
            commands::resources::tool_name_list_only("cis-bands", command),
            commands::resources::run_list_only_resource("cis-bands", command, ctx, start).await,
        ),
        Commands::EmailAddresses { command } => (
            commands::resources::tool_name_list_only("email-addresses", command),
            commands::resources::run_list_only_resource("email-addresses", command, ctx, start)
                .await,
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
        Commands::SalesTaxRates { command } => (
            commands::resources::tool_name_list_only("sales-tax-rates", command),
            commands::resources::run_list_only_resource("sales-tax-rates", command, ctx, start)
                .await,
        ),
        Commands::CreditNotes { command } => (
            commands::resources::credit_notes_tool_name(command),
            commands::resources::run_credit_notes(command, ctx, start).await,
        ),
        Commands::CreditNoteReconciliations { command } => (
            commands::resources::tool_name("credit-note-reconciliations", command),
            commands::resources::run_resource("credit-note-reconciliations", command, ctx, start)
                .await,
        ),
        Commands::Estimates { command } => (
            commands::resources::estimates_tool_name(command),
            commands::resources::run_estimates(command, ctx, start).await,
        ),
        Commands::EstimateItems { command } => (
            commands::resources::tool_name_write_only("estimate-items", command),
            commands::resources::run_write_only_resource("estimate-items", command, ctx, start)
                .await,
        ),
        Commands::RecurringInvoices { command } => (
            commands::resources::tool_name_read_only("recurring-invoices", command),
            commands::resources::run_read_only_resource("recurring-invoices", command, ctx, start)
                .await,
        ),
        Commands::PriceListItems { command } => (
            commands::resources::tool_name("price-list-items", command),
            commands::resources::run_resource("price-list-items", command, ctx, start).await,
        ),
        Commands::JournalSets { command } => (
            commands::resources::journal_sets_tool_name(command),
            commands::resources::run_journal_sets(command, ctx, start).await,
        ),
        Commands::Users { command } => (
            commands::resources::users_tool_name(command),
            commands::resources::run_users(command, ctx, start).await,
        ),
        Commands::CapitalAssets { command } => (
            commands::resources::tool_name_read_only("capital-assets", command),
            commands::resources::run_read_only_resource("capital-assets", command, ctx, start)
                .await,
        ),
        Commands::CapitalAssetTypes { command } => (
            commands::resources::tool_name("capital-asset-types", command),
            commands::resources::run_resource("capital-asset-types", command, ctx, start).await,
        ),
        Commands::HirePurchases { command } => (
            commands::resources::tool_name_read_only("hire-purchases", command),
            commands::resources::run_read_only_resource("hire-purchases", command, ctx, start)
                .await,
        ),
        Commands::Notes { command } => (
            commands::resources::tool_name("notes", command),
            commands::resources::run_resource("notes", command, ctx, start).await,
        ),
        Commands::Properties { command } => (
            commands::resources::tool_name("properties", command),
            commands::resources::run_resource("properties", command, ctx, start).await,
        ),
        Commands::StockItems { command } => (
            commands::resources::tool_name_read_only("stock-items", command),
            commands::resources::run_read_only_resource("stock-items", command, ctx, start).await,
        ),
        Commands::Tasks { command } => (
            commands::resources::tool_name("tasks", command),
            commands::resources::run_resource("tasks", command, ctx, start).await,
        ),
        Commands::Projects { command } => (
            commands::resources::tool_name("projects", command),
            commands::resources::run_resource("projects", command, ctx, start).await,
        ),
        Commands::Timeslips { command } => (
            commands::resources::timeslips_tool_name(command),
            commands::resources::run_timeslips(command, ctx, start).await,
        ),
        Commands::Attachments { command } => (
            commands::resources::tool_name_get_delete("attachments", command),
            commands::resources::run_get_delete_resource("attachments", command, ctx, start).await,
        ),
    }
}

fn resolve_output_mode(cli: &Cli) -> OutputMode {
    if let Some(format) = cli.format {
        return match format {
            OutputFormat::Table => OutputMode::Table,
            OutputFormat::Csv => OutputMode::Csv,
        };
    }

    if cli.text {
        OutputMode::Text
    } else {
        OutputMode::Json
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

fn top_level_tool_name(command: &Commands) -> String {
    match command {
        Commands::Start => "tui.start".to_string(),
        Commands::Tools { name } => {
            if name.is_some() {
                "tools.get".to_string()
            } else {
                "tools.list".to_string()
            }
        }
        Commands::Health => "health.check".to_string(),
        Commands::Config { command } => commands::config::tool_name(command).to_string(),
        Commands::Auth { command } => commands::auth::tool_name(command).to_string(),
        Commands::Company { command } => commands::company::tool_name(command).to_string(),
        Commands::Reports { command } => commands::reports::tool_name(command).to_string(),
        Commands::Contacts { command } => commands::resources::contacts_tool_name(command),
        Commands::Invoices { command } => commands::resources::invoices_tool_name(command),
        Commands::BankAccounts { command } => {
            commands::resources::tool_name("bank-accounts", command)
        }
        Commands::BankFeeds { command } => {
            commands::resources::tool_name_read_only("bank-feeds", command)
        }
        Commands::BankTransactions { command } => {
            commands::resources::bank_transactions_tool_name(command)
        }
        Commands::BankTransactionExplanations { command } => {
            commands::resources::tool_name("bank-transaction-explanations", command)
        }
        Commands::Bills { command } => commands::resources::tool_name("bills", command),
        Commands::Expenses { command } => commands::resources::expenses_tool_name(command),
        Commands::Categories { command } => commands::resources::tool_name("categories", command),
        Commands::CisBands { command } => {
            commands::resources::tool_name_list_only("cis-bands", command)
        }
        Commands::EmailAddresses { command } => {
            commands::resources::tool_name_list_only("email-addresses", command)
        }
        Commands::Transactions { command } => {
            commands::resources::tool_name_read_only("transactions", command)
        }
        Commands::CorporationTaxReturns { command } => {
            commands::tax::corporation_tool_name(command).to_string()
        }
        Commands::SelfAssessmentReturns { command } => {
            commands::tax::self_assessment_tool_name(command).to_string()
        }
        Commands::VatReturns { command } => commands::tax::vat_tool_name(command).to_string(),
        Commands::FinalAccountsReports { command } => {
            commands::tax::final_accounts_tool_name(command).to_string()
        }
        Commands::Payroll { command } => commands::payroll::payroll_tool_name(command).to_string(),
        Commands::PayrollProfiles { command } => {
            commands::payroll::payroll_profile_tool_name(command).to_string()
        }
        Commands::SalesTaxPeriods { command } => {
            commands::resources::tool_name("sales-tax-periods", command)
        }
        Commands::SalesTaxRates { command } => {
            commands::resources::tool_name_list_only("sales-tax-rates", command)
        }
        Commands::CreditNotes { command } => commands::resources::credit_notes_tool_name(command),
        Commands::CreditNoteReconciliations { command } => {
            commands::resources::tool_name("credit-note-reconciliations", command)
        }
        Commands::Estimates { command } => commands::resources::estimates_tool_name(command),
        Commands::EstimateItems { command } => {
            commands::resources::tool_name_write_only("estimate-items", command)
        }
        Commands::RecurringInvoices { command } => {
            commands::resources::tool_name_read_only("recurring-invoices", command)
        }
        Commands::PriceListItems { command } => {
            commands::resources::tool_name("price-list-items", command)
        }
        Commands::JournalSets { command } => commands::resources::journal_sets_tool_name(command),
        Commands::Users { command } => commands::resources::users_tool_name(command),
        Commands::CapitalAssets { command } => {
            commands::resources::tool_name_read_only("capital-assets", command)
        }
        Commands::CapitalAssetTypes { command } => {
            commands::resources::tool_name("capital-asset-types", command)
        }
        Commands::HirePurchases { command } => {
            commands::resources::tool_name_read_only("hire-purchases", command)
        }
        Commands::Notes { command } => commands::resources::tool_name("notes", command),
        Commands::Properties { command } => commands::resources::tool_name("properties", command),
        Commands::StockItems { command } => {
            commands::resources::tool_name_read_only("stock-items", command)
        }
        Commands::Tasks { command } => commands::resources::tool_name("tasks", command),
        Commands::Projects { command } => commands::resources::tool_name("projects", command),
        Commands::Timeslips { command } => commands::resources::timeslips_tool_name(command),
        Commands::Attachments { command } => {
            commands::resources::tool_name_get_delete("attachments", command)
        }
    }
}
