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
///
/// Defaults are read-only to follow least privilege. Use `CHO_SCOPES` to
/// explicitly opt into write scopes when required.
const DEFAULT_SCOPES: &str = "accounting.transactions.read accounting.contacts.read accounting.settings.read accounting.reports.read accounting.journals.read accounting.budgets.read";

/// Authenticates using the client_credentials grant type.
///
/// Custom Connections tokens expire after 30 minutes. Since there is no
/// refresh token, a new access token must be requested each time.
pub(crate) async fn authenticate(
    client: &reqwest::Client,
    params: &ClientCredentialsParams,
) -> crate::error::Result<TokenResponse> {
    authenticate_at(client, params, TOKEN_ENDPOINT).await
}

pub(crate) async fn authenticate_at(
    client: &reqwest::Client,
    params: &ClientCredentialsParams,
    endpoint: &str,
) -> crate::error::Result<TokenResponse> {
    let scopes = params.scopes.as_deref().unwrap_or(DEFAULT_SCOPES);

    debug!("Requesting client_credentials token");

    let response = client
        .post(endpoint)
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
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn default_scopes_do_not_include_offline_access() {
        assert!(!DEFAULT_SCOPES.contains("offline_access"));
    }

    #[test]
    fn default_scopes_exclude_payroll_and_projects() {
        assert!(!DEFAULT_SCOPES.contains("payroll."));
        assert!(!DEFAULT_SCOPES.contains("projects."));
    }

    #[test]
    fn default_scopes_are_read_only() {
        let scopes: std::collections::HashSet<&str> = DEFAULT_SCOPES.split_whitespace().collect();
        assert!(scopes.contains("accounting.transactions.read"));
        assert!(scopes.contains("accounting.contacts.read"));
        assert!(scopes.contains("accounting.settings.read"));
        assert!(!scopes.contains("accounting.transactions"));
        assert!(!scopes.contains("accounting.contacts"));
        assert!(!scopes.contains("accounting.settings"));
    }

    #[tokio::test]
    async fn authenticate_at_success_with_scope_override() {
        let server = MockServer::start().await;
        let endpoint = format!("{}/connect/token", server.uri());

        Mock::given(method("POST"))
            .and(path("/connect/token"))
            .and(body_string_contains("grant_type=client_credentials"))
            .and(body_string_contains("scope=accounting.transactions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "cc-access",
                "expires_in": 1800,
                "token_type": "Bearer",
                "scope": "accounting.transactions accounting.contacts"
            })))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let params = ClientCredentialsParams {
            client_id: "test-client-id".to_string(),
            client_secret: SecretString::from("secret".to_string()),
            scopes: Some("accounting.transactions accounting.contacts".to_string()),
        };

        let response = authenticate_at(&client, &params, &endpoint)
            .await
            .expect("authenticate succeeds");
        assert_eq!(response.access_token, "cc-access");
    }
}
