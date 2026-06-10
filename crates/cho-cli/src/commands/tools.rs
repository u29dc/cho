//! Tool catalog command.

use std::time::Instant;

use cho_sdk::error::Result;
use serde::Serialize;

use crate::audit::AuditLogger;
use crate::envelope::{self, OutputFormat};
use crate::registry::{GLOBAL_FLAGS, tool_catalog};

#[derive(Serialize)]
struct ToolsPayload {
    version: &'static str,
    #[serde(rename = "outputFormats")]
    output_formats: &'static [&'static str],
    #[serde(rename = "defaultOutputFormat")]
    default_output_format: &'static str,
    #[serde(rename = "globalFlags")]
    global_flags: &'static [crate::registry::GlobalFlagMeta],
    tools: Vec<crate::registry::ToolMeta>,
}

/// Runs `tools` command.
pub fn run(
    name: Option<&str>,
    output_format: OutputFormat,
    start: Instant,
    audit: &AuditLogger,
) -> Result<i32> {
    let tools = tool_catalog();

    if let Some(name) = name {
        if let Some(tool) = tools.iter().find(|tool| tool.name == name) {
            let output =
                envelope::emit_success("tools.get", tool, start, None, None, None, output_format);
            audit.log_command_output("tools.get", &output)?;
            envelope::write_stdout(&output);
            return Ok(0);
        }

        let output = envelope::emit_error(
            "tools.get",
            "not_found",
            format!("Tool '{name}' was not found"),
            "Run 'cho tools' to inspect available tools".to_string(),
            None,
            start,
            output_format,
        );
        audit.log_command_output("tools.get", &output)?;
        envelope::write_stdout(&output);
        return Ok(1);
    }

    let payload = ToolsPayload {
        version: env!("CARGO_PKG_VERSION"),
        output_formats: &["json", "toon"],
        default_output_format: "json",
        global_flags: GLOBAL_FLAGS,
        tools,
    };
    let output = envelope::emit_success(
        "tools.list",
        payload,
        start,
        None,
        None,
        None,
        output_format,
    );

    audit.log_command_output("tools.list", &output)?;
    envelope::write_stdout(&output);
    Ok(0)
}
