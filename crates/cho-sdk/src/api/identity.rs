//! Identity API: list connected organisations (tenants).

use crate::client::XeroClient;
use crate::error::Result;
use crate::models::connection::Connection;

/// Xero Identity API connections endpoint.
const CONNECTIONS_URL: &str = "https://api.xero.com/connections";

/// API handle for identity operations (connections/tenants).
pub struct IdentityApi<'a> {
    client: &'a XeroClient,
}

impl<'a> IdentityApi<'a> {
    /// Creates a new identity API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists all connected organisations (tenants).
    ///
    /// The connections endpoint returns a flat JSON array (not wrapped in
    /// an envelope), and uses camelCase (not PascalCase).
    pub async fn connections(&self) -> Result<Vec<Connection>> {
        self.client
            .get_absolute::<Vec<Connection>>(CONNECTIONS_URL, &[])
            .await
    }
}
