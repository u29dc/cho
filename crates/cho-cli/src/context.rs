//! CLI execution context: client, formatting options, and global state.

use cho_sdk::client::XeroClient;
use cho_sdk::http::pagination::{ListResult, PaginationParams};
use serde::Serialize;

use crate::output::OutputFormat;
use crate::output::json::{JsonOptions, format_json, format_json_list};
use crate::output::value_to_rows;

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
        match self.format {
            OutputFormat::Json => format_json(value, &self.json_options)
                .map_err(|e| cho_sdk::error::ChoSdkError::Parse { message: e }),
            OutputFormat::Table | OutputFormat::Csv => {
                let json_value = serde_json::to_value(value).map_err(|e| {
                    cho_sdk::error::ChoSdkError::Parse {
                        message: format!("JSON serialization failed: {e}"),
                    }
                })?;
                let transformed = if self.json_options.raw {
                    json_value
                } else {
                    crate::output::json::pascal_to_snake_keys(json_value)
                };
                let (headers, rows) = value_to_rows(&transformed);
                match self.format {
                    OutputFormat::Table => {
                        let columns: Vec<_> = headers
                            .iter()
                            .map(|h| crate::output::table::text_col_static(h))
                            .collect();
                        Ok(crate::output::table::format_table(&columns, &rows))
                    }
                    OutputFormat::Csv => {
                        let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
                        Ok(crate::output::csv::format_csv(&header_refs, &rows))
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    /// Formats a paginated list result, threading pagination metadata through
    /// the `--meta` envelope when enabled.
    pub fn format_paginated_output<T: Serialize>(
        &self,
        result: &ListResult<T>,
    ) -> cho_sdk::error::Result<String> {
        let pagination_json = result
            .pagination
            .as_ref()
            .and_then(|p| serde_json::to_value(p).ok());

        self.format_list_inner(&result.items, pagination_json.as_ref())
    }

    /// Formats a non-paginated list of items.
    pub fn format_list_output<T: Serialize>(&self, items: &[T]) -> cho_sdk::error::Result<String> {
        self.format_list_inner(items, None)
    }

    /// Inner list formatting with optional pagination metadata.
    fn format_list_inner<T: Serialize>(
        &self,
        items: &[T],
        pagination: Option<&serde_json::Value>,
    ) -> cho_sdk::error::Result<String> {
        match self.format {
            OutputFormat::Json => format_json_list(items, pagination, &self.json_options)
                .map_err(|e| cho_sdk::error::ChoSdkError::Parse { message: e }),
            OutputFormat::Table | OutputFormat::Csv => {
                let json_value = serde_json::to_value(items).map_err(|e| {
                    cho_sdk::error::ChoSdkError::Parse {
                        message: format!("JSON serialization failed: {e}"),
                    }
                })?;
                let transformed = if self.json_options.raw {
                    json_value
                } else {
                    crate::output::json::pascal_to_snake_keys(json_value)
                };
                let (headers, rows) = value_to_rows(&transformed);
                match self.format {
                    OutputFormat::Table => {
                        let columns: Vec<_> = headers
                            .iter()
                            .map(|h| crate::output::table::text_col_static(h))
                            .collect();
                        Ok(crate::output::table::format_table(&columns, &rows))
                    }
                    OutputFormat::Csv => {
                        let header_refs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
                        Ok(crate::output::csv::format_csv(&header_refs, &rows))
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    /// Checks whether write operations are allowed.
    ///
    /// Write operations are gated behind a config-file-only setting:
    /// `[safety] allow_writes = true` in `~/.config/cho/config.toml`.
    ///
    /// This cannot be overridden by CLI flags or environment variables.
    pub fn require_writes_allowed(&self) -> cho_sdk::error::Result<()> {
        check_writes_allowed()
    }
}

/// Reads the configuration file and checks if `[safety] allow_writes` is true.
///
/// Returns `Ok(())` if writes are allowed, or an error with a helpful message.
pub fn check_writes_allowed() -> cho_sdk::error::Result<()> {
    let config_path = cho_sdk::auth::storage::config_dir()?.join("config.toml");

    if !config_path.exists() {
        return Err(cho_sdk::error::ChoSdkError::WriteNotAllowed {
            message: format!(
                "Write operations are disabled by default. \
                 To enable, add the following to {}:\n\n\
                 [safety]\n\
                 allow_writes = true",
                config_path.display()
            ),
        });
    }

    let content =
        std::fs::read_to_string(&config_path).map_err(|e| cho_sdk::error::ChoSdkError::Config {
            message: format!("Failed to read config: {e}"),
        })?;

    let table: toml::Table = content
        .parse()
        .map_err(|e| cho_sdk::error::ChoSdkError::Config {
            message: format!("Failed to parse config: {e}"),
        })?;

    let allowed = table
        .get("safety")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("allow_writes"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if allowed {
        Ok(())
    } else {
        Err(cho_sdk::error::ChoSdkError::WriteNotAllowed {
            message: format!(
                "Write operations are not allowed in the current configuration. \
                 To enable, set the following in {}:\n\n\
                 [safety]\n\
                 allow_writes = true",
                config_path.display()
            ),
        })
    }
}
