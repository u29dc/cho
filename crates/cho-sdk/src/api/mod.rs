//! API modules providing typed access to Xero resources.
//!
//! Each module exposes an API handle struct (e.g., `InvoicesApi`) that is
//! obtained via the corresponding method on [`XeroClient`](crate::client::XeroClient).

pub mod accounts;
pub mod bank_transactions;
pub mod budgets;
pub mod contacts;
pub mod credit_notes;
pub mod currencies;
pub mod identity;
pub mod invoices;
pub mod items;
pub mod linked_transactions;
pub mod manual_journals;
pub mod organisations;
pub mod overpayments;
pub mod payments;
pub mod prepayments;
pub mod purchase_orders;
pub mod quotes;
pub mod repeating_invoices;
pub mod reports;
pub mod tax_rates;
pub mod tracking_categories;
