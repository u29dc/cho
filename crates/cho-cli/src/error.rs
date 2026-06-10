//! CLI error mapping and exit codes.

use std::time::Instant;

use cho_sdk::error::ChoSdkError;

use crate::envelope::{self, OutputFormat};

/// Stable CLI error codes.
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    /// Authentication required.
    AuthRequired,
    /// Token expired and refresh failed.
    TokenExpired,
    /// Rate limited.
    RateLimited,
    /// Resource not found.
    NotFound,
    /// Validation/business error.
    ValidationError,
    /// Generic API failure.
    ApiError,
    /// Network failure.
    NetworkError,
    /// Parse failure.
    ParseError,
    /// Configuration issue.
    ConfigError,
    /// Writes disabled.
    WriteNotAllowed,
    /// Usage issue.
    UsageError,
    /// Audit log unavailable for required safety guarantees.
    AuditLogUnavailable,
}

impl ErrorCode {
    /// Code string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AuthRequired => "auth_required",
            Self::TokenExpired => "token_expired",
            Self::RateLimited => "rate_limited",
            Self::NotFound => "not_found",
            Self::ValidationError => "validation_error",
            Self::ApiError => "api_error",
            Self::NetworkError => "network_error",
            Self::ParseError => "parse_error",
            Self::ConfigError => "config_error",
            Self::WriteNotAllowed => "write_not_allowed",
            Self::UsageError => "usage_error",
            Self::AuditLogUnavailable => "audit_log_unavailable",
        }
    }

    /// Actionable hint.
    pub fn hint(self) -> &'static str {
        match self {
            Self::AuthRequired => "Run 'cho auth login' to authenticate",
            Self::TokenExpired => "Run 'cho auth login' to re-authenticate",
            Self::RateLimited => "Wait and retry using error.details.retryAfter when provided",
            Self::NotFound => "Verify the resource identifier/path",
            Self::ValidationError => "Check request payload fields and values",
            Self::ApiError => "Retry once and inspect FreeAgent API response details",
            Self::NetworkError => "Check network connectivity and retry",
            Self::ParseError => "Use --verbose and inspect raw response data",
            Self::ConfigError => "Run 'cho health' and fix reported checks",
            Self::WriteNotAllowed => "Set [safety] allow_writes = true in config.toml",
            Self::UsageError => "Run command with --help for valid arguments",
            Self::AuditLogUnavailable => {
                "Ensure ~/.tools/cho/history.log is writable before running mutating commands"
            }
        }
    }

    /// Exit code.
    pub fn exit_code(self) -> i32 {
        if matches!(
            self,
            Self::AuthRequired
                | Self::TokenExpired
                | Self::WriteNotAllowed
                | Self::AuditLogUnavailable
        ) {
            2
        } else {
            1
        }
    }
}

impl From<&ChoSdkError> for ErrorCode {
    fn from(value: &ChoSdkError) -> Self {
        match value {
            ChoSdkError::AuthRequired { .. } => Self::AuthRequired,
            ChoSdkError::TokenExpired { .. } => Self::TokenExpired,
            ChoSdkError::RateLimited { .. } => Self::RateLimited,
            ChoSdkError::NotFound { .. } => Self::NotFound,
            ChoSdkError::ApiError { status, .. } if *status == 400 || *status == 422 => {
                Self::ValidationError
            }
            ChoSdkError::ApiError { .. } => Self::ApiError,
            ChoSdkError::Network(_) => Self::NetworkError,
            ChoSdkError::Parse { .. } => Self::ParseError,
            ChoSdkError::WriteNotAllowed { .. } => Self::WriteNotAllowed,
            ChoSdkError::Config { message } if looks_like_usage_error(message) => Self::UsageError,
            ChoSdkError::Config { message } if message.contains("AUDIT_LOG_UNAVAILABLE") => {
                Self::AuditLogUnavailable
            }
            ChoSdkError::Config { .. } => Self::ConfigError,
        }
    }
}

/// Formats an error for current output mode.
pub fn format_error(
    err: &ChoSdkError,
    output_format: OutputFormat,
    tool: &str,
    start: Instant,
) -> String {
    let code = ErrorCode::from(err);
    let details = match err {
        ChoSdkError::RateLimited { retry_after } => {
            Some(serde_json::json!({ "retryAfter": retry_after }))
        }
        _ => None,
    };

    envelope::emit_error(
        tool,
        code.as_str(),
        err.to_string(),
        code.hint().to_string(),
        details,
        start,
        output_format,
    )
}

/// Exit code for SDK error.
pub fn exit_code(err: &ChoSdkError) -> i32 {
    ErrorCode::from(err).exit_code()
}

fn looks_like_usage_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.starts_with("invalid ") || lower.contains("usage") || lower.contains("unknown option")
}
