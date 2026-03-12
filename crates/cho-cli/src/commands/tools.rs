//! Tool catalog command.

use std::time::Instant;

use serde::Serialize;

use crate::audit::AuditLogger;
use crate::envelope;
use crate::output::{OutputFormat, OutputMode, format_value};
use crate::registry::{GLOBAL_FLAGS, tool_catalog};

#[derive(Serialize)]
struct ToolsPayload {
    version: &'static str,
    #[serde(rename = "globalFlags")]
    global_flags: &'static [crate::registry::GlobalFlagMeta],
    tools: Vec<crate::registry::ToolMeta>,
}

/// Runs `tools` command.
pub fn run(name: Option<&str>, output_mode: OutputMode, start: Instant, audit: &AuditLogger) {
    let tools = tool_catalog();

    if let Some(name) = name {
        if let Some(tool) = tools.iter().find(|tool| tool.name == name) {
            let output = match output_mode {
                OutputMode::Json => {
                    envelope::emit_success("tools.get", tool, start, None, None, None)
                }
                OutputMode::Text => format!(
                    "{}\n  {}\n  command: {}",
                    tool.name, tool.description, tool.command
                ),
                OutputMode::Table => format_value(
                    &serde_json::to_value(tool).expect("tool metadata should serialize"),
                    OutputFormat::Table,
                ),
                OutputMode::Csv => format_value(
                    &serde_json::to_value(tool).expect("tool metadata should serialize"),
                    OutputFormat::Csv,
                ),
            };
            println!("{output}");
            let _ = audit.log_command_output("tools.get", &output);
            return;
        }

        if output_mode.is_json() {
            let output = envelope::emit_error(
                "tools.get",
                "NOT_FOUND",
                format!("Tool '{name}' was not found"),
                "Run 'cho tools' to inspect available tools".to_string(),
                None,
                start,
            );
            println!("{output}");
            let _ = audit.log_command_output("tools.get", &output);
            std::process::exit(1);
        }

        let output = format!("Tool '{name}' not found.");
        eprintln!("{output}");
        let _ = audit.log_command_output("tools.get", &output);
        std::process::exit(1);
    }

    let output = match output_mode {
        OutputMode::Json => {
            let payload = ToolsPayload {
                version: env!("CARGO_PKG_VERSION"),
                global_flags: GLOBAL_FLAGS,
                tools,
            };
            envelope::emit_success("tools.list", payload, start, None, None, None)
        }
        OutputMode::Text => {
            let mut current = String::new();
            let mut output = String::new();
            for tool in &tools {
                if tool.category != current {
                    if !current.is_empty() {
                        output.push('\n');
                    }
                    current = tool.category.clone();
                    output.push_str(&format!("{}\n", current.to_uppercase()));
                }
                output.push_str(&format!("  {:<44} {}\n", tool.name, tool.description));
            }
            output.trim_end().to_string()
        }
        OutputMode::Table => format_value(
            &serde_json::to_value(&tools).expect("tool catalog should serialize"),
            OutputFormat::Table,
        ),
        OutputMode::Csv => format_value(
            &serde_json::to_value(&tools).expect("tool catalog should serialize"),
            OutputFormat::Csv,
        ),
    };

    println!("{output}");
    let _ = audit.log_command_output("tools.list", &output);
}
