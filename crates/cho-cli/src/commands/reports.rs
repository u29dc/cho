//! Report commands.

use std::time::Instant;

use cho_sdk::error::Result;
use clap::Subcommand;

use crate::context::CliContext;

/// Report subcommands.
#[derive(Debug, Subcommand)]
pub enum ReportCommands {
    /// Profit and loss summary.
    ProfitAndLoss {
        /// Start date (YYYY-MM-DD).
        #[arg(long)]
        from_date: Option<String>,
        /// End date (YYYY-MM-DD).
        #[arg(long)]
        to_date: Option<String>,
    },
    /// Balance sheet report.
    BalanceSheet {
        /// Report date (YYYY-MM-DD).
        #[arg(long)]
        as_at_date: Option<String>,
    },
    /// Balance sheet opening balances.
    BalanceSheetOpeningBalances,
    /// Trial balance summary.
    TrialBalance {
        /// Start date (YYYY-MM-DD).
        #[arg(long)]
        from_date: Option<String>,
        /// End date (YYYY-MM-DD).
        #[arg(long)]
        to_date: Option<String>,
    },
    /// Trial balance opening balances.
    TrialBalanceOpeningBalances,
    /// Cashflow report.
    Cashflow {
        /// Start date (YYYY-MM-DD).
        #[arg(long)]
        from_date: Option<String>,
        /// End date (YYYY-MM-DD).
        #[arg(long)]
        to_date: Option<String>,
        /// Number of months to project when no date range is provided.
        #[arg(long)]
        months: Option<u32>,
    },
}

/// Tool name for report command.
pub fn tool_name(command: &ReportCommands) -> &'static str {
    match command {
        ReportCommands::ProfitAndLoss { .. } => "reports.profit-and-loss",
        ReportCommands::BalanceSheet { .. } => "reports.balance-sheet",
        ReportCommands::BalanceSheetOpeningBalances => "reports.balance-sheet-opening-balances",
        ReportCommands::TrialBalance { .. } => "reports.trial-balance",
        ReportCommands::TrialBalanceOpeningBalances => "reports.trial-balance-opening-balances",
        ReportCommands::Cashflow { .. } => "reports.cashflow",
    }
}

/// Runs report command.
pub async fn run(command: &ReportCommands, ctx: &CliContext, start: Instant) -> Result<()> {
    match command {
        ReportCommands::ProfitAndLoss { from_date, to_date } => {
            let mut query = Vec::new();
            maybe_push(&mut query, "from_date", from_date);
            maybe_push(&mut query, "to_date", to_date);
            let value = ctx
                .client()
                .get_json("accounting/profit_and_loss/summary", &query)
                .await?;
            ctx.emit_success("reports.profit-and-loss", &value, start)
        }
        ReportCommands::BalanceSheet { as_at_date } => {
            let mut query = Vec::new();
            maybe_push(&mut query, "as_at_date", as_at_date);
            let value = ctx
                .client()
                .get_json("accounting/balance_sheet", &query)
                .await?;
            ctx.emit_success("reports.balance-sheet", &value, start)
        }
        ReportCommands::BalanceSheetOpeningBalances => {
            let value = ctx
                .client()
                .get_json("accounting/balance_sheet/opening_balances", &[])
                .await?;
            ctx.emit_success("reports.balance-sheet-opening-balances", &value, start)
        }
        ReportCommands::TrialBalance { from_date, to_date } => {
            let mut query = Vec::new();
            maybe_push(&mut query, "from_date", from_date);
            maybe_push(&mut query, "to_date", to_date);
            let value = ctx
                .client()
                .get_json("accounting/trial_balance/summary", &query)
                .await?;
            ctx.emit_success("reports.trial-balance", &value, start)
        }
        ReportCommands::TrialBalanceOpeningBalances => {
            let value = ctx
                .client()
                .get_json("accounting/trial_balance/summary/opening_balances", &[])
                .await?;
            ctx.emit_success("reports.trial-balance-opening-balances", &value, start)
        }
        ReportCommands::Cashflow {
            from_date,
            to_date,
            months,
        } => {
            let mut query = Vec::new();

            if let Some(months) = months {
                query.push(("months".to_string(), months.to_string()));
            } else {
                maybe_push(&mut query, "from_date", from_date);
                maybe_push(&mut query, "to_date", to_date);
            }

            if query.is_empty() {
                query.push(("months".to_string(), "12".to_string()));
            }
            let value = ctx.client().get_json("cashflow", &query).await?;
            ctx.emit_success("reports.cashflow", &value, start)
        }
    }
}

fn maybe_push(query: &mut Vec<(String, String)>, key: &str, value: &Option<String>) {
    if let Some(value) = value
        && !value.trim().is_empty()
    {
        query.push((key.to_string(), value.to_string()));
    }
}
