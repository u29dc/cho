//! Authentication manager for FreeAgent OAuth.

pub mod oauth;
pub mod storage;
pub mod token;

use std::sync::Arc;

use secrecy::{ExposeSecret, SecretString};
use tokio::sync::{Mutex, RwLock};

use crate::config::SdkConfig;
use crate::error::{ChoSdkError, Result};
use crate::models::TokenStatus;

use self::token::{TokenPair, TokenResponse};

/// Login flow output details.
#[derive(Debug, Clone)]
pub struct LoginResult {
    /// Authorization URL used.
    pub authorize_url: String,
    /// Redirect URI used.
    pub redirect_uri: String,
}

/// Authentication manager.
pub struct AuthManager {
    client_id: String,
    client_secret: SecretString,
    config: SdkConfig,
    http_client: reqwest::Client,
    token: Arc<RwLock<Option<TokenPair>>>,
    persist_tokens: bool,
    refresh_lock: Mutex<()>,
}

impl AuthManager {
    /// Creates a new auth manager.
    pub fn new(client_id: String, client_secret: SecretString, config: SdkConfig) -> Result<Self> {
        if client_id.trim().is_empty() {
            return Err(ChoSdkError::Config {
                message: "Client ID is required for authentication".to_string(),
            });
        }

        if client_secret.expose_secret().trim().is_empty() {
            return Err(ChoSdkError::Config {
                message: "Client secret is required for authentication".to_string(),
            });
        }

        let http_client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(ChoSdkError::Network)?;

        Ok(Self {
            client_id,
            client_secret,
            config,
            http_client,
            token: Arc::new(RwLock::new(None)),
            persist_tokens: true,
            refresh_lock: Mutex::new(()),
        })
    }

    /// Enables or disables persistent token storage side effects.
    pub fn with_token_persistence(mut self, persist_tokens: bool) -> Self {
        self.persist_tokens = persist_tokens;
        self
    }

