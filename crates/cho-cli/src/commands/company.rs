//! Company commands.

use std::time::Instant;

use cho_sdk::error::Result;
use cho_sdk::liabilities::annotate_tax_response;
use clap::Subcommand;

use crate::context::CliContext;

/// Company subcommands.
#[derive(Debug, Subcommand)]
pub enum CompanyCommands {
    /// Get company details.
    Get,
    /// Get company tax timeline.
    TaxTimeline,
    /// Get supported company business categories.
    BusinessCategories,
}

/// Tool name for company command.
pub fn tool_name(command: &CompanyCommands) -> &'static str {
    match command {
        CompanyCommands::Get => "company.get",
        CompanyCommands::TaxTimeline => "company.tax-timeline",
        CompanyCommands::BusinessCategories => "company.business-categories",
    }
}

/// Runs company command.
pub async fn run(command: &CompanyCommands, ctx: &CliContext, start: Instant) -> Result<()> {
    let (tool, path) = match command {
        CompanyCommands::Get => ("company.get", "company"),
        CompanyCommands::TaxTimeline => ("company.tax-timeline", "company/tax_timeline"),
        CompanyCommands::BusinessCategories => {
            ("company.business-categories", "company/business_categories")
        }
    };

    let mut value = ctx.client().get_json(path, &[]).await?;
    if matches!(command, CompanyCommands::TaxTimeline) {
        annotate_tax_response(&mut value);
    }
    ctx.emit_success(tool, &value, start)
}
