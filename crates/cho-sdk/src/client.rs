//! FreeAgent API client.

use std::sync::Arc;
use std::time::Instant;

use serde_json::Value;
use tracing::{debug, warn};

use crate::api::resource::ResourceApi;
use crate::api::specs::ResourceSpec;
use crate::auth::AuthManager;
use crate::config::SdkConfig;
use crate::error::{ChoSdkError, Result};
use crate::models::{ListResult, Pagination};

/// Observer for low-level HTTP events.
pub trait HttpObserver: Send + Sync {
    /// Called before a request is sent.
    fn on_request(&self, event: &HttpRequestEvent);
    /// Called after a response is received (or request fails).
    fn on_response(&self, event: &HttpResponseEvent);
}

/// HTTP request event.
#[derive(Debug, Clone)]
pub struct HttpRequestEvent {
    /// HTTP method.
    pub method: String,
    /// Full URL.
    pub url: String,
    /// Query parameters.
    pub query: Vec<(String, String)>,
    /// True if request carried a body.
    pub has_body: bool,
    /// True when request is mutating.
    pub mutating: bool,
}

/// HTTP response event.
#[derive(Debug, Clone)]
pub struct HttpResponseEvent {
    /// HTTP method.
    pub method: String,
    /// Full URL.
    pub url: String,
    /// Response status code, if available.
    pub status: Option<u16>,
    /// Response elapsed time.
    pub elapsed_ms: u64,
    /// Retry-after seconds if present.
    pub retry_after: Option<u64>,
    /// Error summary, when request failed.
    pub error: Option<String>,
}

/// Main FreeAgent API client.
pub struct FreeAgentClient {
    config: SdkConfig,
    auth: Arc<AuthManager>,
    http_client: reqwest::Client,
    observer: Option<Arc<dyn HttpObserver>>,
}

impl FreeAgentClient {
    /// Creates a builder.
    pub fn builder() -> FreeAgentClientBuilder {
        FreeAgentClientBuilder::default()
    }

    /// Returns current configuration.
    pub fn config(&self) -> &SdkConfig {
        &self.config
    }

    /// Returns auth manager.
    pub fn auth(&self) -> &AuthManager {
        &self.auth
    }

