//! CLI execution context: client, formatting options, and global state.

use cho_sdk::client::XeroClient;
use cho_sdk::http::pagination::PaginationParams;
use serde::Serialize;

use crate::output::OutputFormat;
use crate::output::json::{JsonOptions, format_json, format_json_list};

/// Shared context for all CLI commands.
pub struct CliContext {
    /// The Xero API client.
    client: XeroClient,

    /// Output format.
    format: OutputFormat,

    /// JSON formatting options.
    json_options: JsonOptions,

    /// Maximum items for list commands.
    limit: usize,

    /// Fetch all pages (no limit).
    all: bool,
}

impl CliContext {
    /// Creates a new CLI context.
    pub fn new(
        client: XeroClient,
        format: OutputFormat,
        json_options: JsonOptions,
        limit: usize,
        all: bool,
    ) -> Self {
        Self {
            client,
            format,
            json_options,
            limit,
            all,
        }
    }

    /// Returns the Xero client.
    pub fn client(&self) -> &XeroClient {
        &self.client
    }

    /// Returns pagination parameters based on --limit and --all flags.
    pub fn pagination_params(&self) -> PaginationParams {
        if self.all {
            PaginationParams::all()
        } else {
            PaginationParams::with_limit(self.limit)
        }
    }

    /// Returns whether JSON error formatting should be used.
    pub fn json_errors(&self) -> bool {
        self.format == OutputFormat::Json
    }

    /// Formats a serializable value according to the selected output format.
    pub fn format_output<T: Serialize>(&self, value: &T) -> cho_sdk::error::Result<String> {
        format_json(value, &self.json_options)
            .map_err(|e| cho_sdk::error::ChoSdkError::Parse { message: e })
    }

    /// Formats a list of items, optionally wrapping with `--meta` envelope.
    pub fn format_list_output<T: Serialize>(&self, items: &[T]) -> cho_sdk::error::Result<String> {
        format_json_list(items, None, &self.json_options)
            .map_err(|e| cho_sdk::error::ChoSdkError::Parse { message: e })
    }
}