    /// Returns client ID.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Loads cached tokens from storage.
    pub async fn load_stored_tokens(&self) -> Result<bool> {
        if let Some(stored) = storage::load_tokens()? {
            let pair = TokenPair::from_stored(&stored);
            let mut guard = self.token.write().await;
            *guard = Some(pair);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Clears tokens from memory and storage.
    pub async fn logout(&self) -> Result<()> {
        storage::clear_tokens()?;
        let mut guard = self.token.write().await;
        *guard = None;
        Ok(())
    }

    /// Returns current token status.
    pub async fn status(&self) -> TokenStatus {
        let guard = self.token.read().await;
        match guard.as_ref() {
            Some(pair) => TokenStatus {
                authenticated: !pair.is_expired(),
                expires_at: Some(pair.expires_at().to_rfc3339()),
                expires_in_seconds: Some(pair.expires_in_seconds()),
                token_state: Some(if !pair.is_expired() {
                    "valid".to_string()
                } else if pair.can_refresh() {
                    "refreshable_expired".to_string()
                } else {
                    "expired".to_string()
                }),
                can_refresh: Some(pair.can_refresh()),
                needs_refresh: Some(pair.needs_refresh()),
            },
            None => TokenStatus {
                authenticated: false,
                expires_at: None,
                expires_in_seconds: None,
                token_state: Some("missing".to_string()),
                can_refresh: Some(false),
                needs_refresh: Some(false),
            },
        }
    }

    /// Returns true when a valid access token is present.
    pub async fn is_authenticated(&self) -> bool {
        let guard = self.token.read().await;
        guard
            .as_ref()
            .map(|pair| !pair.is_expired())
            .unwrap_or(false)
    }

    /// Seeds in-memory tokens without touching persistent storage.
    pub async fn set_tokens_in_memory(&self, stored: token::StoredTokens) {
        let mut guard = self.token.write().await;
        *guard = Some(TokenPair::from_stored(&stored));
    }

    /// Runs browser login flow and stores resulting token pair.
    pub async fn login_browser(&self, port: u16, open_browser: bool) -> Result<LoginResult> {
        let (listener, redirect_uri) = oauth::start_callback_listener(port).await?;
        let state = oauth::random_state();

        let authorize_url = oauth::authorization_url(
            &self.config.authorize_url,
            &self.client_id,
            &redirect_uri,
            &state,
        )?;

        if open_browser {
            if let Err(err) = open::that(authorize_url.as_str()) {
                tracing::warn!("Failed to open browser automatically for OAuth login: {err}");
                eprintln!(
                    "Open this URL in your browser to continue authentication:\n{}",
                    authorize_url
                );
            }
        } else {
            eprintln!(
                "Open this URL in your browser to continue authentication:\n{}",
                authorize_url
            );
        }

        let callback = oauth::receive_callback(listener, 300).await?;
        if callback.state.as_deref() != Some(state.as_str()) {
            return Err(ChoSdkError::AuthRequired {
                message: "OAuth state mismatch in callback".to_string(),
            });
        }

        let token_response = self
            .exchange_authorization_code(&callback.code, &redirect_uri)
            .await?;
        let pair = TokenPair::from_response(&token_response);

        self.store_pair(pair).await?;

        Ok(LoginResult {
            authorize_url: authorize_url.to_string(),
            redirect_uri,
        })
    }

    /// Exchanges authorization code for token pair.
    pub async fn exchange_authorization_code(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<TokenResponse> {
        let response = self
            .http_client
            .post(&self.config.token_url)
            .basic_auth(&self.client_id, Some(self.client_secret.expose_secret()))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await
            .map_err(ChoSdkError::Network)?;

        parse_token_response(response).await
    }

    /// Refreshes tokens using current refresh token.
    pub async fn refresh(&self) -> Result<()> {
        let _guard = self.refresh_lock.lock().await;

        let refresh_token = {
            let guard = self.token.read().await;
            let pair = guard.as_ref().ok_or_else(|| ChoSdkError::AuthRequired {
                message: "No token available, run 'cho auth login'".to_string(),
            })?;
            if !pair.can_refresh() {
                return Err(ChoSdkError::TokenExpired {
                    message: "Refresh token is unavailable or expired, run 'cho auth login'"
                        .to_string(),
                });
            }
            pair.refresh_token().unwrap_or_default().to_string()
        };

        let response = self
            .http_client
            .post(&self.config.token_url)
            .basic_auth(&self.client_id, Some(self.client_secret.expose_secret()))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.as_str()),
            ])
            .send()
            .await
            .map_err(ChoSdkError::Network)?;

        let token_response = parse_token_response(response).await?;
        let pair = TokenPair::from_response(&token_response);
        self.store_pair(pair).await
    }

    /// Returns a valid access token, refreshing when required.
    pub async fn get_access_token(&self) -> Result<String> {
        {
            let guard = self.token.read().await;
            if let Some(pair) = guard.as_ref()
                && !pair.needs_refresh()
            {
                return Ok(pair.access_token().to_string());
            }
        }

        self.refresh().await?;

        let guard = self.token.read().await;
        let pair = guard.as_ref().ok_or_else(|| ChoSdkError::AuthRequired {
            message: "No token available, run 'cho auth login'".to_string(),
        })?;

        Ok(pair.access_token().to_string())
    }

    async fn store_pair(&self, pair: TokenPair) -> Result<()> {
        if self.persist_tokens {
            storage::store_tokens(&pair.to_stored())?;
        }
        let mut guard = self.token.write().await;
        *guard = Some(pair);
        Ok(())
    }
}

impl std::fmt::Debug for AuthManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthManager")
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .field("config", &self.config)
            .field("token", &"[REDACTED]")
            .field("persist_tokens", &self.persist_tokens)
            .finish()
    }
}

async fn parse_token_response(response: reqwest::Response) -> Result<TokenResponse> {
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(match status {
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                ChoSdkError::AuthRequired {
                    message: format!("Token endpoint rejected credentials ({status}): {body}"),
                }
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => ChoSdkError::RateLimited { retry_after: 60 },
            _ => ChoSdkError::api(status, body),
        });
    }

    response
        .json::<TokenResponse>()
        .await
        .map_err(|e| ChoSdkError::Parse {
            message: format!("Failed to parse token endpoint response: {e}"),
        })
}