    /// Returns generic resource API wrapper for a spec.
    pub fn resource(&self, spec: ResourceSpec) -> ResourceApi<'_> {
        ResourceApi::new(self, spec)
    }

    /// Fetches a singleton resource/object.
    pub async fn get_json(&self, path: &str, query: &[(String, String)]) -> Result<Value> {
        let response = self
            .request(reqwest::Method::GET, path, query, None, false)
            .await?;
        Ok(response.body)
    }

    /// Sends POST JSON.
    pub async fn post_json(&self, path: &str, body: &Value, mutating: bool) -> Result<Value> {
        let response = self
            .request(reqwest::Method::POST, path, &[], Some(body), mutating)
            .await?;
        Ok(response.body)
    }

    /// Sends PUT JSON.
    pub async fn put_json(&self, path: &str, body: &Value, mutating: bool) -> Result<Value> {
        let response = self
            .request(reqwest::Method::PUT, path, &[], Some(body), mutating)
            .await?;
        Ok(response.body)
    }

    /// Sends DELETE.
    pub async fn delete_json(&self, path: &str, mutating: bool) -> Result<Value> {
        let response = self
            .request(reqwest::Method::DELETE, path, &[], None, mutating)
            .await?;
        Ok(response.body)
    }

    /// Fetches all pages for a list endpoint.
    pub async fn list_paginated(
        &self,
        path: &str,
        collection_key: &str,
        query: &[(String, String)],
        pagination: Pagination,
    ) -> Result<ListResult> {
        let per_page = pagination.per_page.clamp(1, 100);
        let mut page: u32 = 1;
        let mut items: Vec<Value> = Vec::new();
        let mut total: Option<usize> = None;
        let mut has_more;

        loop {
            let mut page_query = query.to_vec();
            page_query.push(("page".to_string(), page.to_string()));
            page_query.push(("per_page".to_string(), per_page.to_string()));

            let response = self
                .request(reqwest::Method::GET, path, &page_query, None, false)
                .await?;

            if total.is_none() {
                total = response
                    .headers
                    .get("X-Total-Count")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|raw| raw.parse::<usize>().ok());
            }

            let page_items = extract_collection(&response.body, collection_key)?;
            let count_before = items.len();
            items.extend(page_items);

            if !pagination.all && pagination.limit > 0 && items.len() >= pagination.limit {
                items.truncate(pagination.limit);
                has_more = true;
                break;
            }

            let added = items.len() - count_before;
            has_more = response_has_next_link(&response.headers);

            if added == 0 || !has_more {
                break;
            }

            page += 1;
        }

        Ok(ListResult {
            items,
            total,
            has_more,
            page,
            per_page,
        })
    }

    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        query: &[(String, String)],
        body: Option<&Value>,
        mutating: bool,
    ) -> Result<RawResponse> {
        if mutating && !self.config.allow_writes {
            return Err(ChoSdkError::WriteNotAllowed {
                message:
                    "Set [safety] allow_writes = true in config.toml to enable mutating commands"
                        .to_string(),
            });
        }

        let max_retries = self.config.max_retries;
        let url = build_url(&self.config.base_url, path)?;
        let mut did_refresh = false;

        for attempt in 0..=max_retries {
            let started = Instant::now();
            let access_token = self.auth.get_access_token().await?;

            if let Some(observer) = &self.observer {
                observer.on_request(&HttpRequestEvent {
                    method: method.as_str().to_string(),
                    url: url.clone(),
                    query: query.to_vec(),
                    has_body: body.is_some(),
                    mutating,
                });
            }

            let mut request = self
                .http_client
                .request(method.clone(), &url)
                .header(reqwest::header::ACCEPT, "application/json")
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .header(reqwest::header::USER_AGENT, &self.config.user_agent)
                .bearer_auth(access_token)
                .query(query);

            if let Some(payload) = body {
                request = request.json(payload);
            }

            let result = request.send().await;
            let elapsed_ms = started.elapsed().as_millis() as u64;

            let response = match result {
                Ok(resp) => resp,
                Err(err) => {
                    if let Some(observer) = &self.observer {
                        observer.on_response(&HttpResponseEvent {
                            method: method.as_str().to_string(),
                            url: url.clone(),
                            status: None,
                            elapsed_ms,
                            retry_after: None,
                            error: Some(err.to_string()),
                        });
                    }

                    if attempt < max_retries && (err.is_connect() || err.is_timeout()) {
                        let delay = backoff_delay(attempt);
                        warn!(
                            attempt = attempt + 1,
                            max_attempts = max_retries + 1,
                            delay_ms = delay.as_millis() as u64,
                            "network error, retrying"
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    return Err(ChoSdkError::Network(err));
                }
            };

            let status = response.status();
            let headers = response.headers().clone();
            let retry_after = headers
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());

            if let Some(observer) = &self.observer {
                observer.on_response(&HttpResponseEvent {
                    method: method.as_str().to_string(),
                    url: url.clone(),
                    status: Some(status.as_u16()),
                    elapsed_ms,
                    retry_after,
                    error: None,
                });
            }

            if status == reqwest::StatusCode::UNAUTHORIZED {
                if !did_refresh {
                    did_refresh = true;
                    self.auth.refresh().await?;
                    continue;
                }
                return Err(ChoSdkError::TokenExpired {
                    message: "Access token invalid and refresh failed".to_string(),
                });
            }

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                let wait = retry_after.unwrap_or(60);
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
                    continue;
                }
                return Err(ChoSdkError::RateLimited { retry_after: wait });
            }

            let text = response.text().await.map_err(ChoSdkError::Network)?;

            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(ChoSdkError::NotFound {
                    resource: path.to_string(),
                    id: path.rsplit('/').next().unwrap_or_default().to_string(),
                });
            }

            if !status.is_success() {
                return Err(ChoSdkError::api(status, text));
            }

            let body = if text.trim().is_empty() {
                Value::Object(serde_json::Map::new())
            } else {
                serde_json::from_str::<Value>(&text).map_err(|e| ChoSdkError::Parse {
                    message: format!("Failed to parse API response JSON: {e}"),
                })?
            };

            debug!(status = status.as_u16(), "api request successful");

            return Ok(RawResponse { body, headers });
        }

        Err(ChoSdkError::ApiError {
            status: 0,
            message: "Request failed after retries".to_string(),
        })
    }
}

impl std::fmt::Debug for FreeAgentClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FreeAgentClient")
            .field("config", &self.config)
            .field("auth", &"[REDACTED]")
            .finish()
    }
}

struct RawResponse {
    body: Value,
    headers: reqwest::header::HeaderMap,
}

/// Builder for [`FreeAgentClient`].
#[derive(Default)]
pub struct FreeAgentClientBuilder {
    config: Option<SdkConfig>,
    auth: Option<AuthManager>,
    observer: Option<Arc<dyn HttpObserver>>,
}

