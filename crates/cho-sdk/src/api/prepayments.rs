//! Prepayments API: list and get prepayments.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{ListResult, PaginatedResponse, PaginationParams};
use crate::http::request::ListParams;
use crate::models::common::Pagination;
use crate::models::prepayment::{Prepayment, Prepayments};

impl PaginatedResponse for Prepayments {
    type Item = Prepayment;

    fn into_items(self) -> Vec<Prepayment> {
        self.prepayments.unwrap_or_default()
    }

    fn pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }
}

/// API handle for prepayment operations.
pub struct PrepaymentsApi<'a> {
    client: &'a XeroClient,
}

impl<'a> PrepaymentsApi<'a> {
    /// Creates a new prepayments API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists prepayments with optional filtering and pagination.
    pub async fn list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<Prepayment>> {
        self.client
            .get_all_pages::<Prepayments>("Prepayments", params, pagination)
            .await
    }

    /// Gets a single prepayment by ID.
    pub async fn get(&self, id: Uuid) -> Result<Prepayment> {
        let response: Prepayments = self.client.get(&format!("Prepayments/{id}"), &[]).await?;

        response
            .prepayments
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "Prepayment".to_string(),
                id: id.to_string(),
            })
    }
}
