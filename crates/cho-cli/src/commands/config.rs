//! Config commands: set, show.

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

/// Runs a config subcommand.
pub async fn run(cmd: &ConfigCommands, _ctx: &CliContext) -> cho_sdk::error::Result<()> {
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
                println!("{content}");
            } else {
                eprintln!("No configuration file found.");
                eprintln!("Config path: {}", config_path.display());
            }
            Ok(())
        }
    }
}