impl FreeAgentClientBuilder {
    /// Sets config.
    pub fn config(mut self, config: SdkConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Sets auth manager.
    pub fn auth_manager(mut self, auth: AuthManager) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Sets optional HTTP observer.
    pub fn observer(mut self, observer: Arc<dyn HttpObserver>) -> Self {
        self.observer = Some(observer);
        self
    }

    /// Builds client.
    pub fn build(self) -> Result<FreeAgentClient> {
        let config = self.config.unwrap_or_default();

        if !config.is_valid_url_scheme() {
            return Err(ChoSdkError::Config {
                message: "Invalid URL scheme in SDK config. Only http(s) URLs are allowed."
                    .to_string(),
            });
        }

        let auth = self.auth.ok_or_else(|| ChoSdkError::Config {
            message: "Auth manager is required".to_string(),
        })?;

        let http_client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(ChoSdkError::Network)?;

        Ok(FreeAgentClient {
            config,
            auth: Arc::new(auth),
            http_client,
            observer: self.observer,
        })
    }
}

fn extract_collection(body: &Value, collection_key: &str) -> Result<Vec<Value>> {
    let array = body
        .get(collection_key)
        .and_then(|value| value.as_array())
        .ok_or_else(|| ChoSdkError::Parse {
            message: format!("List response missing collection key '{collection_key}'"),
        })?;

    Ok(array.clone())
}

fn response_has_next_link(headers: &reqwest::header::HeaderMap) -> bool {
    let Some(link) = headers.get("Link") else {
        return false;
    };

    let Ok(link_raw) = link.to_str() else {
        return false;
    };

    link_raw
        .split(',')
        .map(|segment| segment.trim())
        .any(|segment| segment.ends_with("rel=\"next\"") || segment.ends_with("rel=next"))
}

fn build_url(base_url: &str, path: &str) -> Result<String> {
    if path.starts_with("http://") || path.starts_with("https://") {
        return Ok(path.to_string());
    }

    let base = base_url.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    Ok(format!("{base}/{path}"))
}

fn backoff_delay(attempt: u32) -> std::time::Duration {
    let base_secs = 1_u64 << attempt.min(4);
    std::time::Duration::from_secs(base_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_collection_returns_array_items() {
        let body = serde_json::json!({
            "contacts": [
                {"url": "https://api.freeagent.com/v2/contacts/1"}
            ]
        });

        let items = extract_collection(&body, "contacts").expect("array should be extracted");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["url"], "https://api.freeagent.com/v2/contacts/1");
    }

    #[test]
    fn extract_collection_errors_when_key_missing() {
        let body = serde_json::json!({ "contact": {} });
        let err = extract_collection(&body, "contacts").expect_err("missing key must fail");
        assert!(
            err.to_string()
                .contains("missing collection key 'contacts'")
        );
    }

    #[test]
    fn response_has_next_link_detects_next_relation() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Link",
            reqwest::header::HeaderValue::from_static(
                "<https://api.freeagent.com/v2/contacts?page=2>; rel=\"next\"",
            ),
        );

        assert!(response_has_next_link(&headers));
    }

    #[test]
    fn response_has_next_link_returns_false_without_next_relation() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Link",
            reqwest::header::HeaderValue::from_static(
                "<https://api.freeagent.com/v2/contacts?page=1>; rel=\"prev\"",
            ),
        );

        assert!(!response_has_next_link(&headers));
    }

    #[test]
    fn build_url_joins_relative_path_against_base() {
        let url = build_url("https://api.freeagent.com/v2/", "contacts/123")
            .expect("url should be built");
        assert_eq!(url, "https://api.freeagent.com/v2/contacts/123");
    }

    #[test]
    fn build_url_preserves_absolute_path() {
        let url = build_url(
            "https://api.freeagent.com/v2/",
            "https://api.freeagent.com/v2/contacts/123",
        )
        .expect("url should be preserved");
        assert_eq!(url, "https://api.freeagent.com/v2/contacts/123");
    }

    #[test]
    fn backoff_delay_caps_growth_at_sixteen_seconds() {
        assert_eq!(backoff_delay(0), std::time::Duration::from_secs(1));
        assert_eq!(backoff_delay(1), std::time::Duration::from_secs(2));
        assert_eq!(backoff_delay(2), std::time::Duration::from_secs(4));
        assert_eq!(backoff_delay(5), std::time::Duration::from_secs(16));
        assert_eq!(backoff_delay(8), std::time::Duration::from_secs(16));
    }
}
