//! OAuth token models and lifecycle helpers.

use chrono::{DateTime, Duration, Utc};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

/// Refresh margin before token expiry.
const REFRESH_MARGIN_SECS: i64 = 120;

/// Token response from FreeAgent token endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    /// Access token.
    pub access_token: String,
    /// Token type, typically `bearer`.
    pub token_type: Option<String>,
    /// Token lifetime in seconds.
    pub expires_in: Option<i64>,
    /// Refresh token.
    pub refresh_token: Option<String>,
    /// Refresh token lifetime in seconds, when returned.
    pub refresh_token_expires_in: Option<i64>,
}

/// Storable token record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTokens {
    /// Access token.
    pub access_token: String,
    /// Refresh token.
    pub refresh_token: Option<String>,
    /// Access token expiry timestamp.
    pub expires_at: DateTime<Utc>,
    /// Refresh token expiry timestamp if provided.
    pub refresh_expires_at: Option<DateTime<Utc>>,
}

/// In-memory token pair.
#[derive(Clone)]
pub struct TokenPair {
    access_token: SecretString,
    refresh_token: Option<SecretString>,
    expires_at: DateTime<Utc>,
    refresh_expires_at: Option<DateTime<Utc>>,
}

impl TokenPair {
    /// Creates from token response.
    pub fn from_response(response: &TokenResponse) -> Self {
        let now = Utc::now();
        let expires_in = response.expires_in.unwrap_or(3600);
        let refresh_expires_at = response
            .refresh_token_expires_in
            .map(|secs| now + Duration::seconds(secs.max(0)));

        Self {
            access_token: SecretString::from(response.access_token.clone()),
            refresh_token: response
                .refresh_token
                .as_ref()
                .map(|value| SecretString::from(value.clone())),
            expires_at: now + Duration::seconds(expires_in.max(1)),
            refresh_expires_at,
        }
    }

    /// Creates from stored tokens.
    pub fn from_stored(stored: &StoredTokens) -> Self {
        Self {
            access_token: SecretString::from(stored.access_token.clone()),
            refresh_token: stored
                .refresh_token
                .as_ref()
                .map(|value| SecretString::from(value.clone())),
            expires_at: stored.expires_at,
            refresh_expires_at: stored.refresh_expires_at,
        }
    }

    /// Converts to storable token record.
    pub fn to_stored(&self) -> StoredTokens {
        StoredTokens {
            access_token: self.access_token.expose_secret().to_string(),
            refresh_token: self
                .refresh_token
                .as_ref()
                .map(|value| value.expose_secret().to_string()),
            expires_at: self.expires_at,
            refresh_expires_at: self.refresh_expires_at,
        }
    }

    /// Returns access token string.
    pub fn access_token(&self) -> &str {
        self.access_token.expose_secret()
    }

    /// Returns refresh token string.
    pub fn refresh_token(&self) -> Option<&str> {
        self.refresh_token
            .as_ref()
            .map(|value| value.expose_secret())
    }

    /// Whether the access token has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// Whether the token should be refreshed soon.
    pub fn needs_refresh(&self) -> bool {
        Utc::now() >= self.expires_at - Duration::seconds(REFRESH_MARGIN_SECS)
    }

    /// Approximate seconds until expiry.
    pub fn expires_in_seconds(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds()
    }

    /// Access token expiry timestamp.
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    /// Returns true if refresh token is still likely valid.
    pub fn can_refresh(&self) -> bool {
        match self.refresh_expires_at {
            Some(expiry) => Utc::now() < expiry,
            None => self.refresh_token.is_some(),
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
            .field("refresh_expires_at", &self.refresh_expires_at)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_pair_from_response_uses_defaults_and_refresh_token() {
        let response = TokenResponse {
            access_token: "access-token".to_string(),
            token_type: Some("bearer".to_string()),
            expires_in: None,
            refresh_token: Some("refresh-token".to_string()),
            refresh_token_expires_in: None,
        };

        let pair = TokenPair::from_response(&response);
        assert_eq!(pair.access_token(), "access-token");
        assert_eq!(pair.refresh_token(), Some("refresh-token"));
        assert!(!pair.is_expired());
        assert!(pair.can_refresh());
    }

    #[test]
    fn token_pair_round_trips_through_stored_representation() {
        let response = TokenResponse {
            access_token: "access-token".to_string(),
            token_type: Some("bearer".to_string()),
            expires_in: Some(3600),
            refresh_token: Some("refresh-token".to_string()),
            refresh_token_expires_in: Some(86_400),
        };

        let original = TokenPair::from_response(&response);
        let stored = original.to_stored();
        let restored = TokenPair::from_stored(&stored);

        assert_eq!(restored.access_token(), "access-token");
        assert_eq!(restored.refresh_token(), Some("refresh-token"));
        assert_eq!(restored.expires_at(), stored.expires_at);
    }

    #[test]
    fn token_pair_needs_refresh_when_close_to_expiry() {
        let stored = StoredTokens {
            access_token: "access-token".to_string(),
            refresh_token: Some("refresh-token".to_string()),
            expires_at: Utc::now() + Duration::seconds(30),
            refresh_expires_at: Some(Utc::now() + Duration::hours(1)),
        };

        let pair = TokenPair::from_stored(&stored);
        assert!(pair.needs_refresh());
    }

    #[test]
    fn token_pair_does_not_need_refresh_when_expiry_is_far_out() {
        let stored = StoredTokens {
            access_token: "access-token".to_string(),
            refresh_token: Some("refresh-token".to_string()),
            expires_at: Utc::now() + Duration::minutes(30),
            refresh_expires_at: Some(Utc::now() + Duration::hours(1)),
        };

        let pair = TokenPair::from_stored(&stored);
        assert!(!pair.needs_refresh());
    }

    #[test]
    fn token_pair_cannot_refresh_after_refresh_expiry() {
        let stored = StoredTokens {
            access_token: "access-token".to_string(),
            refresh_token: Some("refresh-token".to_string()),
            expires_at: Utc::now() + Duration::minutes(30),
            refresh_expires_at: Some(Utc::now() - Duration::seconds(1)),
        };

        let pair = TokenPair::from_stored(&stored);
        assert!(!pair.can_refresh());
    }
}
