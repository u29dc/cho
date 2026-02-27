//! Runtime configuration loading for `cho-tui`.

use cho_sdk::config::SdkConfig;
use cho_sdk::error::{ChoSdkError, Result};
use serde::{Deserialize, Serialize};

/// Persisted config structure.
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
    /// Default list limit.
    pub limit: Option<usize>,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self { limit: Some(100) }
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

    /// Converts config to redacted JSON.
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
