//! SDK configuration.
//!
//! [`SdkConfig`] controls base URL, timeouts, and retry behavior for the HTTP client.

use std::time::Duration;

/// Configuration for the Xero SDK HTTP client.
#[derive(Debug, Clone)]
pub struct SdkConfig {
    /// Base URL for the Xero Accounting API.
    ///
    /// Defaults to `https://api.xero.com/api.xro/2.0/`.
    pub base_url: String,

    /// HTTP request timeout.
    ///
    /// Defaults to 30 seconds.
    pub timeout: Duration,

    /// Maximum number of retries for transient failures.
    ///
    /// Defaults to 3.
    pub max_retries: u32,

    /// Whether write operations (PUT/POST) are allowed.
    ///
    /// Defaults to `false`. Must be explicitly enabled to prevent
    /// accidental mutations against the Xero API.
    pub allow_writes: bool,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.xero.com/api.xro/2.0/".to_owned(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            allow_writes: false,
        }
    }
}

impl SdkConfig {
    /// Create a new [`SdkConfig`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base URL for the Xero API.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the HTTP request timeout in seconds.
    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    /// Set the maximum number of retries.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Enable or disable write operations (PUT/POST).
    ///
    /// Writes are disabled by default to prevent accidental mutations.
    pub fn with_allow_writes(mut self, allow: bool) -> Self {
        self.allow_writes = allow;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = SdkConfig::default();
        assert_eq!(config.base_url, "https://api.xero.com/api.xro/2.0/");
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn builder_pattern() {
        let config = SdkConfig::new()
            .with_base_url("http://localhost:8080/")
            .with_timeout_secs(10)
            .with_max_retries(5);
        assert_eq!(config.base_url, "http://localhost:8080/");
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_retries, 5);
    }
}
