//! Tool catalog command.

use std::time::Instant;

use crate::audit::AuditLogger;
use crate::envelope;
use crate::registry::{GLOBAL_FLAGS, tool_catalog};

/// Runs `tools` command.
pub fn run(name: Option<&str>, json_mode: bool, start: Instant, audit: &AuditLogger) {
    let tools = tool_catalog();

    if let Some(name) = name {
        if let Some(tool) = tools.iter().find(|tool| tool.name == name) {
            if json_mode {
                let output = envelope::emit_success("tools.get", tool, start, None, None, None);
                println!("{output}");
                let _ = audit.log_command_output("tools.get", &output);
            } else {
                let output = format!(
                    "{}\n  {}\n  command: {}",
                    tool.name, tool.description, tool.command
                );
                eprintln!("{output}");
                let _ = audit.log_command_output("tools.get", &output);
            }
            return;
        }

        if json_mode {
            let output = envelope::emit_error(
                "tools.get",
                "NOT_FOUND",
                format!("Tool '{name}' was not found"),
                "Run 'cho tools --json' to inspect available tools".to_string(),
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

    if json_mode {
        let payload = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "tools": tools,
            "globalFlags": GLOBAL_FLAGS,
        });

        let output = envelope::emit_success("tools.list", payload, start, None, None, None);
        println!("{output}");
        let _ = audit.log_command_output("tools.list", &output);
        return;
    }

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
    eprintln!("{}", output.trim_end());
    let _ = audit.log_command_output("tools.list", output.trim_end());
}
