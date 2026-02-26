//! Config commands.

use std::time::Instant;

use cho_sdk::error::Result;
use clap::Subcommand;

use crate::audit::AuditLogger;
use crate::envelope;

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
    json_mode: bool,
    start: Instant,
    audit: &AuditLogger,
) -> Result<()> {
    match command {
        ConfigCommands::Show => {
            let config = AppConfig::load()?;
            let payload = config.as_redacted_json();
            if json_mode {
                let output =
                    envelope::emit_success("config.show", payload, start, None, None, None);
                println!("{output}");
                let _ = audit.log_command_output("config.show", &output);
            } else {
                let output =
                    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
                eprintln!("{output}");
                let _ = audit.log_command_output("config.show", &output);
            }
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
            if json_mode {
                let output = envelope::emit_success("config.set", payload, start, None, None, None);
                println!("{output}");
                let _ = audit.log_command_output("config.set", &output);
            } else {
                let output = format!(
                    "Set {} = {}",
                    key,
                    if key == "auth.client_secret" {
                        "[REDACTED]"
                    } else {
                        value
                    }
                );
                eprintln!("{output}");
                let _ = audit.log_command_output("config.set", &output);
            }
            Ok(())
        }
    }
}
