//! API modules providing typed access to Xero resources.
//!
//! Each module exposes an API handle struct (e.g., `InvoicesApi`) that is
//! obtained via the corresponding method on [`XeroClient`](crate::client::XeroClient).

pub mod accounts;
pub mod bank_transactions;
pub mod contacts;
pub mod identity;
pub mod invoices;
pub mod payments;
pub mod reports;
