//! Blocking wrapper for [`crate::client::FreeAgentClient`].

use crate::api::ResourceSpec;
use crate::client::FreeAgentClient;
use crate::error::{ChoSdkError, Result};
use crate::models::{ListResult, Pagination};

/// Synchronous client wrapper.
pub struct BlockingClient {
    inner: FreeAgentClient,
    runtime: tokio::runtime::Runtime,
}

impl BlockingClient {
    /// Creates a blocking client from an async client.
    pub fn from_async(inner: FreeAgentClient) -> Result<Self> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(ChoSdkError::Config {
                message: "Blocking client cannot be created from within an async runtime"
                    .to_string(),
            });
        }

        let runtime = tokio::runtime::Runtime::new().map_err(|e| ChoSdkError::Config {
            message: format!("Failed to create tokio runtime for blocking client: {e}"),
        })?;

        Ok(Self { inner, runtime })
    }

    /// Lists resources synchronously.
    pub fn list(
        &self,
        spec: ResourceSpec,
        query: &[(String, String)],
        pagination: Pagination,
    ) -> Result<ListResult> {
        self.runtime
            .block_on(self.inner.resource(spec).list(query, pagination))
    }

    /// Gets resource synchronously.
    pub fn get(&self, spec: ResourceSpec, id: &str) -> Result<serde_json::Value> {
        self.runtime.block_on(self.inner.resource(spec).get(id))
    }
}
