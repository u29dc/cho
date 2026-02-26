//! Shared command helpers.

use std::path::{Path, PathBuf};

use cho_sdk::config::SdkConfig;
use cho_sdk::error::{ChoSdkError, Result};
use serde::{Deserialize, Serialize};

/// Max accepted JSON payload file size (50 MB).
const MAX_JSON_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Tool configuration persisted in `config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Auth section.
    #[serde(default)]
    pub auth: AuthConfig,
    /// Defaults section.
    #[serde(default)]
    pub defaults: DefaultsConfig,
    /// SDK section.
    #[serde(default)]
    pub sdk: SdkConfigFile,
    /// Safety section.
    #[serde(default)]
    pub safety: SafetyConfig,
}

/// Auth config.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    /// OAuth client id.
    pub client_id: Option<String>,
    /// OAuth client secret.
    pub client_secret: Option<String>,
}

/// Defaults config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Default output format.
    pub format: Option<String>,
    /// Default list limit.
    pub limit: Option<usize>,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            format: Some("json".to_string()),
            limit: Some(100),
        }
    }
}

/// SDK config persisted in file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SdkConfigFile {
    /// API base URL.
    pub base_url: Option<String>,
    /// Auth URL override.
    pub authorize_url: Option<String>,
    /// Token URL override.
    pub token_url: Option<String>,
    /// Timeout seconds.
    pub timeout_secs: Option<u64>,
    /// Max retries.
    pub max_retries: Option<u32>,
}

/// Safety config.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SafetyConfig {
    /// Explicit write opt-in.
    pub allow_writes: bool,
}

impl AppConfig {
    /// Loads config from disk.
    pub fn load() -> Result<Self> {
        let path = cho_sdk::home::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = std::fs::read_to_string(&path).map_err(|e| ChoSdkError::Config {
            message: format!("Failed reading config {}: {e}", path.display()),
        })?;

        toml::from_str::<Self>(&raw).map_err(|e| ChoSdkError::Config {
            message: format!("Failed parsing config {}: {e}", path.display()),
        })
    }

    /// Saves config to disk.
    pub fn save(&self) -> Result<PathBuf> {
        let path = cho_sdk::home::config_path()?;
        let raw = toml::to_string_pretty(self).map_err(|e| ChoSdkError::Config {
            message: format!("Failed serializing config: {e}"),
        })?;

        std::fs::write(&path, raw).map_err(|e| ChoSdkError::Config {
            message: format!("Failed writing config {}: {e}", path.display()),
        })?;

        Ok(path)
    }

    /// Resolves client id from env > config.
    pub fn resolve_client_id(&self) -> Option<String> {
        std::env::var("CHO_CLIENT_ID")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                self.auth
                    .client_id
                    .clone()
                    .filter(|value| !value.trim().is_empty())
            })
    }

    /// Resolves client secret from env > config.
    pub fn resolve_client_secret(&self) -> Option<String> {
        std::env::var("CHO_CLIENT_SECRET")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                self.auth
                    .client_secret
                    .clone()
                    .filter(|value| !value.trim().is_empty())
            })
    }

    /// Builds runtime SDK config.
    pub fn sdk_config(&self) -> SdkConfig {
        let mut config = SdkConfig::default();

        if let Some(base_url) = std::env::var("CHO_BASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| self.sdk.base_url.clone())
        {
            config = config.with_base_url(base_url);
        }

        if let Some(authorize_url) = self.sdk.authorize_url.clone() {
            config = config.with_authorize_url(authorize_url);
        }

        if let Some(token_url) = self.sdk.token_url.clone() {
            config = config.with_token_url(token_url);
        }

        if let Some(timeout_secs) = self.sdk.timeout_secs {
            config = config.with_timeout_secs(timeout_secs);
        }

        if let Some(max_retries) = self.sdk.max_retries {
            config = config.with_max_retries(max_retries);
        }

        config.with_allow_writes(self.safety.allow_writes)
    }

    /// Sets dotted key to string value.
    pub fn set_key(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "auth.client_id" => self.auth.client_id = Some(value.to_string()),
            "auth.client_secret" => self.auth.client_secret = Some(value.to_string()),
            "defaults.format" => self.defaults.format = Some(value.to_string()),
            "defaults.limit" => {
                let parsed = value.parse::<usize>().map_err(|e| ChoSdkError::Config {
                    message: format!("defaults.limit must be an integer: {e}"),
                })?;
                self.defaults.limit = Some(parsed);
            }
            "sdk.base_url" => self.sdk.base_url = Some(value.to_string()),
            "sdk.authorize_url" => self.sdk.authorize_url = Some(value.to_string()),
            "sdk.token_url" => self.sdk.token_url = Some(value.to_string()),
            "sdk.timeout_secs" => {
                let parsed = value.parse::<u64>().map_err(|e| ChoSdkError::Config {
                    message: format!("sdk.timeout_secs must be an integer: {e}"),
                })?;
                self.sdk.timeout_secs = Some(parsed);
            }
            "sdk.max_retries" => {
                let parsed = value.parse::<u32>().map_err(|e| ChoSdkError::Config {
                    message: format!("sdk.max_retries must be an integer: {e}"),
                })?;
                self.sdk.max_retries = Some(parsed);
            }
            "safety.allow_writes" => {
                let parsed = parse_bool(value)?;
                self.safety.allow_writes = parsed;
            }
            unknown => {
                return Err(ChoSdkError::Config {
                    message: format!("Unsupported config key '{unknown}'"),
                });
            }
        }

        Ok(())
    }

    /// JSON value with secret redaction.
    pub fn as_redacted_json(&self) -> serde_json::Value {
        let mut value = serde_json::to_value(self).unwrap_or_else(|_| serde_json::json!({}));
        if let Some(secret) = value
            .get_mut("auth")
            .and_then(|auth| auth.get_mut("client_secret"))
        {
            *secret = serde_json::Value::String("[REDACTED]".to_string());
        }
        value
    }
}

/// Parses key-value `query` args where each entry is `key=value`.
pub fn parse_query_pairs(entries: &[String]) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();

    for entry in entries {
        let (key, value) = entry.split_once('=').ok_or_else(|| ChoSdkError::Config {
            message: format!("Invalid query argument '{entry}', expected key=value"),
        })?;

        out.push((key.to_string(), value.to_string()));
    }

    Ok(out)
}

/// Reads and parses a JSON payload file.
pub fn read_json_file(path: &Path) -> Result<serde_json::Value> {
    let metadata = std::fs::metadata(path).map_err(|e| ChoSdkError::Config {
        message: format!("Failed reading metadata for {}: {e}", path.display()),
    })?;

    if metadata.len() > MAX_JSON_FILE_SIZE {
        return Err(ChoSdkError::Config {
            message: format!(
                "JSON file {} exceeds maximum size ({} > {})",
                path.display(),
                metadata.len(),
                MAX_JSON_FILE_SIZE
            ),
        });
    }

    let raw = std::fs::read_to_string(path).map_err(|e| ChoSdkError::Config {
        message: format!("Failed reading JSON file {}: {e}", path.display()),
    })?;

    serde_json::from_str::<serde_json::Value>(&raw).map_err(|e| ChoSdkError::Parse {
        message: format!("Failed parsing JSON file {}: {e}", path.display()),
    })
}

/// Converts bool-like strings to bool.
pub fn parse_bool(input: &str) -> Result<bool> {
    match input.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(ChoSdkError::Config {
            message: format!("Expected boolean value, got '{input}'"),
        }),
    }
}
