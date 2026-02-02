//! Accounts API: list chart of accounts.

use crate::client::XeroClient;
use crate::error::Result;
use crate::http::request::ListParams;
use crate::models::account::{Account, Accounts};

/// API handle for account operations.
pub struct AccountsApi<'a> {
    client: &'a XeroClient,
}

impl<'a> AccountsApi<'a> {
    /// Creates a new accounts API handle.
    pub(crate) fn new(client: &'a XeroClient) -> Self {
        Self { client }
    }

    /// Lists all accounts (chart of accounts).
    ///
    /// The Accounts endpoint is not paginated — it returns all accounts
    /// in a single response.
    pub async fn list(&self, params: &ListParams) -> Result<Vec<Account>> {
        let query = params.to_query_pairs();
        let response: Accounts = self.client.get("Accounts", &query).await?;
        Ok(response.accounts.unwrap_or_default())
    }
}
