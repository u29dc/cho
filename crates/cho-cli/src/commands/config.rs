//! Config commands.

use std::time::Instant;

use cho_sdk::error::Result;
use clap::Subcommand;

use crate::audit::AuditLogger;
use crate::envelope;
use crate::output::{OutputFormat, OutputMode, format_value};

use super::utils::AppConfig;

/// Config subcommands.
#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Show current config.
    Show,
    /// Set config key/value.
    Set {
        /// Dotted key.
        key: String,
        /// Value.
        value: String,
    },
}

/// Tool name for subcommand.
pub fn tool_name(command: &ConfigCommands) -> &'static str {
    match command {
        ConfigCommands::Show => "config.show",
        ConfigCommands::Set { .. } => "config.set",
    }
}

/// Runs config subcommand.
pub fn run(
    command: &ConfigCommands,
    output_mode: OutputMode,
    start: Instant,
    audit: &AuditLogger,
) -> Result<()> {
    match command {
        ConfigCommands::Show => {
            let config = AppConfig::load()?;
            let payload = config.as_redacted_json();
            let output = match output_mode {
                OutputMode::Json => {
                    envelope::emit_success("config.show", &payload, start, None, None, None)
                }
                OutputMode::Text => {
                    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                }
                OutputMode::Table => format_value(&payload, OutputFormat::Table),
                OutputMode::Csv => format_value(&payload, OutputFormat::Csv),
            };
            println!("{output}");
            let _ = audit.log_command_output("config.show", &output);
            Ok(())
        }
        ConfigCommands::Set { key, value } => {
            let mut config = AppConfig::load()?;
            config.set_key(key, value)?;
            let path = config.save()?;
            let payload = serde_json::json!({
                "key": key,
                "value": if key == "auth.client_secret" { "[REDACTED]" } else { value },
                "path": path,
            });
            let output = match output_mode {
                OutputMode::Json => {
                    envelope::emit_success("config.set", &payload, start, None, None, None)
                }
                OutputMode::Text => format!(
                    "Set {} = {}",
                    key,
                    if key == "auth.client_secret" {
                        "[REDACTED]"
                    } else {
                        value
                    }
                ),
                OutputMode::Table => format_value(&payload, OutputFormat::Table),
                OutputMode::Csv => format_value(&payload, OutputFormat::Csv),
            };
            println!("{output}");
            let _ = audit.log_command_output("config.set", &output);
            Ok(())
        }
    }
}
