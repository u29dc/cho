//! Resource specifications for FreeAgent endpoints.

/// Resource endpoint metadata.
#[derive(Debug, Clone, Copy)]
pub struct ResourceSpec {
    /// Tool category and logical resource name.
    pub name: &'static str,
    /// API path relative to `/v2/`.
    pub path: &'static str,
    /// Collection key in list responses.
    pub collection_key: &'static str,
    /// Singular key in single/create/update responses.
    pub singular_key: &'static str,
    /// Whether the resource supports list/get/create/update/delete.
    pub capabilities: ResourceCapabilities,
}

/// Supported CRUD capabilities.
#[derive(Debug, Clone, Copy)]
pub struct ResourceCapabilities {
    /// List support.
    pub list: bool,
    /// Get support.
    pub get: bool,
    /// Create support.
    pub create: bool,
    /// Update support.
    pub update: bool,
    /// Delete support.
    pub delete: bool,
}

const fn caps(
    list: bool,
    get: bool,
    create: bool,
    update: bool,
    delete: bool,
) -> ResourceCapabilities {
    ResourceCapabilities {
        list,
        get,
        create,
        update,
        delete,
    }
}

/// Generic CRUD-oriented resources.
pub const RESOURCES: &[ResourceSpec] = &[
    ResourceSpec {
        name: "contacts",
        path: "contacts",
        collection_key: "contacts",
        singular_key: "contact",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "invoices",
        path: "invoices",
        collection_key: "invoices",
        singular_key: "invoice",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "bank-accounts",
        path: "bank_accounts",
        collection_key: "bank_accounts",
        singular_key: "bank_account",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "bank-transactions",
        path: "bank_transactions",
        collection_key: "bank_transactions",
        singular_key: "bank_transaction",
        capabilities: caps(true, true, false, false, false),
    },
    ResourceSpec {
        name: "bank-transaction-explanations",
        path: "bank_transaction_explanations",
        collection_key: "bank_transaction_explanations",
        singular_key: "bank_transaction_explanation",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "bills",
        path: "bills",
        collection_key: "bills",
        singular_key: "bill",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "expenses",
        path: "expenses",
        collection_key: "expenses",
        singular_key: "expense",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "categories",
        path: "categories",
        collection_key: "categories",
        singular_key: "category",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "transactions",
        path: "accounting/transactions",
        collection_key: "transactions",
        singular_key: "transaction",
        capabilities: caps(true, true, false, false, false),
    },
    ResourceSpec {
        name: "corporation-tax-returns",
        path: "corporation_tax_returns",
        collection_key: "corporation_tax_returns",
        singular_key: "corporation_tax_return",
        capabilities: caps(true, true, false, false, false),
    },
    ResourceSpec {
        name: "vat-returns",
        path: "vat_returns",
        collection_key: "vat_returns",
        singular_key: "vat_return",
        capabilities: caps(true, true, false, false, false),
    },
    ResourceSpec {
        name: "final-accounts-reports",
        path: "final_accounts_reports",
        collection_key: "final_accounts_reports",
        singular_key: "final_accounts_report",
        capabilities: caps(true, true, false, false, false),
    },
    ResourceSpec {
        name: "sales-tax-periods",
        path: "sales_tax_periods",
        collection_key: "sales_tax_periods",
        singular_key: "sales_tax_period",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "credit-notes",
        path: "credit_notes",
        collection_key: "credit_notes",
        singular_key: "credit_note",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "estimates",
        path: "estimates",
        collection_key: "estimates",
        singular_key: "estimate",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "recurring-invoices",
        path: "recurring_invoices",
        collection_key: "recurring_invoices",
        singular_key: "recurring_invoice",
        capabilities: caps(true, true, false, false, false),
    },
    ResourceSpec {
        name: "journal-sets",
        path: "journal_sets",
        collection_key: "journal_sets",
        singular_key: "journal_set",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "users",
        path: "users",
        collection_key: "users",
        singular_key: "user",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "capital-assets",
        path: "capital_assets",
        collection_key: "capital_assets",
        singular_key: "capital_asset",
        capabilities: caps(true, true, false, false, false),
    },
    ResourceSpec {
        name: "stock-items",
        path: "stock_items",
        collection_key: "stock_items",
        singular_key: "stock_item",
        capabilities: caps(true, true, false, false, false),
    },
    ResourceSpec {
        name: "projects",
        path: "projects",
        collection_key: "projects",
        singular_key: "project",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "timeslips",
        path: "timeslips",
        collection_key: "timeslips",
        singular_key: "timeslip",
        capabilities: caps(true, true, true, true, true),
    },
    ResourceSpec {
        name: "attachments",
        path: "attachments",
        collection_key: "attachments",
        singular_key: "attachment",
        capabilities: caps(false, true, false, false, true),
    },
];

/// Looks up a resource by CLI name.
pub fn by_name(name: &str) -> Option<ResourceSpec> {
    RESOURCES.iter().copied().find(|spec| spec.name == name)
}
