//! Config commands: set, show.

use std::time::Instant;

use clap::Subcommand;

use crate::context::CliContext;

/// Config subcommands.
#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Set a configuration value.
    Set {
        /// Configuration key (e.g., "auth.tenant_id", "defaults.format").
        key: String,
        /// Value to set.
        value: String,
    },
    /// Show current configuration.
    Show,
}

/// Returns the tool name for a config subcommand.
pub fn tool_name(cmd: &ConfigCommands) -> &'static str {
    match cmd {
        ConfigCommands::Set { .. } => "config.set",
        ConfigCommands::Show => "config.show",
    }
}

/// Runs a config subcommand.
pub async fn run(
    cmd: &ConfigCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        ConfigCommands::Set { key, value } => {
            let config_path = cho_sdk::auth::storage::config_dir()?.join("config.toml");

            let mut config: toml::Table = if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).map_err(|e| {
                    cho_sdk::error::ChoSdkError::Config {
                        message: format!("Failed to read config: {e}"),
                    }
                })?;
                content
                    .parse()
                    .map_err(|e| cho_sdk::error::ChoSdkError::Config {
                        message: format!("Failed to parse config: {e}"),
                    })?
            } else {
                toml::Table::new()
            };

            // Parse key as "section.key" format
            let parts: Vec<&str> = key.splitn(2, '.').collect();
            if parts.len() == 2 {
                let section = config
                    .entry(parts[0].to_string())
                    .or_insert_with(|| toml::Value::Table(toml::Table::new()));
                if let toml::Value::Table(table) = section {
                    table.insert(parts[1].to_string(), toml::Value::String(value.clone()));
                }
            } else {
                config.insert(key.clone(), toml::Value::String(value.clone()));
            }

            let output = toml::to_string_pretty(&config).map_err(|e| {
                cho_sdk::error::ChoSdkError::Config {
                    message: format!("Failed to serialize config: {e}"),
                }
            })?;

            std::fs::write(&config_path, output).map_err(|e| {
                cho_sdk::error::ChoSdkError::Config {
                    message: format!("Failed to write config: {e}"),
                }
            })?;

            eprintln!("Set {key} = {value}");
            ctx.emit_success(
                "config.set",
                &serde_json::json!({"key": key, "value": value}),
                start,
            )?;
            Ok(())
        }
        ConfigCommands::Show => {
            let config_path = cho_sdk::auth::storage::config_dir()?.join("config.toml");

            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).map_err(|e| {
                    cho_sdk::error::ChoSdkError::Config {
                        message: format!("Failed to read config: {e}"),
                    }
                })?;
                let config_value: serde_json::Value =
                    toml::from_str(&content).map_err(|e| cho_sdk::error::ChoSdkError::Config {
                        message: format!("Failed to parse config: {e}"),
                    })?;
                ctx.emit_success("config.show", &config_value, start)?;
            } else {
                eprintln!("No configuration file found.");
                eprintln!("Config path: {}", config_path.display());
                ctx.emit_success("config.show", &serde_json::json!({}), start)?;
            }
            Ok(())
        }
    }
}
