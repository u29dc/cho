//! CLI error handling with structured error codes, hints, and exit codes.

use std::time::Instant;

use cho_sdk::error::ChoSdkError;

use crate::envelope;

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
    /// Write operations are not allowed.
    WriteNotAllowed,
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
            Self::WriteNotAllowed => "WRITE_NOT_ALLOWED",
            Self::UsageError => "USAGE_ERROR",
        }
    }

    /// Returns an actionable recovery hint for agents.
    pub fn hint(&self) -> &'static str {
        match self {
            Self::AuthRequired => "Run 'cho auth login' to authenticate",
            Self::TokenExpired => "Run 'cho auth login' to re-authenticate",
            Self::RateLimited => "Wait and retry. Use --verbose for rate limit details",
            Self::NotFound => "Verify the resource ID or number",
            Self::ValidationError => "Check the request payload against Xero's API requirements",
            Self::ApiError => "Retry the request. Check 'cho health --json' for system status",
            Self::NetworkError => "Check network connectivity and retry",
            Self::ParseError => "This may indicate an API change. Use --verbose for details",
            Self::WriteNotAllowed => "Set [safety] allow_writes = true in config.toml",
            Self::UsageError => "Run 'cho <command> --help' for usage information",
        }
    }

    /// Returns whether this error blocks further CLI usage.
    ///
    /// Blocking errors return exit code 2; non-blocking return exit code 1.
    pub fn is_blocking(&self) -> bool {
        matches!(
            self,
            Self::AuthRequired | Self::TokenExpired | Self::WriteNotAllowed
        )
    }

    /// Returns the exit code for this error.
    pub fn exit_code(&self) -> i32 {
        if self.is_blocking() { 2 } else { 1 }
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
            ChoSdkError::WriteNotAllowed { .. } => Self::WriteNotAllowed,
        }
    }
}

/// Formats an SDK error for output.
///
/// When `json_mode` is true, outputs a compose-contract error envelope to stdout.
/// Otherwise outputs human-readable text for stderr.
pub fn format_error(err: &ChoSdkError, json_mode: bool, tool: &str, start: Instant) -> String {
    let code = ErrorCode::from(err);

    if json_mode {
        envelope::emit_error(
            tool,
            code.as_str(),
            err.to_string(),
            code.hint().to_string(),
            start,
        )
    } else {
        format!("Error: {err}")
    }
}

/// Returns the exit code for an SDK error.
pub fn exit_code(err: &ChoSdkError) -> i32 {
    ErrorCode::from(err).exit_code()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_hint_nonempty() {
        let codes = [
            ErrorCode::AuthRequired,
            ErrorCode::TokenExpired,
            ErrorCode::RateLimited,
            ErrorCode::NotFound,
            ErrorCode::ValidationError,
            ErrorCode::ApiError,
            ErrorCode::NetworkError,
            ErrorCode::ParseError,
            ErrorCode::WriteNotAllowed,
            ErrorCode::UsageError,
        ];
        for code in &codes {
            assert!(!code.hint().is_empty(), "{:?} has empty hint", code);
        }
    }

    #[test]
    fn blocking_codes_exit_2() {
        assert_eq!(ErrorCode::AuthRequired.exit_code(), 2);
        assert_eq!(ErrorCode::TokenExpired.exit_code(), 2);
        assert_eq!(ErrorCode::WriteNotAllowed.exit_code(), 2);
    }

    #[test]
    fn non_blocking_codes_exit_1() {
        assert_eq!(ErrorCode::RateLimited.exit_code(), 1);
        assert_eq!(ErrorCode::NotFound.exit_code(), 1);
        assert_eq!(ErrorCode::ApiError.exit_code(), 1);
        assert_eq!(ErrorCode::NetworkError.exit_code(), 1);
        assert_eq!(ErrorCode::ParseError.exit_code(), 1);
        assert_eq!(ErrorCode::UsageError.exit_code(), 1);
    }

    #[test]
    fn format_error_json_envelope() {
        let err = ChoSdkError::AuthRequired {
            message: "no token".to_string(),
        };
        let start = Instant::now();
        let json = format_error(&err, true, "invoices.list", start);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["ok"], false);
        assert_eq!(v["error"]["code"], "AUTH_REQUIRED");
        assert!(
            v["error"]["hint"]
                .as_str()
                .unwrap()
                .contains("cho auth login")
        );
        assert_eq!(v["meta"]["tool"], "invoices.list");
    }

    #[test]
    fn format_error_human() {
        let err = ChoSdkError::NotFound {
            resource: "Invoice".to_string(),
            id: "123".to_string(),
        };
        let start = Instant::now();
        let msg = format_error(&err, false, "invoices.get", start);
        assert!(msg.starts_with("Error:"));
        assert!(!msg.contains("\"ok\""));
    }
}
