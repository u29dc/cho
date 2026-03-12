//! Tool registry used by `cho tools`.

use cho_sdk::api::specs::RESOURCES;
use serde::Serialize;

/// Global flag metadata.
#[derive(Debug, Clone, Serialize)]
pub struct GlobalFlagMeta {
    /// Flag name.
    pub name: &'static str,
    /// Description.
    pub description: &'static str,
    /// Default value.
    pub default: &'static str,
}

/// Parameter metadata.
#[derive(Debug, Clone, Serialize)]
pub struct ParameterMeta {
    /// Parameter name.
    pub name: &'static str,
    /// Parameter type.
    #[serde(rename = "type")]
    pub param_type: &'static str,
    /// Required flag.
    pub required: bool,
    /// Description.
    pub description: &'static str,
}

/// Tool metadata entry.
#[derive(Debug, Clone, Serialize)]
pub struct ToolMeta {
    /// Dotted tool name.
    pub name: String,
    /// Full command.
    pub command: String,
    /// Category.
    pub category: String,
    /// Description.
    pub description: String,
    /// Parameter metadata.
    pub parameters: Vec<ParameterMeta>,
    /// Key output fields.
    #[serde(rename = "outputFields")]
    pub output_fields: Vec<String>,
    /// Optional output schema hint.
    #[serde(rename = "outputSchema")]
    pub output_schema: Option<serde_json::Value>,
    /// Optional input schema hint.
    #[serde(rename = "inputSchema")]
    pub input_schema: Option<serde_json::Value>,
    /// Idempotent operation marker.
    pub idempotent: bool,
    /// Rate-limit group marker.
    #[serde(rename = "rateLimit")]
    pub rate_limit: Option<String>,
    /// Example command line.
    pub example: String,
}

/// Registry global flags.
pub const GLOBAL_FLAGS: &[GlobalFlagMeta] = &[
    GlobalFlagMeta {
        name: "--text",
        description: "Emit human-readable text on stdout instead of the default JSON envelope",
        default: "false",
    },
    GlobalFlagMeta {
        name: "--format",
        description: "Emit human-readable output as table|csv on stdout",
        default: "json",
    },
    GlobalFlagMeta {
        name: "--limit",
        description: "Maximum items to return for list commands",
        default: "100",
    },
    GlobalFlagMeta {
        name: "--all",
        description: "Fetch all available pages",
        default: "false",
    },
    GlobalFlagMeta {
        name: "--verbose",
        description: "Enable tracing logs to stderr",
        default: "false",
    },
    GlobalFlagMeta {
        name: "--precise",
        description: "Render decimal-like JSON numbers as strings",
        default: "false",
    },
];

