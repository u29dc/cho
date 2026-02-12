//! Token pair management: access token, refresh token, and expiry tracking.

use std::time::{Duration, Instant};

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

/// Xero OAuth 2.0 token endpoint URL.
pub const TOKEN_ENDPOINT: &str = "https://identity.xero.com/connect/token";

/// Default access token lifetime (30 minutes).
const DEFAULT_TOKEN_LIFETIME: Duration = Duration::from_secs(30 * 60);

/// Safety margin before expiry to trigger refresh (5 minutes).
const REFRESH_MARGIN: Duration = Duration::from_secs(5 * 60);

/// Raw token response from the Xero token endpoint.
#[derive(Debug, Deserialize)]
pub(crate) struct TokenResponse {
    /// The access token.
    pub access_token: String,

    /// The refresh token (only present with `offline_access` scope).
    pub refresh_token: Option<String>,

    /// Token lifetime in seconds.
    pub expires_in: Option<u64>,

    /// Token type (always "Bearer").
    #[allow(dead_code)]
    pub token_type: Option<String>,

    /// Scopes granted.
    #[allow(dead_code)]
    pub scope: Option<String>,
}

/// Serializable token data for storage (without runtime expiry tracking).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoredTokens {
    /// The access token string.
    pub access_token: String,

    /// The refresh token string.
    pub refresh_token: Option<String>,

    /// Token lifetime in seconds at time of issue.
    pub expires_in: u64,

    /// Unix timestamp (seconds) when the token was issued.
    pub issued_at: u64,
}

impl StoredTokens {
    /// Returns true if the stored token has expired based on issued_at + expires_in.
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now >= self.issued_at + self.expires_in
    }

    /// Returns true if the stored token will expire within the refresh margin.
    #[cfg(test)]
    pub(crate) fn needs_refresh(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now + REFRESH_MARGIN.as_secs() >= self.issued_at + self.expires_in
    }
}

/// A live OAuth 2.0 token pair with runtime expiry tracking.
///
/// Access and refresh tokens are wrapped in [`SecretString`] to prevent
/// accidental logging or display.
pub struct TokenPair {
    /// The access token (Bearer token for API requests).
    access_token: SecretString,

    /// The refresh token for obtaining new access tokens.
    refresh_token: Option<SecretString>,

    /// When the access token expires (monotonic clock).
    expires_at: Instant,

    /// Token lifetime as returned by the server.
    expires_in: Duration,
}

impl TokenPair {
    /// Creates a token pair for testing purposes.
    ///
    /// The token will be valid for the specified number of seconds.
    pub fn for_testing(access_token: &str, expires_in_secs: u64) -> Self {
        let lifetime = Duration::from_secs(expires_in_secs);
        Self {
            access_token: SecretString::from(access_token.to_string()),
            refresh_token: None,
            expires_at: Instant::now() + lifetime,
            expires_in: lifetime,
        }
    }

    /// Creates a token pair for testing with a refresh token.
    pub fn for_testing_with_refresh(
        access_token: &str,
        refresh_token: &str,
        expires_in_secs: u64,
    ) -> Self {
        let lifetime = Duration::from_secs(expires_in_secs);
        Self {
            access_token: SecretString::from(access_token.to_string()),
            refresh_token: Some(SecretString::from(refresh_token.to_string())),
            expires_at: Instant::now() + lifetime,
            expires_in: lifetime,
        }
    }

    /// Creates a new token pair from a token endpoint response.
    pub(crate) fn from_response(response: &TokenResponse) -> Self {
        let lifetime = response
            .expires_in
            .map(Duration::from_secs)
            .unwrap_or(DEFAULT_TOKEN_LIFETIME);

        Self {
            access_token: SecretString::from(response.access_token.clone()),
            refresh_token: response
                .refresh_token
                .as_ref()
                .map(|t| SecretString::from(t.clone())),
            expires_at: Instant::now() + lifetime,
            expires_in: lifetime,
        }
    }

    /// Creates a token pair from stored token data.
    ///
    /// The expiry is estimated from the stored `issued_at` + `expires_in`,
    /// mapped onto the monotonic clock.
    pub(crate) fn from_stored(stored: &StoredTokens) -> Self {
        let lifetime = Duration::from_secs(stored.expires_in);
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let expiry_unix = stored.issued_at + stored.expires_in;
        let remaining = if now_unix < expiry_unix {
            Duration::from_secs(expiry_unix - now_unix)
        } else {
            Duration::ZERO
        };

        Self {
            access_token: SecretString::from(stored.access_token.clone()),
            refresh_token: stored
                .refresh_token
                .as_ref()
                .map(|t| SecretString::from(t.clone())),
            expires_at: Instant::now() + remaining,
            expires_in: lifetime,
        }
    }

