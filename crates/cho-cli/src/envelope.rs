//! Structured stdout envelope contract.

use std::time::Instant;

use serde::Serialize;

/// Structured stdout format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Compact JSON envelope.
    Json,
    /// Toon envelope.
    Toon,
}

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

/// Renders a success envelope.
pub fn emit_success<T: Serialize>(
    tool: &str,
    data: T,
    start: Instant,
    count: Option<usize>,
    total: Option<usize>,
    has_more: Option<bool>,
    format: OutputFormat,
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

    render(&envelope, format).unwrap_or_else(|err| fallback_error(tool, &err))
}

/// Renders an error envelope.
pub fn emit_error(
    tool: &str,
    code: &'static str,
    message: String,
    hint: String,
    details: Option<serde_json::Value>,
    start: Instant,
    format: OutputFormat,
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

    render(&envelope, format).unwrap_or_else(|err| fallback_error(tool, &err))
}

/// Writes one structured envelope payload to stdout.
pub fn write_stdout(output: &str) {
    println!("{output}");
}

fn render<T: Serialize>(value: &T, format: OutputFormat) -> Result<String, String> {
    match format {
        OutputFormat::Json => serde_json::to_string(value).map_err(|err| err.to_string()),
        OutputFormat::Toon => toon_format::encode_default(value).map_err(|err| err.to_string()),
    }
}

fn fallback_error(tool: &str, error: &str) -> String {
    format!(
        "{{\"ok\":false,\"error\":{{\"code\":\"internal_error\",\"message\":\"{}\",\"hint\":\"Report this bug\"}},\"meta\":{{\"tool\":\"{}\",\"elapsed\":0}}}}",
        escape_json(&format!("Serialization failed: {error}")),
        escape_json(tool)
    )
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