/// Builds complete tool metadata catalog.
pub fn tool_catalog() -> Vec<ToolMeta> {
    let mut tools = vec![
        static_tool(
            "tui.start",
            "cho start",
            "tui",
            "Launch cho terminal UI",
            true,
        ),
        static_tool(
            "tools.list",
            "cho tools",
            "tools",
            "List all available tools and metadata",
            true,
        ),
        static_tool(
            "tools.get",
            "cho tools <name>",
            "tools",
            "Get metadata for one tool",
            true,
        ),
        static_tool(
            "health.check",
            "cho health",
            "health",
            "Check CLI readiness and remediation hints",
            true,
        ),
        static_tool(
            "config.show",
            "cho config show",
            "config",
            "Show current configuration",
            true,
        ),
        static_tool(
            "config.set",
            "cho config set <key> <value>",
            "config",
            "Set configuration key/value",
            true,
        ),
        static_tool(
            "auth.login",
            "cho auth login",
            "auth",
            "Run OAuth login flow",
            true,
        ),
        static_tool(
            "auth.status",
            "cho auth status",
            "auth",
            "Show authentication status",
            true,
        ),
        static_tool(
            "auth.refresh",
            "cho auth refresh",
            "auth",
            "Refresh access token",
            false,
        ),
        static_tool(
            "auth.logout",
            "cho auth logout",
            "auth",
            "Clear stored authentication tokens",
            false,
        ),
        static_tool(
            "company.get",
            "cho company get",
            "company",
            "Get company details",
            true,
        ),
        static_tool(
            "company.tax-timeline",
            "cho company tax-timeline",
            "company",
            "Get company tax timeline",
            true,
        ),
        static_tool(
            "company.business-categories",
            "cho company business-categories",
            "company",
            "Get supported business categories",
            true,
        ),
        static_tool(
            "reports.profit-and-loss",
            "cho reports profit-and-loss",
            "reports",
            "Get profit and loss summary",
            true,
        ),
        static_tool(
            "reports.balance-sheet",
            "cho reports balance-sheet",
            "reports",
            "Get balance sheet report",
            true,
        ),
        static_tool(
            "reports.trial-balance",
            "cho reports trial-balance",
            "reports",
            "Get trial balance summary",
            true,
        ),
        static_tool(
            "reports.cashflow",
            "cho reports cashflow",
            "reports",
            "Get cashflow report",
            true,
        ),
        static_tool(
            "self-assessment-returns.list",
            "cho self-assessment-returns list --user <id>",
            "self-assessment-returns",
            "List self-assessment returns for a user",
            true,
        ),
        static_tool(
            "self-assessment-returns.get",
            "cho self-assessment-returns get --user <id> <period_ends_on>",
            "self-assessment-returns",
            "Get self-assessment return for a period",
            true,
        ),
        static_tool(
            "self-assessment-returns.mark-filed",
            "cho self-assessment-returns mark-filed --user <id> <period_ends_on>",
            "self-assessment-returns",
            "Mark self-assessment return as filed",
            false,
        ),
        static_tool(
            "self-assessment-returns.mark-unfiled",
            "cho self-assessment-returns mark-unfiled --user <id> <period_ends_on>",
            "self-assessment-returns",
            "Mark self-assessment return as unfiled",
            false,
        ),
        static_tool(
            "self-assessment-returns.mark-payment-paid",
            "cho self-assessment-returns mark-payment-paid --user <id> <period_ends_on> <payment_date>",
            "self-assessment-returns",
            "Mark self-assessment payment as paid",
            false,
        ),
        static_tool(
            "self-assessment-returns.mark-payment-unpaid",
            "cho self-assessment-returns mark-payment-unpaid --user <id> <period_ends_on> <payment_date>",
            "self-assessment-returns",
            "Mark self-assessment payment as unpaid",
            false,
        ),
        static_tool(
            "payroll.periods",
            "cho payroll periods <year>",
            "payroll",
            "List payroll periods for tax year",
            true,
        ),
        static_tool(
            "payroll.period",
            "cho payroll period <year> <period>",
            "payroll",
            "Get payroll period and payslip details",
            true,
        ),
        static_tool(
            "payroll.mark-payment-paid",
            "cho payroll mark-payment-paid <year> <payment_date>",
            "payroll",
            "Mark payroll payment as paid",
            false,
        ),
        static_tool(
            "payroll.mark-payment-unpaid",
            "cho payroll mark-payment-unpaid <year> <payment_date>",
            "payroll",
            "Mark payroll payment as unpaid",
            false,
        ),
        static_tool(
            "payroll-profiles.list",
            "cho payroll-profiles list <year> [--user <url>]",
            "payroll-profiles",
            "List payroll profiles",
            true,
        ),
    ];

    for spec in RESOURCES {
        let category = spec.name.to_string();

        if spec.capabilities.list {
            tools.push(static_tool_owned(
                format!("{}.list", spec.name),
                format!("cho {} list", spec.name),
                category.clone(),
                format!("List {}", spec.name),
                true,
            ));
        }

        if spec.capabilities.get {
            tools.push(static_tool_owned(
                format!("{}.get", spec.name),
                format!("cho {} get <id>", spec.name),
                category.clone(),
                format!("Get one {} item", spec.name),
                true,
            ));
        }

        if spec.capabilities.create {
            tools.push(static_tool_owned(
                format!("{}.create", spec.name),
                format!("cho {} create --file <path>", spec.name),
                category.clone(),
                format!("Create {} item", spec.name),
                false,
            ));
        }

        if spec.capabilities.update {
            tools.push(static_tool_owned(
                format!("{}.update", spec.name),
                format!("cho {} update <id> --file <path>", spec.name),
                category.clone(),
                format!("Update {} item", spec.name),
                false,
            ));
        }

        if spec.capabilities.delete {
            tools.push(static_tool_owned(
                format!("{}.delete", spec.name),
                format!("cho {} delete <id>", spec.name),
                category.clone(),
                format!("Delete {} item", spec.name),
                false,
            ));
        }
    }

    tools.push(static_tool(
        "contacts.search",
        "cho contacts search <term>",
        "contacts",
        "Search contacts by name or email",
        true,
    ));
    tools.push(static_tool(
        "invoices.transition",
        "cho invoices transition <id> <action>",
        "invoices",
        "Trigger invoice status transition",
        false,
    ));
    tools.push(static_tool(
        "invoices.send-email",
        "cho invoices send-email <id> [--file <path>]",
        "invoices",
        "Send invoice email",
        false,
    ));
    tools.push(static_tool(
        "invoices.duplicate",
        "cho invoices duplicate <id>",
        "invoices",
        "Duplicate invoice",
        false,
    ));
    tools.push(static_tool(
        "invoices.direct-debit",
        "cho invoices direct-debit <id> [--file <path>]",
        "invoices",
        "Take invoice payment via direct debit",
        false,
    ));
    tools.push(static_tool(
        "invoices.timeline",
        "cho invoices timeline",
        "invoices",
        "Get invoice timeline",
        true,
    ));
    tools.push(static_tool(
        "invoices.get-pdf",
        "cho invoices get-pdf <id> [--output <path>]",
        "invoices",
        "Get invoice PDF payload or write decoded PDF bytes",
        true,
    ));
    tools.push(static_tool(
        "invoices.default-additional-text.get",
        "cho invoices default-additional-text get",
        "invoices",
        "Get invoice default additional text",
        true,
    ));
    tools.push(static_tool(
        "invoices.default-additional-text.update",
        "cho invoices default-additional-text update --file <path>",
        "invoices",
        "Update invoice default additional text",
        false,
    ));
    tools.push(static_tool(
        "invoices.default-additional-text.delete",
        "cho invoices default-additional-text delete",
        "invoices",
        "Delete invoice default additional text",
        false,
    ));
    tools.push(static_tool(
        "credit-notes.transition",
        "cho credit-notes transition <id> <action>",
        "credit-notes",
        "Trigger credit note status transition",
        false,
    ));
    tools.push(static_tool(
        "credit-notes.send-email",
        "cho credit-notes send-email <id> [--file <path>]",
        "credit-notes",
        "Send credit note email",
        false,
    ));
    tools.push(static_tool(
        "credit-notes.get-pdf",
        "cho credit-notes get-pdf <id> [--output <path>]",
        "credit-notes",
        "Get credit note PDF payload or write decoded PDF bytes",
        true,
    ));
    tools.push(static_tool(
        "estimates.transition",
        "cho estimates transition <id> <action>",
        "estimates",
        "Trigger estimate status transition",
        false,
    ));
    tools.push(static_tool(
        "estimates.send-email",
        "cho estimates send-email <id> [--file <path>]",
        "estimates",
        "Send estimate email",
        false,
    ));
    tools.push(static_tool(
        "estimates.duplicate",
        "cho estimates duplicate <id>",
        "estimates",
        "Duplicate estimate",
        false,
    ));
    tools.push(static_tool(
        "estimates.get-pdf",
        "cho estimates get-pdf <id> [--output <path>]",
        "estimates",
        "Get estimate PDF payload or write decoded PDF bytes",
        true,
    ));
    tools.push(static_tool(
        "estimates.default-additional-text.get",
        "cho estimates default-additional-text get",
        "estimates",
        "Get estimate default additional text",
        true,
    ));
    tools.push(static_tool(
        "estimates.default-additional-text.update",
        "cho estimates default-additional-text update --file <path>",
        "estimates",
        "Update estimate default additional text",
        false,
    ));
    tools.push(static_tool(
        "estimates.default-additional-text.delete",
        "cho estimates default-additional-text delete",
        "estimates",
        "Delete estimate default additional text",
        false,
    ));
    tools.push(static_tool(
        "bank-transactions.upload-statement",
        "cho bank-transactions upload-statement --bank-account <url> --file <path>",
        "bank-transactions",
        "Upload bank statement CSV for account",
        false,
    ));
    tools.push(static_tool(
        "bank-transactions.for-approval",
        "cho bank-transactions for-approval",
        "bank-transactions",
        "List bank transactions marked for review",
        true,
    ));
    tools.push(static_tool(
        "bank-transactions.delete",
        "cho bank-transactions delete <id>",
        "bank-transactions",
        "Delete bank transaction by id",
        false,
    ));
    tools.push(static_tool(
        "bank-transactions.update-explanation",
        "cho bank-transactions update-explanation <transaction> [--description <text>] [--mark-for-review <bool>] [--attachment <path>]",
        "bank-transactions",
        "Update explanation fields for a transaction; supports local attachment path",
        false,
    ));
    tools.push(static_tool(
        "expenses.mileage-settings",
        "cho expenses mileage-settings",
        "expenses",
        "Get expense mileage settings",
        true,
    ));
    tools.push(static_tool(
        "journal-sets.opening-balances",
        "cho journal-sets opening-balances",
        "journal-sets",
        "Get journal set opening balances",
        true,
    ));
    tools.push(static_tool(
        "users.me",
        "cho users me",
        "users",
        "Get authenticated user",
        true,
    ));
    tools.push(static_tool(
        "users.update-me",
        "cho users update-me --file <path>",
        "users",
        "Update authenticated user",
        false,
    ));
    tools.push(static_tool(
        "timeslips.start-timer",
        "cho timeslips start-timer <id>",
        "timeslips",
        "Start timer on a timeslip",
        false,
    ));
    tools.push(static_tool(
        "timeslips.stop-timer",
        "cho timeslips stop-timer <id>",
        "timeslips",
        "Stop timer on a timeslip",
        false,
    ));
    tools.push(static_tool(
        "reports.balance-sheet-opening-balances",
        "cho reports balance-sheet-opening-balances",
        "reports",
        "Get balance sheet opening balances",
        true,
    ));
    tools.push(static_tool(
        "reports.trial-balance-opening-balances",
        "cho reports trial-balance-opening-balances",
        "reports",
        "Get trial balance opening balances",
        true,
    ));
    tools.push(static_tool(
        "corporation-tax-returns.mark-filed",
        "cho corporation-tax-returns mark-filed <period_ends_on>",
        "corporation-tax-returns",
        "Mark corporation tax return as filed",
        false,
    ));
    tools.push(static_tool(
        "corporation-tax-returns.mark-unfiled",
        "cho corporation-tax-returns mark-unfiled <period_ends_on>",
        "corporation-tax-returns",
        "Mark corporation tax return as unfiled",
        false,
    ));
    tools.push(static_tool(
        "corporation-tax-returns.mark-paid",
        "cho corporation-tax-returns mark-paid <period_ends_on>",
        "corporation-tax-returns",
        "Mark corporation tax return as paid",
        false,
    ));
    tools.push(static_tool(
        "corporation-tax-returns.mark-unpaid",
        "cho corporation-tax-returns mark-unpaid <period_ends_on>",
        "corporation-tax-returns",
        "Mark corporation tax return as unpaid",
        false,
    ));
    tools.push(static_tool(
        "vat-returns.mark-filed",
        "cho vat-returns mark-filed <period_ends_on>",
        "vat-returns",
        "Mark VAT return as filed",
        false,
    ));
    tools.push(static_tool(
        "vat-returns.mark-unfiled",
        "cho vat-returns mark-unfiled <period_ends_on>",
        "vat-returns",
        "Mark VAT return as unfiled",
        false,
    ));
    tools.push(static_tool(
        "vat-returns.mark-payment-paid",
        "cho vat-returns mark-payment-paid <period_ends_on> <payment_date>",
        "vat-returns",
        "Mark VAT payment as paid",
        false,
    ));
    tools.push(static_tool(
        "vat-returns.mark-payment-unpaid",
        "cho vat-returns mark-payment-unpaid <period_ends_on> <payment_date>",
        "vat-returns",
        "Mark VAT payment as unpaid",
        false,
    ));
    tools.push(static_tool(
        "final-accounts-reports.mark-filed",
        "cho final-accounts-reports mark-filed <period_ends_on>",
        "final-accounts-reports",
        "Mark final accounts report as filed",
        false,
    ));
    tools.push(static_tool(
        "final-accounts-reports.mark-unfiled",
        "cho final-accounts-reports mark-unfiled <period_ends_on>",
        "final-accounts-reports",
        "Mark final accounts report as unfiled",
        false,
    ));

    tools.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.name.cmp(&b.name))
    });

    tools
}

fn static_tool(
    name: &'static str,
    command: &'static str,
    category: &'static str,
    description: &'static str,
    idempotent: bool,
) -> ToolMeta {
    ToolMeta {
        name: name.to_string(),
        command: command.to_string(),
        category: category.to_string(),
        description: description.to_string(),
        parameters: vec![],
        output_fields: vec![],
        output_schema: None,
        input_schema: None,
        idempotent,
        rate_limit: Some("freeagent-user".to_string()),
        example: command.to_string(),
    }
}

fn static_tool_owned(
    name: String,
    command: String,
    category: String,
    description: String,
    idempotent: bool,
) -> ToolMeta {
    ToolMeta {
        name,
        command: command.clone(),
        category,
        description,
        parameters: vec![],
        output_fields: vec![],
        output_schema: None,
        input_schema: None,
        idempotent,
        rate_limit: Some("freeagent-user".to_string()),
        example: command,
    }
}
