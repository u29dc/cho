//! Organisations API: get organisation details.

use crate::client::XeroClient;
use crate::error::Result;
use crate::models::organisation::{Organisation, Organisations};

/// API handle for organisation operations.
pub struct OrganisationsApi<'a> {
    client: &'a XeroClient,
}

impl<'a> OrganisationsApi<'a> {
    /// Creates a new organisations API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Gets the organisation details for the current tenant.
    ///
    /// The Organisation endpoint returns a single organisation
    /// (the one connected to the current tenant).
    pub async fn get(&self) -> Result<Organisation> {
        let response: Organisations = self.client.get("Organisation", &[]).await?;

        response
            .organisations
            .and_then(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.remove(0))
                }
            })
            .ok_or_else(|| crate::error::ChoSdkError::NotFound {
                resource: "Organisation".to_string(),
                id: String::new(),
            })
    }
}
