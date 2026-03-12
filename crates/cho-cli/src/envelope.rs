//! JSON envelope contract.

use std::time::Instant;

use serde::Serialize;

/// Success envelope.
#[derive(Serialize)]
pub struct SuccessEnvelope<T: Serialize> {
    /// Always true.
    pub ok: bool,
    /// Response data.
    pub data: T,
    /// Metadata.
    pub meta: Meta,
}

/// Error envelope.
#[derive(Serialize)]
pub struct ErrorEnvelope {
    /// Always false.
    pub ok: bool,
    /// Error details.
    pub error: EnvelopeError,
    /// Metadata.
    pub meta: Meta,
}

/// Error payload.
#[derive(Serialize)]
pub struct EnvelopeError {
    /// Stable error code.
    pub code: &'static str,
    /// Human-readable message.
    pub message: String,
    /// Actionable hint.
    pub hint: String,
    /// Optional structured details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Envelope metadata.
#[derive(Serialize)]
pub struct Meta {
    /// Tool name (`group.action`).
    pub tool: String,
    /// Elapsed duration in ms.
    pub elapsed: u64,
    /// Optional item count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    /// Optional total count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
    /// Optional has-more flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hasMore")]
    pub has_more: Option<bool>,
}

/// Emits compact success JSON.
pub fn emit_success<T: Serialize>(
    tool: &str,
    data: T,
    start: Instant,
    count: Option<usize>,
    total: Option<usize>,
    has_more: Option<bool>,
) -> String {
    let envelope = SuccessEnvelope {
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

    serde_json::to_string(&envelope).unwrap_or_else(|err| {
        format!(
            "{{\"ok\":false,\"error\":{{\"code\":\"INTERNAL\",\"message\":\"{}\",\"hint\":\"Report this bug\"}},\"meta\":{{\"tool\":\"{}\",\"elapsed\":0}}}}",
            escape_json(&format!("Serialization failed: {err}")),
            escape_json(tool)
        )
    })
}

/// Emits compact error JSON.
pub fn emit_error(
    tool: &str,
    code: &'static str,
    message: String,
    hint: String,
    details: Option<serde_json::Value>,
    start: Instant,
) -> String {
    let envelope = ErrorEnvelope {
        ok: false,
        error: EnvelopeError {
            code,
            message,
            hint,
            details,
        },
        meta: Meta {
            tool: tool.to_string(),
            elapsed: start.elapsed().as_millis() as u64,
            count: None,
            total: None,
            has_more: None,
        },
    };

    serde_json::to_string(&envelope).unwrap_or_else(|err| {
        format!(
            "{{\"ok\":false,\"error\":{{\"code\":\"INTERNAL\",\"message\":\"{}\",\"hint\":\"Report this bug\"}},\"meta\":{{\"tool\":\"{}\",\"elapsed\":0}}}}",
            escape_json(&format!("Serialization failed: {err}")),
            escape_json(tool)
        )
    })
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
