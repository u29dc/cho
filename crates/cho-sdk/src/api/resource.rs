//! Generic FreeAgent resource API.

use serde_json::Value;

use crate::client::FreeAgentClient;
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
        self.client
            .list_paginated(self.spec.path, self.spec.collection_key, query, pagination)
            .await
    }

    /// Gets a single resource by identifier.
    pub async fn get(&self, id: &str) -> Result<Value> {
        let path = resource_target_path(self.spec.path, id);
        let response = self.client.get_json(&path, &[]).await?;
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
