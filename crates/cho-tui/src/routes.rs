//! Route catalog and workspace grouping for `cho-tui`.

use cho_sdk::api::specs::{RESOURCES, ResourceSpec};

/// High-level UI workspace groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Workspace {
    Reference,
    Sales,
    Banking,
    Purchases,
    Accounting,
    TaxFiling,
    Reports,
    PayrollPeople,
    System,
}

impl Workspace {
    /// Display label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Reference => "Reference",
            Self::Sales => "Sales",
            Self::Banking => "Banking",
            Self::Purchases => "Purchases",
            Self::Accounting => "Accounting",
            Self::TaxFiling => "Tax & Filing",
            Self::Reports => "Reports",
            Self::PayrollPeople => "Payroll & People",
            Self::System => "System",
        }
    }
}

/// Route data-fetch behavior.
#[derive(Debug, Clone, Copy)]
pub enum RouteKind {
    /// Generic resource route backed by a FreeAgent resource spec.
    Resource(ResourceSpec),
    /// Company details singleton.
    CompanyGet,
    /// Company tax timeline singleton.
    CompanyTaxTimeline,
    /// Company business categories singleton.
    CompanyBusinessCategories,
    /// Profit and loss report.
    ReportProfitAndLoss,
    /// Balance sheet report.
    ReportBalanceSheet,
    /// Trial balance report.
    ReportTrialBalance,
    /// Cashflow report.
    ReportCashflow,
    /// Expense mileage settings singleton.
    ExpenseMileageSettings,
    /// Self-assessment returns (user scoped).
    SelfAssessmentReturns,
    /// Payroll periods for a selected year.
    PayrollPeriods,
    /// Payroll period detail for selected year+period.
    PayrollPeriodDetail,
    /// Payroll profiles for selected year.
    PayrollProfiles,
    /// Auth status page.
    AuthStatus,
    /// Health checks page.
    Health,
    /// Config page.
    Config,
}

/// Route definition used by navigation and command palette.
#[derive(Debug, Clone)]
pub struct RouteDefinition {
    /// Stable route identifier.
    pub id: String,
    /// User-facing label.
    pub label: String,
    /// Workspace bucket.
    pub workspace: Workspace,
    /// Route fetch mode.
    pub kind: RouteKind,
}

