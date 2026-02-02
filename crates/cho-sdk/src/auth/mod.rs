//! Authentication and token management for the Xero API.
//!
//! Supports two authentication methods:
//! - **PKCE flow** (default): Browser-based OAuth 2.0 for interactive use.
//! - **Client credentials**: Headless server-to-server for Custom Connections.
//!
//! Token lifecycle is managed automatically: tokens are stored in the OS
//! keychain (with file fallback), and refreshed before expiry.

pub mod credentials;
pub mod pkce;
pub mod storage;
pub mod token;

use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::debug;

use self::token::{TokenPair, refresh_access_token};

/// Manages authentication state and automatic token refresh.
///
/// Wraps a [`TokenPair`] behind an async `RwLock` so it can be shared
/// across concurrent requests and refreshed transparently.
pub struct AuthManager {
    /// Current token pair (may be None if not yet authenticated).
    token: Arc<RwLock<Option<TokenPair>>>,

    /// Client ID for token refresh.
    client_id: String,

    /// HTTP client for token endpoint requests.
    http_client: reqwest::Client,
}

impl AuthManager {
    /// Creates a new auth manager.
    pub fn new(client_id: String) -> Self {
        Self {
            token: Arc::new(RwLock::new(None)),
            client_id,
            http_client: reqwest::Client::new(),
        }
    }

    /// Creates an auth manager with a pre-existing token pair.
    pub fn with_token(client_id: String, token: TokenPair) -> Self {
        Self {
            token: Arc::new(RwLock::new(Some(token))),
            client_id,
            http_client: reqwest::Client::new(),
        }
    }

    /// Attempts to load tokens from storage.
    pub fn load_stored_tokens(&self) -> crate::error::Result<bool> {
        if let Some(stored) = storage::load_tokens()? {
            if stored.is_expired() && stored.refresh_token.is_none() {
                debug!("Stored tokens expired with no refresh token");
                return Ok(false);
            }
            let pair = TokenPair::from_stored(&stored);
            // We can't set the RwLock synchronously from an async context,
            // but this is called during initialization, so blocking is fine.
            let mut guard = self.token.blocking_write();
            *guard = Some(pair);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Runs the PKCE flow interactively, storing the resulting tokens.
    pub async fn login_pkce(&self, port: u16) -> crate::error::Result<()> {
        let params = pkce::PkceFlowParams {
            client_id: self.client_id.clone(),
            port,
            scopes: None,
        };

        let response = pkce::run_pkce_flow(&params).await?;
        let pair = TokenPair::from_response(&response);

        // Store tokens
        let stored = pair.to_stored();
        storage::store_tokens(&stored)?;

        let mut guard = self.token.write().await;
        *guard = Some(pair);

        Ok(())
    }

    /// Authenticates using client credentials (Custom Connections).
    pub async fn login_client_credentials(
        &self,
        client_secret: secrecy::SecretString,
    ) -> crate::error::Result<()> {
        let params = credentials::ClientCredentialsParams {
            client_id: self.client_id.clone(),
            client_secret,
            scopes: None,
        };

        let response = credentials::authenticate(&self.http_client, &params).await?;
        let pair = TokenPair::from_response(&response);

        // Client credentials tokens are not persisted (no refresh token, short-lived)
        let mut guard = self.token.write().await;
        *guard = Some(pair);

        Ok(())
    }

    /// Returns a valid access token, refreshing if necessary.
    ///
    /// This is the primary method called by the HTTP layer before each request.
    pub async fn get_access_token(&self) -> crate::error::Result<String> {
        // Fast path: token exists and is valid
        {
            let guard = self.token.read().await;
            if let Some(ref pair) = *guard
                && !pair.needs_refresh()
            {
                return Ok(pair.access_token().to_string());
            }
        }

        // Slow path: need to refresh
        self.refresh().await?;

        let guard = self.token.read().await;
        match *guard {
            Some(ref pair) => Ok(pair.access_token().to_string()),
            None => Err(crate::error::ChoSdkError::AuthRequired {
                message: "No valid token available. Please login first.".to_string(),
            }),
        }
    }

    /// Forces a token refresh using the current refresh token.
    pub async fn refresh(&self) -> crate::error::Result<()> {
        let refresh_token = {
            let guard = self.token.read().await;
            match *guard {
                Some(ref pair) => match pair.refresh_token() {
                    Some(rt) => rt.to_string(),
                    None => {
                        return Err(crate::error::ChoSdkError::TokenExpired {
                            message: "No refresh token available. Please login again.".to_string(),
                        });
                    }
                },
                None => {
                    return Err(crate::error::ChoSdkError::AuthRequired {
                        message: "No token available. Please login first.".to_string(),
                    });
                }
            }
        };

        debug!("Refreshing access token");
        let response =
            refresh_access_token(&self.http_client, &self.client_id, &refresh_token).await?;
        let pair = TokenPair::from_response(&response);

        // Persist the new tokens (refresh tokens are single-use)
        let stored = pair.to_stored();
        storage::store_tokens(&stored)?;

        let mut guard = self.token.write().await;
        *guard = Some(pair);

        debug!("Token refreshed successfully");
        Ok(())
    }

    /// Returns true if there is a valid (non-expired) token.
    pub async fn is_authenticated(&self) -> bool {
        let guard = self.token.read().await;
        match *guard {
            Some(ref pair) => !pair.is_expired(),
            None => false,
        }
    }

    /// Clears stored tokens and logs out.
    pub async fn logout(&self) -> crate::error::Result<()> {
        storage::clear_tokens()?;
        let mut guard = self.token.write().await;
        *guard = None;
        Ok(())
    }
}

impl std::fmt::Debug for AuthManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthManager")
            .field("client_id", &self.client_id)
            .field("has_token", &"<check is_authenticated()>")
            .finish()
    }
}
