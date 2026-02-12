//! Agent-native JSON envelope contract.
//!
//! All JSON output goes through this module. Success responses use
//! `{ok: true, data, meta}`, errors use `{ok: false, error, meta}`.
//! Table/CSV output bypasses the envelope entirely.

use std::time::Instant;

use serde::Serialize;

/// Success envelope: `{ok: true, data: T, meta: {...}}`.
#[derive(Serialize)]
pub struct Envelope<T: Serialize> {
    /// Always `true` for success responses.
    pub ok: bool,
    /// The response payload.
    pub data: T,
    /// Request metadata.
    pub meta: Meta,
}

/// Error envelope: `{ok: false, error: {...}, meta: {...}}`.
#[derive(Serialize)]
pub struct ErrorEnvelope {
    /// Always `false` for error responses.
    pub ok: bool,
    /// Structured error information.
    pub error: EnvelopeError,
    /// Request metadata.
    pub meta: Meta,
}

/// Structured error within the envelope.
#[derive(Serialize)]
pub struct EnvelopeError {
    /// Machine-readable error code (e.g., "AUTH_REQUIRED").
    pub code: &'static str,
    /// Human-readable error description.
    pub message: String,
    /// Actionable recovery suggestion for agents.
    pub hint: String,
}

/// Request metadata attached to every envelope.
#[derive(Serialize)]
pub struct Meta {
    /// Tool name in `category.action` format (e.g., "invoices.list").
    pub tool: String,
    /// Request duration in milliseconds.
    pub elapsed: u64,
    /// Number of items in the response (for list operations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    /// Total items available (from pagination).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
    /// Whether more pages are available.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hasMore")]
    pub has_more: Option<bool>,
}

/// Builds a success envelope JSON string.
pub fn emit_success<T: Serialize>(
    tool: &str,
    data: T,
    start: Instant,
    count: Option<usize>,
    total: Option<usize>,
    has_more: Option<bool>,
) -> String {
    let envelope = Envelope {
        ok: true,
        data,
        meta: Meta {
            tool: tool.to_string(),
            elapsed: start.elapsed().as_millis() as u64,
            count,
            total,
            has_more,
        },
    };
    // Safe: envelope is always serializable
    serde_json::to_string_pretty(&envelope).unwrap_or_else(|e| {
        format!(
            r#"{{"ok":false,"error":{{"code":"INTERNAL","message":"Serialization failed: {e}","hint":"Report this bug"}},"meta":{{"tool":"{tool}","elapsed":0}}}}"#
        )
    })
}

/// Builds an error envelope JSON string.
pub fn emit_error(
    tool: &str,
    code: &'static str,
    message: String,
    hint: String,
    start: Instant,
) -> String {
    let envelope = ErrorEnvelope {
        ok: false,
        error: EnvelopeError {
            code,
            message,
            hint,
        },
        meta: Meta {
            tool: tool.to_string(),
            elapsed: start.elapsed().as_millis() as u64,
            count: None,
            total: None,
            has_more: None,
        },
    };
    serde_json::to_string_pretty(&envelope).unwrap_or_else(|e| {
        format!(
            r#"{{"ok":false,"error":{{"code":"INTERNAL","message":"Serialization failed: {e}","hint":"Report this bug"}},"meta":{{"tool":"{tool}","elapsed":0}}}}"#
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_envelope_structure() {
        let start = Instant::now();
        let json = emit_success(
            "invoices.list",
            serde_json::json!([{"id": "123"}]),
            start,
            Some(1),
            Some(50),
            Some(true),
        );
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["ok"], true);
        assert!(v["data"].is_array());
        assert_eq!(v["meta"]["tool"], "invoices.list");
        assert_eq!(v["meta"]["count"], 1);
        assert_eq!(v["meta"]["total"], 50);
        assert_eq!(v["meta"]["hasMore"], true);
        assert!(v["meta"]["elapsed"].is_u64());
    }

    #[test]
    fn error_envelope_structure() {
        let start = Instant::now();
        let json = emit_error(
            "invoices.list",
            "AUTH_REQUIRED",
            "No token".into(),
            "Run cho auth login".into(),
            start,
        );
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["ok"], false);
        assert_eq!(v["error"]["code"], "AUTH_REQUIRED");
        assert_eq!(v["error"]["message"], "No token");
        assert_eq!(v["error"]["hint"], "Run cho auth login");
        assert_eq!(v["meta"]["tool"], "invoices.list");
    }

    #[test]
    fn success_envelope_omits_none_fields() {
        let start = Instant::now();
        let json = emit_success(
            "auth.status",
            serde_json::json!({"ok": true}),
            start,
            None,
            None,
            None,
        );
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v["meta"].get("count").is_none());
        assert!(v["meta"].get("total").is_none());
        assert!(v["meta"].get("hasMore").is_none());
    }
}