/// Builds all routes in deterministic display order.
pub fn build_routes() -> Vec<RouteDefinition> {
    let mut routes = Vec::new();

    push(
        &mut routes,
        "company.get",
        "Company Details",
        Workspace::Reference,
        RouteKind::CompanyGet,
    );
    push(
        &mut routes,
        "company.tax-timeline",
        "Tax Timeline",
        Workspace::Reference,
        RouteKind::CompanyTaxTimeline,
    );
    push(
        &mut routes,
        "company.business-categories",
        "Business Categories",
        Workspace::Reference,
        RouteKind::CompanyBusinessCategories,
    );
    push_resource(
        &mut routes,
        "categories",
        "Categories",
        Workspace::Reference,
    );
    push(
        &mut routes,
        "expenses.mileage-settings",
        "Mileage Settings",
        Workspace::Reference,
        RouteKind::ExpenseMileageSettings,
    );

    push_resource(&mut routes, "contacts", "Contacts", Workspace::Sales);
    push_resource(&mut routes, "invoices", "Invoices", Workspace::Sales);
    push_resource(&mut routes, "estimates", "Estimates", Workspace::Sales);
    push_resource(
        &mut routes,
        "credit-notes",
        "Credit Notes",
        Workspace::Sales,
    );
    push_resource(
        &mut routes,
        "recurring-invoices",
        "Recurring Invoices",
        Workspace::Sales,
    );
    push_resource(&mut routes, "projects", "Projects", Workspace::Sales);
    push_resource(&mut routes, "timeslips", "Timeslips", Workspace::Sales);
    push_resource(&mut routes, "stock-items", "Stock Items", Workspace::Sales);

    push_resource(
        &mut routes,
        "bank-accounts",
        "Bank Accounts",
        Workspace::Banking,
    );
    push_resource(
        &mut routes,
        "bank-transactions",
        "Bank Transactions",
        Workspace::Banking,
    );
    push_resource(
        &mut routes,
        "attachments",
        "Attachments",
        Workspace::Banking,
    );

    push_resource(&mut routes, "bills", "Bills", Workspace::Purchases);
    push_resource(&mut routes, "expenses", "Expenses", Workspace::Purchases);

    push_resource(
        &mut routes,
        "transactions",
        "Transactions",
        Workspace::Accounting,
    );
    push_resource(
        &mut routes,
        "journal-sets",
        "Journal Sets",
        Workspace::Accounting,
    );
    push_resource(
        &mut routes,
        "capital-assets",
        "Capital Assets",
        Workspace::Accounting,
    );
    push_resource(
        &mut routes,
        "sales-tax-periods",
        "Sales Tax Periods",
        Workspace::Accounting,
    );

    push_resource(
        &mut routes,
        "vat-returns",
        "VAT Returns",
        Workspace::TaxFiling,
    );
    push_resource(
        &mut routes,
        "corporation-tax-returns",
        "Corporation Tax Returns",
        Workspace::TaxFiling,
    );
    push(
        &mut routes,
        "self-assessment-returns",
        "Self-Assessment Returns",
        Workspace::TaxFiling,
        RouteKind::SelfAssessmentReturns,
    );
    push_resource(
        &mut routes,
        "final-accounts-reports",
        "Final Accounts Reports",
        Workspace::TaxFiling,
    );

    push(
        &mut routes,
        "reports.profit-and-loss",
        "Profit and Loss",
        Workspace::Reports,
        RouteKind::ReportProfitAndLoss,
    );
    push(
        &mut routes,
        "reports.balance-sheet",
        "Balance Sheet",
        Workspace::Reports,
        RouteKind::ReportBalanceSheet,
    );
    push(
        &mut routes,
        "reports.trial-balance",
        "Trial Balance",
        Workspace::Reports,
        RouteKind::ReportTrialBalance,
    );
    push(
        &mut routes,
        "reports.cashflow",
        "Cashflow",
        Workspace::Reports,
        RouteKind::ReportCashflow,
    );

    push(
        &mut routes,
        "payroll.periods",
        "Payroll Periods",
        Workspace::PayrollPeople,
        RouteKind::PayrollPeriods,
    );
    push(
        &mut routes,
        "payroll.period",
        "Payroll Period Detail",
        Workspace::PayrollPeople,
        RouteKind::PayrollPeriodDetail,
    );
    push(
        &mut routes,
        "payroll-profiles.list",
        "Payroll Profiles",
        Workspace::PayrollPeople,
        RouteKind::PayrollProfiles,
    );
    push_resource(&mut routes, "users", "Users", Workspace::PayrollPeople);

    push(
        &mut routes,
        "auth.status",
        "Auth Status",
        Workspace::System,
        RouteKind::AuthStatus,
    );
    push(
        &mut routes,
        "health.check",
        "Health",
        Workspace::System,
        RouteKind::Health,
    );
    push(
        &mut routes,
        "config.show",
        "Config",
        Workspace::System,
        RouteKind::Config,
    );

    routes
}

fn push(
    routes: &mut Vec<RouteDefinition>,
    id: &str,
    label: &str,
    workspace: Workspace,
    kind: RouteKind,
) {
    routes.push(RouteDefinition {
        id: id.to_string(),
        label: label.to_string(),
        workspace,
        kind,
    });
}

fn push_resource(routes: &mut Vec<RouteDefinition>, name: &str, label: &str, workspace: Workspace) {
    if let Some(spec) = RESOURCES.iter().copied().find(|spec| spec.name == name) {
        routes.push(RouteDefinition {
            id: name.to_string(),
            label: label.to_string(),
            workspace,
            kind: RouteKind::Resource(spec),
        });
    }
}
