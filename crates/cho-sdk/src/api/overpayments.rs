//! Overpayments API: list and get overpayments.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{PaginatedResponse, PaginationParams};
use crate::http::request::ListParams;
use crate::models::common::Pagination;
use crate::models::overpayment::{Overpayment, Overpayments};

impl PaginatedResponse for Overpayments {
    type Item = Overpayment;

    fn into_items(self) -> Vec<Overpayment> {
        self.overpayments.unwrap_or_default()
    }

    fn pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }
}

/// API handle for overpayment operations.
pub struct OverpaymentsApi<'a> {
    client: &'a XeroClient,
}

impl<'a> OverpaymentsApi<'a> {
    /// Creates a new overpayments API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists overpayments with optional filtering and pagination.
    pub async fn list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<Overpayment>> {
        self.client
            .get_all_pages::<Overpayments>("Overpayments", params, pagination)
            .await
    }

    /// Gets a single overpayment by ID.
    pub async fn get(&self, id: Uuid) -> Result<Overpayment> {
        let response: Overpayments =
            self.client.get(&format!("Overpayments/{id}"), &[]).await?;

        response
            .overpayments
            .and_then(|mut v| if v.is_empty() { None } else { Some(v.remove(0)) })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "Overpayment".to_string(),
                id: id.to_string(),
            })
    }
}
