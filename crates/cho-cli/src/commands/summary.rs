//! Concise finance summary commands.

use std::collections::BTreeMap;
use std::time::Instant;

use cho_sdk::error::Result;
use cho_sdk::liabilities::{ReconcileOptions, TaxCalendarOptions};
use cho_sdk::models::TaxCalendarEntry;
use chrono::{Datelike, Utc};
use clap::Subcommand;
use serde_json::Value;

use crate::context::CliContext;

use super::resources::InvoiceListArgs;
use super::resources_sales::fetch_filtered_invoices;

/// Summary commands.
#[derive(Debug, Clone, Subcommand)]
pub enum SummaryCommands {
    /// Summarize tax and payroll obligations.
    Obligations {
        /// Optional self-assessment user to include.
        #[arg(long)]
        user: Option<String>,
        /// Optional payroll year override.
        #[arg(long)]
        payroll_year: Option<i32>,
        /// Include detailed obligation items instead of the compact upcoming slice.
        #[arg(long)]
        details: bool,
    },
    /// Summarize invoice receivables.
    Receivables {
        #[command(flatten)]
        args: Box<InvoiceListArgs>,
    },
    /// Summarize payroll obligations for a year.
    Payroll {
        /// Optional payroll year override.
        #[arg(long)]
        year: Option<i32>,
        /// Include the detailed period/event list.
        #[arg(long)]
        details: bool,
    },
}

/// Tool name for summary command.
pub fn tool_name(command: &SummaryCommands) -> &'static str {
    match command {
        SummaryCommands::Obligations { .. } => "summary.obligations",
        SummaryCommands::Receivables { .. } => "summary.receivables",
        SummaryCommands::Payroll { .. } => "summary.payroll",
    }
}

