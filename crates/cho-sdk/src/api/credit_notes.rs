//! Credit Notes API: list and get credit notes.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{PaginatedResponse, PaginationParams};
use crate::http::request::ListParams;
use crate::models::common::Pagination;
use crate::models::credit_note::{CreditNote, CreditNotes};

impl PaginatedResponse for CreditNotes {
    type Item = CreditNote;

    fn into_items(self) -> Vec<CreditNote> {
        self.credit_notes.unwrap_or_default()
    }

    fn pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }
}

/// API handle for credit note operations.
pub struct CreditNotesApi<'a> {
    client: &'a XeroClient,
}

impl<'a> CreditNotesApi<'a> {
    /// Creates a new credit notes API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists credit notes with optional filtering and pagination.
    pub async fn list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<CreditNote>> {
        self.client
            .get_all_pages::<CreditNotes>("CreditNotes", params, pagination)
            .await
    }

    /// Gets a single credit note by ID.
    pub async fn get(&self, id: Uuid) -> Result<CreditNote> {
        let response: CreditNotes = self.client.get(&format!("CreditNotes/{id}"), &[]).await?;

        response
            .credit_notes
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "CreditNote".to_string(),
                id: id.to_string(),
            })
    }
}
