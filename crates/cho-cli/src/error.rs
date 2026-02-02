//! CLI error handling with structured error codes and exit codes.

use cho_sdk::error::ChoSdkError;

/// CLI error codes for structured agent-consumable output.
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    /// No valid token, login needed.
    AuthRequired,
    /// Refresh failed, re-login needed.
    TokenExpired,
    /// Retry after N seconds.
    RateLimited,
    /// Resource does not exist.
    NotFound,
    /// Xero rejected the request.
    ValidationError,
    /// Server error (5xx).
    ApiError,
    /// Connection/timeout failure.
    NetworkError,
    /// Response deserialization failed.
    ParseError,
    /// Invalid arguments/flags.
    UsageError,
}

impl ErrorCode {
    /// Returns the string code for JSON error output.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AuthRequired => "AUTH_REQUIRED",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::RateLimited => "RATE_LIMITED",
            Self::NotFound => "NOT_FOUND",
            Self::ValidationError => "VALIDATION_ERROR",
            Self::ApiError => "API_ERROR",
            Self::NetworkError => "NETWORK_ERROR",
            Self::ParseError => "PARSE_ERROR",
            Self::UsageError => "USAGE_ERROR",
        }
    }

    /// Returns the exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::AuthRequired | Self::TokenExpired => 2,
            Self::UsageError => 3,
            _ => 1,
        }
    }
}

impl From<&ChoSdkError> for ErrorCode {
    fn from(err: &ChoSdkError) -> Self {
        match err {
            ChoSdkError::AuthRequired { .. } => Self::AuthRequired,
            ChoSdkError::TokenExpired { .. } => Self::TokenExpired,
            ChoSdkError::RateLimited { .. } => Self::RateLimited,
            ChoSdkError::NotFound { .. } => Self::NotFound,
            ChoSdkError::ApiError {
                validation_errors, ..
            } if !validation_errors.is_empty() => Self::ValidationError,
            ChoSdkError::ApiError { .. } => Self::ApiError,
            ChoSdkError::Network(_) => Self::NetworkError,
            ChoSdkError::Parse { .. } => Self::ParseError,
            ChoSdkError::Config { .. } => Self::UsageError,
        }
    }
}

/// Formats an SDK error for output.
///
/// When `json_errors` is true, outputs structured JSON to stderr.
/// Otherwise outputs human-readable text.
pub fn format_error(err: &ChoSdkError, json_errors: bool) -> String {
    let code = ErrorCode::from(err);

    if json_errors {
        let json = serde_json::json!({
            "error": err.to_string(),
            "code": code.as_str(),
        });
        serde_json::to_string(&json).unwrap_or_else(|_| err.to_string())
    } else {
        format!("Error: {err}")
    }
}

/// Returns the exit code for an SDK error.
pub fn exit_code(err: &ChoSdkError) -> i32 {
    ErrorCode::from(err).exit_code()
}
