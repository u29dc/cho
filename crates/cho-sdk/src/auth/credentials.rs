//! Custom Connections auth using `client_credentials` grant.
//!
//! This is a Xero paid feature for headless/server-to-server access to a
//! single organisation. No browser redirect needed; uses client_id +
//! client_secret to obtain an access token directly.

use secrecy::{ExposeSecret, SecretString};
use tracing::debug;

use super::token::{TOKEN_ENDPOINT, TokenResponse};

/// Parameters for client credentials authentication.
pub struct ClientCredentialsParams {
    /// Xero OAuth 2.0 client ID.
    pub client_id: String,

    /// Xero OAuth 2.0 client secret.
    pub client_secret: SecretString,

    /// OAuth scopes to request.
    pub scopes: Option<String>,
}

/// Default scopes for client credentials (no offline_access needed).
const DEFAULT_SCOPES: &str = "accounting.transactions.read accounting.contacts.read accounting.settings.read accounting.reports.read accounting.journals.read files.read assets.read projects.read payroll.employees payroll.timesheets payroll.settings";

/// Authenticates using the client_credentials grant type.
///
/// Custom Connections tokens expire after 30 minutes. Since there is no
/// refresh token, a new access token must be requested each time.
pub(crate) async fn authenticate(
    client: &reqwest::Client,
    params: &ClientCredentialsParams,
) -> crate::error::Result<TokenResponse> {
    let scopes = params.scopes.as_deref().unwrap_or(DEFAULT_SCOPES);

    debug!("Requesting client_credentials token");

    let response = client
        .post(TOKEN_ENDPOINT)
        .basic_auth(
            &params.client_id,
            Some(params.client_secret.expose_secret()),
        )
        .form(&[("grant_type", "client_credentials"), ("scope", scopes)])
        .send()
        .await
        .map_err(crate::error::ChoSdkError::Network)?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(crate::error::ChoSdkError::AuthRequired {
            message: format!("Client credentials auth failed (HTTP {status}): {body}"),
        });
    }

    response
        .json::<TokenResponse>()
        .await
        .map_err(|e| crate::error::ChoSdkError::Parse {
            message: format!("Failed to parse token response: {e}"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scopes_do_not_include_offline_access() {
        assert!(!DEFAULT_SCOPES.contains("offline_access"));
    }
}
