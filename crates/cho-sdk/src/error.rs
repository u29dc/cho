//! SDK error types.
//!
//! [`ChoSdkError`] is the primary error enum returned by all SDK operations.

/// Primary error type for all cho-sdk operations.
#[derive(Debug, thiserror::Error)]
pub enum ChoSdkError {
    /// No valid authentication token is available. User must log in.
    #[error("authentication required: {message}")]
    AuthRequired {
        /// Human-readable explanation of why auth is required.
        message: String,
    },

    /// Access token has expired and refresh failed or returned 401.
    #[error("token expired: {message}")]
    TokenExpired {
        /// Human-readable explanation of the token expiry.
        message: String,
    },

    /// Xero API returned HTTP 429 (rate limited).
    #[error("rate limited, retry after {retry_after} seconds")]
    RateLimited {
        /// Number of seconds to wait before retrying.
        retry_after: u64,
    },

    /// Xero API returned an error response (4xx/5xx, excluding 401 and 429).
    #[error("API error {status}: {message}")]
    ApiError {
        /// HTTP status code.
        status: u16,
        /// Error message from the API response.
        message: String,
        /// Validation error details, if any.
        validation_errors: Vec<String>,
    },

    /// Xero API returned 404 for the requested resource.
    #[error("{resource} not found: {id}")]
    NotFound {
        /// The type of resource that was not found (e.g., "Invoice").
        resource: String,
        /// The ID or identifier that was looked up.
        id: String,
    },

    /// Network or connection error from the HTTP client.
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    /// Failed to parse/deserialize an API response.
    #[error("parse error: {message}")]
    Parse {
        /// Description of what failed to parse.
        message: String,
    },

    /// Configuration or keychain error.
    #[error("config error: {message}")]
    Config {
        /// Description of the configuration problem.
        message: String,
    },
}

/// Convenience type alias for SDK results.
pub type Result<T> = std::result::Result<T, ChoSdkError>;
