//! Quotes API: list and get quotes.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{ListResult, PaginatedResponse, PaginationParams};
use crate::http::request::ListParams;
use crate::models::common::Pagination;
use crate::models::quote::{Quote, Quotes};

impl PaginatedResponse for Quotes {
    type Item = Quote;

    fn into_items(self) -> Vec<Quote> {
        self.quotes.unwrap_or_default()
    }

    fn pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }
}

/// API handle for quote operations.
pub struct QuotesApi<'a> {
    client: &'a XeroClient,
}

impl<'a> QuotesApi<'a> {
    /// Creates a new quotes API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists quotes with optional filtering and pagination.
    pub async fn list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<Quote>> {
        self.client
            .get_all_pages::<Quotes>("Quotes", params, pagination)
            .await
    }

    /// Gets a single quote by ID.
    pub async fn get(&self, id: Uuid) -> Result<Quote> {
        let response: Quotes = self.client.get(&format!("Quotes/{id}"), &[]).await?;

        response
            .quotes
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "Quote".to_string(),
                id: id.to_string(),
            })
    }
}