    /// Returns the access token string for use in Authorization headers.
    pub fn access_token(&self) -> &str {
        self.access_token.expose_secret()
    }

    /// Returns the refresh token string, if available.
    pub fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_ref().map(|t| t.expose_secret())
    }

    /// Returns true if the token has expired.
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// Returns true if the token should be refreshed (within safety margin).
    pub fn needs_refresh(&self) -> bool {
        Instant::now() + REFRESH_MARGIN >= self.expires_at
    }

    /// Returns the remaining time until expiry, or zero if expired.
    pub fn time_until_expiry(&self) -> Duration {
        self.expires_at
            .checked_duration_since(Instant::now())
            .unwrap_or(Duration::ZERO)
    }

    /// Converts to a storable representation.
    pub(crate) fn to_stored(&self) -> StoredTokens {
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let remaining = self.time_until_expiry();
        let issued_at = now_unix.saturating_sub(
            self.expires_in
                .as_secs()
                .saturating_sub(remaining.as_secs()),
        );

        StoredTokens {
            access_token: self.access_token.expose_secret().to_string(),
            refresh_token: self
                .refresh_token
                .as_ref()
                .map(|t| t.expose_secret().to_string()),
            expires_in: self.expires_in.as_secs(),
            issued_at,
        }
    }
}

impl std::fmt::Debug for TokenPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenPair")
            .field("access_token", &"[REDACTED]")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

/// Refreshes an access token using a refresh token.
///
/// Xero refresh tokens are single-use: each refresh returns a new
/// access_token + refresh_token pair. The caller must store the new pair.
pub(crate) async fn refresh_access_token(
    client: &reqwest::Client,
    client_id: &str,
    refresh_token: &str,
) -> crate::error::Result<TokenResponse> {
    let params = [
        ("grant_type", "refresh_token"),
        ("client_id", client_id),
        ("refresh_token", refresh_token),
    ];

    let response = client
        .post(TOKEN_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(crate::error::ChoSdkError::Network)?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(crate::error::ChoSdkError::TokenExpired {
            message: format!("Token refresh failed (HTTP {status}): {body}"),
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
    fn token_pair_from_response() {
        let response = TokenResponse {
            access_token: "test_access".to_string(),
            refresh_token: Some("test_refresh".to_string()),
            expires_in: Some(1800),
            token_type: Some("Bearer".to_string()),
            scope: Some("openid offline_access".to_string()),
        };
        let pair = TokenPair::from_response(&response);
        assert_eq!(pair.access_token(), "test_access");
        assert_eq!(pair.refresh_token(), Some("test_refresh"));
        assert!(!pair.is_expired());
        assert!(!pair.needs_refresh());
    }

    #[test]
    fn token_pair_default_lifetime() {
        let response = TokenResponse {
            access_token: "test".to_string(),
            refresh_token: None,
            expires_in: None,
            token_type: None,
            scope: None,
        };
        let pair = TokenPair::from_response(&response);
        // Default lifetime is 30 minutes, minus 5 min margin = 25 min remaining
        assert!(!pair.needs_refresh());
        assert!(pair.time_until_expiry() > Duration::from_secs(25 * 60));
    }

    #[test]
    fn token_pair_debug_redacted() {
        let response = TokenResponse {
            access_token: "secret_token".to_string(),
            refresh_token: Some("secret_refresh".to_string()),
            expires_in: Some(1800),
            token_type: None,
            scope: None,
        };
        let pair = TokenPair::from_response(&response);
        let debug = format!("{pair:?}");
        assert!(!debug.contains("secret_token"));
        assert!(!debug.contains("secret_refresh"));
        assert!(debug.contains("[REDACTED]"));
    }

    #[test]
    fn stored_tokens_round_trip() {
        let response = TokenResponse {
            access_token: "access".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_in: Some(1800),
            token_type: None,
            scope: None,
        };
        let pair = TokenPair::from_response(&response);
        let stored = pair.to_stored();
        assert_eq!(stored.access_token, "access");
        assert_eq!(stored.refresh_token.as_deref(), Some("refresh"));
        assert_eq!(stored.expires_in, 1800);
        assert!(!stored.is_expired());
    }

    #[test]
    fn stored_tokens_expired() {
        let stored = StoredTokens {
            access_token: "old_token".to_string(),
            refresh_token: None,
            expires_in: 1800,
            issued_at: 0, // issued at epoch = definitely expired
        };
        assert!(stored.is_expired());
        assert!(stored.needs_refresh());
    }
}
