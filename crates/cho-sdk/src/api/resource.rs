//! Generic FreeAgent resource API.

use serde_json::Value;

use crate::client::FreeAgentClient;
use crate::client::RequestPolicy;
use crate::error::{ChoSdkError, Result};
use crate::models::{ListResult, Pagination};

use super::specs::ResourceSpec;

/// Generic resource API wrapper.
pub struct ResourceApi<'a> {
    client: &'a FreeAgentClient,
    spec: ResourceSpec,
}

impl<'a> ResourceApi<'a> {
    /// Creates a new resource API wrapper.
    pub(crate) fn new(client: &'a FreeAgentClient, spec: ResourceSpec) -> Self {
        Self { client, spec }
    }

    /// Returns resource spec.
    pub fn spec(&self) -> ResourceSpec {
        self.spec
    }

    /// Lists resources using query params and pagination settings.
    pub async fn list(
        &self,
        query: &[(String, String)],
        pagination: Pagination,
    ) -> Result<ListResult> {
        self.list_with_policy(query, pagination, RequestPolicy::default())
            .await
    }

    /// Lists resources using query params and pagination settings with policy overrides.
    pub async fn list_with_policy(
        &self,
        query: &[(String, String)],
        pagination: Pagination,
        policy: RequestPolicy,
    ) -> Result<ListResult> {
        self.client
            .list_paginated_with_policy(
                self.spec.path,
                self.spec.collection_key,
                query,
                pagination,
                policy,
            )
            .await
    }

    /// Gets a single resource by identifier.
    pub async fn get(&self, id: &str) -> Result<Value> {
        self.get_with_policy(id, RequestPolicy::default()).await
    }

    /// Gets a single resource by identifier with policy overrides.
    pub async fn get_with_policy(&self, id: &str, policy: RequestPolicy) -> Result<Value> {
        let path = resource_target_path(self.spec.path, id);
        let response = self.client.get_json_with_policy(&path, &[], policy).await?;
        unwrap_singular(&response, self.spec.singular_key, self.spec.collection_key)
    }

    /// Creates a resource using request payload.
    pub async fn create(&self, body: &Value) -> Result<Value> {
        let payload = normalize_payload(body, self.spec.singular_key);
        let response = self
            .client
            .post_json(self.spec.path, &payload, true)
            .await?;
        unwrap_singular(&response, self.spec.singular_key, self.spec.collection_key)
    }

    /// Updates a resource by identifier.
    pub async fn update(&self, id: &str, body: &Value) -> Result<Value> {
        let payload = normalize_payload(body, self.spec.singular_key);
        let path = resource_target_path(self.spec.path, id);
        let response = self.client.put_json(&path, &payload, true).await?;
        unwrap_singular(&response, self.spec.singular_key, self.spec.collection_key)
    }

    /// Deletes a resource by identifier.
    pub async fn delete(&self, id: &str) -> Result<Value> {
        let path = resource_target_path(self.spec.path, id);
        self.client.delete_json(&path, true).await
    }

    /// Executes an action endpoint under a resource identifier.
    pub async fn action(
        &self,
        id: &str,
        method: reqwest::Method,
        suffix: &str,
        body: Option<&Value>,
        mutating: bool,
    ) -> Result<Value> {
        let suffix = suffix.trim_start_matches('/');
        let path = format!("{}/{}", resource_target_path(self.spec.path, id), suffix);

        match method {
            reqwest::Method::GET => self.client.get_json(&path, &[]).await,
            reqwest::Method::POST => {
                self.client
                    .post_json(
                        &path,
                        body.unwrap_or(&Value::Object(serde_json::Map::new())),
                        mutating,
                    )
                    .await
            }
            reqwest::Method::PUT => {
                self.client
                    .put_json(
                        &path,
                        body.unwrap_or(&Value::Object(serde_json::Map::new())),
                        mutating,
                    )
                    .await
            }
            reqwest::Method::DELETE => self.client.delete_json(&path, mutating).await,
            unsupported => Err(ChoSdkError::Config {
                message: format!("Unsupported action method for resource API: {unsupported}"),
            }),
        }
    }
}

fn normalize_payload(body: &Value, singular_key: &str) -> Value {
    if let Value::Object(map) = body
        && map.contains_key(singular_key)
    {
        return body.clone();
    }

    serde_json::json!({
        singular_key: body,
    })
}

fn unwrap_singular(response: &Value, singular_key: &str, collection_key: &str) -> Result<Value> {
    if let Some(value) = response.get(singular_key) {
        return Ok(value.clone());
    }

    if let Some(array) = response.get(collection_key).and_then(|v| v.as_array()) {
        if let Some(first) = array.first() {
            return Ok(first.clone());
        }
    }

    if response.is_object() {
        return Ok(response.clone());
    }

    Err(ChoSdkError::Parse {
        message: format!(
            "Response did not contain expected keys '{singular_key}' or '{collection_key}'"
        ),
    })
}

fn encode_path_segment(id: &str) -> String {
    url::form_urlencoded::byte_serialize(id.as_bytes()).collect()
}

fn resource_target_path(resource_path: &str, id: &str) -> String {
    let trimmed = id.trim();
    if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
        return trimmed.trim_end_matches('/').to_string();
    }

    format!(
        "{}/{}",
        resource_path.trim_end_matches('/'),
        encode_path_segment(trimmed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_target_path_uses_absolute_url_ids_unchanged_except_trailing_slash() {
        let url = "https://api.freeagent.com/v2/contacts/123/";
        let path = resource_target_path("contacts", url);
        assert_eq!(path, "https://api.freeagent.com/v2/contacts/123");
    }

    #[test]
    fn resource_target_path_encodes_relative_ids() {
        let path = resource_target_path("contacts", "abc/123");
        assert_eq!(path, "contacts/abc%2F123");
    }

    #[test]
    fn normalize_payload_wraps_unwrapped_body() {
        let body = serde_json::json!({
            "first_name": "Ada",
            "last_name": "Lovelace"
        });
        let wrapped = normalize_payload(&body, "contact");
        assert_eq!(
            wrapped,
            serde_json::json!({
                "contact": {
                    "first_name": "Ada",
                    "last_name": "Lovelace"
                }
            })
        );
    }

    #[test]
    fn normalize_payload_keeps_already_wrapped_body() {
        let body = serde_json::json!({
            "contact": {
                "first_name": "Ada"
            }
        });
        let wrapped = normalize_payload(&body, "contact");
        assert_eq!(wrapped, body);
    }

    #[test]
    fn unwrap_singular_prefers_singular_key() {
        let response = serde_json::json!({
            "contact": {
                "url": "https://api.freeagent.com/v2/contacts/1",
                "first_name": "Ada"
            }
        });
        let out = unwrap_singular(&response, "contact", "contacts").expect("must unwrap");
        assert_eq!(out["first_name"], "Ada");
    }

    #[test]
    fn unwrap_singular_falls_back_to_first_collection_item() {
        let response = serde_json::json!({
            "contacts": [
                {"url": "https://api.freeagent.com/v2/contacts/1"},
                {"url": "https://api.freeagent.com/v2/contacts/2"}
            ]
        });
        let out = unwrap_singular(&response, "contact", "contacts").expect("must unwrap");
        assert_eq!(out["url"], "https://api.freeagent.com/v2/contacts/1");
    }
}
