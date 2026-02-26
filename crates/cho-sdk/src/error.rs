//! SDK error types.

use std::fmt;

/// Convenience result alias.
pub type Result<T> = std::result::Result<T, ChoSdkError>;

/// Structured SDK error type.
#[derive(Debug, thiserror::Error)]
pub enum ChoSdkError {
    /// Missing/invalid auth credentials or tokens.
    #[error("authentication required: {message}")]
    AuthRequired {
        /// Human-readable detail.
        message: String,
    },

    /// Token refresh failed and user must re-authenticate.
    #[error("token expired: {message}")]
    TokenExpired {
        /// Human-readable detail.
        message: String,
    },

    /// API request was rate limited.
    #[error("rate limited, retry after {retry_after} seconds")]
    RateLimited {
        /// Retry delay in seconds.
        retry_after: u64,
    },

    /// API returned a non-success response.
    #[error("api error {status}: {message}")]
    ApiError {
        /// HTTP status code.
        status: u16,
        /// Error message/response text.
        message: String,
    },

    /// Requested resource does not exist.
    #[error("{resource} not found: {id}")]
    NotFound {
        /// Resource type/path.
        resource: String,
        /// Identifier.
        id: String,
    },

    /// Network transport error.
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    /// Invalid response parsing/shape.
    #[error("parse error: {message}")]
    Parse {
        /// Human-readable detail.
        message: String,
    },

    /// Local configuration failure.
    #[error("config error: {message}")]
    Config {
        /// Human-readable detail.
        message: String,
    },

    /// Write operations are disabled.
    #[error("write operations not allowed: {message}")]
    WriteNotAllowed {
        /// Human-readable detail.
        message: String,
    },
}

impl ChoSdkError {
    /// Converts an API error response into [`Self::ApiError`].
    pub fn api(status: reqwest::StatusCode, body: impl fmt::Display) -> Self {
        Self::ApiError {
            status: status.as_u16(),
            message: body.to_string(),
        }
    }
}