/// Runs summary command.
pub async fn run(command: &SummaryCommands, ctx: &CliContext, start: Instant) -> Result<()> {
    match command {
        SummaryCommands::Obligations {
            user,
            payroll_year,
            details,
        } => {
            let report = ctx
                .client()
                .liabilities()
                .reconcile_hmrc(&ReconcileOptions {
                    user: user.clone(),
                    payroll_year: *payroll_year,
                    ..ReconcileOptions::default()
                })
                .await?;

            let mut by_kind = BTreeMap::new();
            let mut by_event_type = BTreeMap::new();
            for item in &report.items {
                *by_kind
                    .entry(item.obligation.kind.clone())
                    .or_insert(0usize) += 1;
                *by_event_type
                    .entry(item.obligation.event_type.clone())
                    .or_insert(0usize) += 1;
            }

            let upcoming = limit_reconciliation_items(
                report.items.iter().collect::<Vec<_>>(),
                ctx.summary_limit(5),
            );

            let payload = serde_json::json!({
                "summary": report.summary,
                "by_kind": by_kind,
                "by_event_type": by_event_type,
                "upcoming": upcoming,
                "items": if *details { serde_json::to_value(&report.items).unwrap_or_else(|_| Value::Array(Vec::new())) } else { Value::Null },
            });
            ctx.emit_success("summary.obligations", &payload, start)
        }
        SummaryCommands::Receivables { args } => {
            let result = fetch_filtered_invoices(args, ctx).await?;
            let mut by_status = BTreeMap::new();
            let mut total_value = 0.0;
            let mut outstanding_value = 0.0;
            let mut overdue_count = 0usize;
            let mut open_count = 0usize;

            for item in &result.items {
                let status = item
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_ascii_lowercase();
                *by_status.entry(status.clone()).or_insert(0usize) += 1;

                if status == "overdue" {
                    overdue_count += 1;
                }
                if matches!(status.as_str(), "open" | "overdue" | "sent" | "unpaid") {
                    open_count += 1;
                }

                total_value += extract_amount(item, &["total_value", "gross_value", "value"]);
                outstanding_value += extract_amount(
                    item,
                    &["outstanding_value", "balance", "due_value", "amount_due"],
                );
            }

            let payload = serde_json::json!({
                "count": result.items.len(),
                "overdue_count": overdue_count,
                "open_count": open_count,
                "total_value": total_value,
                "outstanding_value": outstanding_value,
                "by_status": by_status,
                "items": result.items,
            });
            ctx.emit_success("summary.receivables", &payload, start)
        }
        SummaryCommands::Payroll { year, details } => {
            let payroll_year = year.unwrap_or_else(|| Utc::now().year());
            let calendar = ctx
                .client()
                .liabilities()
                .tax_calendar(&TaxCalendarOptions {
                    user: None,
                    payroll_year: Some(payroll_year),
                })
                .await?;
            let items = calendar
                .items
                .into_iter()
                .filter(|item| item.kind == "payroll")
                .collect::<Vec<TaxCalendarEntry>>();

            let mut by_status = BTreeMap::new();
            let mut filed_count = 0usize;
            let mut unfiled_count = 0usize;
            for item in &items {
                *by_status
                    .entry(item.status_trust.system_status.clone())
                    .or_insert(0usize) += 1;
                if item
                    .status_trust
                    .system_status
                    .eq_ignore_ascii_case("filed")
                {
                    filed_count += 1;
                } else {
                    unfiled_count += 1;
                }
            }

            let latest_filed_period = items
                .iter()
                .filter(|item| {
                    item.status_trust
                        .system_status
                        .eq_ignore_ascii_case("filed")
                })
                .max_by(|left, right| left.event_date.cmp(&right.event_date))
                .cloned();

            let next_payment_due = items
                .iter()
                .filter(|item| item.event_type == "payment_event")
                .filter(|item| {
                    !item
                        .status_trust
                        .system_status
                        .eq_ignore_ascii_case("filed")
                })
                .min_by(|left, right| left.event_date.cmp(&right.event_date))
                .cloned();

            let recent_history =
                limit_calendar_items(items.iter().collect::<Vec<_>>(), ctx.summary_limit(5), true);

            let payload = serde_json::json!({
                "year": payroll_year,
                "count": items.len(),
                "filed_count": filed_count,
                "unfiled_count": unfiled_count,
                "by_status": by_status,
                "latest_filed_period": latest_filed_period,
                "next_payment_due": next_payment_due,
                "recent_history": recent_history,
                "items": if *details {
                    if ctx.all_requested() {
                        serde_json::to_value(&items).unwrap_or_else(|_| Value::Array(Vec::new()))
                    } else {
                        limit_calendar_items(items.iter().collect::<Vec<_>>(), ctx.limit(), false)
                    }
                } else {
                    Value::Null
                },
            });
            ctx.emit_success("summary.payroll", &payload, start)
        }
    }
}

fn limit_reconciliation_items(
    mut items: Vec<&cho_sdk::models::ReconciliationItem>,
    limit: usize,
) -> Value {
    items.sort_by(|left, right| left.obligation.event_date.cmp(&right.obligation.event_date));
    let limited = items
        .into_iter()
        .take(limit.max(1))
        .cloned()
        .collect::<Vec<_>>();
    serde_json::to_value(limited).unwrap_or_else(|_| Value::Array(Vec::new()))
}

fn limit_calendar_items(
    mut items: Vec<&TaxCalendarEntry>,
    limit: usize,
    descending: bool,
) -> Value {
    items.sort_by(|left, right| left.event_date.cmp(&right.event_date));
    if descending {
        items.reverse();
    }

    let limited = items
        .into_iter()
        .take(limit.max(1))
        .cloned()
        .collect::<Vec<_>>();
    serde_json::to_value(limited).unwrap_or_else(|_| Value::Array(Vec::new()))
}

fn extract_amount(item: &Value, keys: &[&str]) -> f64 {
    keys.iter()
        .find_map(|key| {
            item.get(*key).and_then(|value| match value {
                Value::Number(number) => number.as_f64(),
                Value::String(raw) => raw.replace([',', '£'], "").parse::<f64>().ok(),
                _ => None,
            })
        })
        .unwrap_or(0.0)
}
