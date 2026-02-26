//! SDK runtime configuration.

use std::time::Duration;

/// Runtime SDK configuration.
#[derive(Debug, Clone)]
pub struct SdkConfig {
    /// FreeAgent API base URL.
    pub base_url: String,
    /// OAuth authorize endpoint.
    pub authorize_url: String,
    /// OAuth token endpoint.
    pub token_url: String,
    /// Request timeout.
    pub timeout: Duration,
    /// Maximum retries for transient failures.
    pub max_retries: u32,
    /// Whether mutating operations are allowed.
    pub allow_writes: bool,
    /// User-Agent header value.
    pub user_agent: String,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.freeagent.com/v2/".to_string(),
            authorize_url: "https://api.freeagent.com/v2/approve_app".to_string(),
            token_url: "https://api.freeagent.com/v2/token_endpoint".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            allow_writes: false,
            user_agent: format!("cho/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl SdkConfig {
    /// Sets API base URL.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Sets authorize URL.
    pub fn with_authorize_url(mut self, url: impl Into<String>) -> Self {
        self.authorize_url = url.into();
        self
    }

    /// Sets token URL.
    pub fn with_token_url(mut self, url: impl Into<String>) -> Self {
        self.token_url = url.into();
        self
    }

    /// Sets timeout seconds.
    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    /// Sets max retries.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Enables/disables mutating calls.
    pub fn with_allow_writes(mut self, allow: bool) -> Self {
        self.allow_writes = allow;
        self
    }

    /// Sets user agent.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Returns true when base/token/auth URLs are all http or https.
    pub fn is_valid_url_scheme(&self) -> bool {
        [
            self.base_url.as_str(),
            self.authorize_url.as_str(),
            self.token_url.as_str(),
        ]
        .iter()
        .all(|u| u.starts_with("https://") || u.starts_with("http://"))
    }
}
