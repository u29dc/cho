//! Manual Journals API: list and get manual journals.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{ListResult, PaginatedResponse, PaginationParams};
use crate::http::request::ListParams;
use crate::models::common::Pagination;
use crate::models::manual_journal::{ManualJournal, ManualJournals};

impl PaginatedResponse for ManualJournals {
    type Item = ManualJournal;

    fn into_items(self) -> Vec<ManualJournal> {
        self.manual_journals.unwrap_or_default()
    }

    fn pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }
}

/// API handle for manual journal operations.
pub struct ManualJournalsApi<'a> {
    client: &'a XeroClient,
}

impl<'a> ManualJournalsApi<'a> {
    /// Creates a new manual journals API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists manual journals with optional filtering and pagination.
    pub async fn list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<ListResult<ManualJournal>> {
        self.client
            .get_all_pages::<ManualJournals>("ManualJournals", params, pagination)
            .await
    }

    /// Gets a single manual journal by ID.
    pub async fn get(&self, id: Uuid) -> Result<ManualJournal> {
        let response: ManualJournals = self
            .client
            .get(&format!("ManualJournals/{id}"), &[])
            .await?;

        response
            .manual_journals
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "ManualJournal".to_string(),
                id: id.to_string(),
            })
    }
}
