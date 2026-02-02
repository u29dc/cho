//! Contacts API: list, get, search, create, and update contacts.

use uuid::Uuid;

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::pagination::{PaginatedResponse, PaginationParams};
use crate::http::request::ListParams;
use crate::models::common::Pagination;
use crate::models::contact::{Contact, Contacts};

impl PaginatedResponse for Contacts {
    type Item = Contact;

    fn into_items(self) -> Vec<Contact> {
        self.contacts.unwrap_or_default()
    }

    fn pagination(&self) -> Option<&Pagination> {
        self.pagination.as_ref()
    }
}

/// API handle for contact operations.
pub struct ContactsApi<'a> {
    client: &'a XeroClient,
}

impl<'a> ContactsApi<'a> {
    /// Creates a new contacts API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists contacts with optional filtering and pagination.
    pub async fn list(
        &self,
        params: &ListParams,
        pagination: &PaginationParams,
    ) -> Result<Vec<Contact>> {
        self.client
            .get_all_pages::<Contacts>("Contacts", params, pagination)
            .await
    }

    /// Gets a single contact by ID.
    pub async fn get(&self, id: Uuid) -> Result<Contact> {
        let response: Contacts = self.client.get(&format!("Contacts/{id}"), &[]).await?;

        response
            .contacts
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "Contact".to_string(),
                id: id.to_string(),
            })
    }

    /// Creates a new contact.
    pub async fn create(
        &self,
        contact: &Contact,
        idempotency_key: Option<&str>,
    ) -> Result<Contact> {
        let wrapper = Contacts {
            contacts: Some(vec![contact.clone()]),
            pagination: None,
            warnings: None,
        };

        let response: Contacts = self
            .client
            .put("Contacts", &wrapper, idempotency_key)
            .await?;

        response
            .contacts
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::Parse {
                message: "No contact returned in create response".to_string(),
            })
    }

    /// Updates an existing contact.
    pub async fn update(
        &self,
        id: Uuid,
        contact: &Contact,
        idempotency_key: Option<&str>,
    ) -> Result<Contact> {
        let wrapper = Contacts {
            contacts: Some(vec![contact.clone()]),
            pagination: None,
            warnings: None,
        };

        let response: Contacts = self
            .client
            .post(&format!("Contacts/{id}"), &wrapper, idempotency_key)
            .await?;

        response
            .contacts
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::Parse {
                message: "No contact returned in update response".to_string(),
            })
    }

    /// Searches contacts by name, email, or other fields using Xero's
    /// built-in search term parameter.
    pub async fn search(&self, term: &str, pagination: &PaginationParams) -> Result<Vec<Contact>> {
        let params = ListParams::new().with_search_term(term);
        self.client
            .get_all_pages::<Contacts>("Contacts", &params, pagination)
            .await
    }
}
