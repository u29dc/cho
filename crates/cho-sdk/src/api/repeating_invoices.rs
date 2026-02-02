//! Repeating Invoices API: list and get repeating invoices.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{PaginatedResponse, PaginationParams};
use crate::http::request::ListParams;
use crate::models::common::Pagination;
use crate::models::repeating_invoice::{RepeatingInvoice, RepeatingInvoices};

/// Repeating invoices don't have standard pagination from Xero, but we
/// implement the trait for consistency with the `get_all_pages` pattern.
impl PaginatedResponse for RepeatingInvoices {
    type Item = RepeatingInvoice;

    fn into_items(self) -> Vec<RepeatingInvoice> {
        self.repeating_invoices.unwrap_or_default()
    }

    fn pagination(&self) -> Option<&Pagination> {
        // RepeatingInvoices collection doesn't include pagination
        None
    }
}

/// API handle for repeating invoice operations.
pub struct RepeatingInvoicesApi<'a> {
    client: &'a XeroClient,
}

impl<'a> RepeatingInvoicesApi<'a> {
    /// Creates a new repeating invoices API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists repeating invoices with optional filtering.
    ///
    /// The RepeatingInvoices endpoint is paginated in Xero's API.
    pub async fn list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<RepeatingInvoice>> {
        self.client
            .get_all_pages::<RepeatingInvoices>("RepeatingInvoices", params, pagination)
            .await
    }

    /// Gets a single repeating invoice by ID.
    pub async fn get(&self, id: Uuid) -> Result<RepeatingInvoice> {
        let response: RepeatingInvoices = self
            .client
            .get(&format!("RepeatingInvoices/{id}"), &[])
            .await?;

        response
            .repeating_invoices
            .and_then(|mut v| if v.is_empty() { None } else { Some(v.remove(0)) })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "RepeatingInvoice".to_string(),
                id: id.to_string(),
            })
    }
}
