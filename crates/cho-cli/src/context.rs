//! Shared CLI execution context.

use std::time::Instant;

use cho_sdk::client::FreeAgentClient;
use cho_sdk::error::{ChoSdkError, Result};
use cho_sdk::models::{ListResult, Pagination};
use serde::Serialize;

use crate::audit::AuditLogger;
use crate::envelope;
use crate::output::json::{JsonOptions, apply_json_options};
use crate::output::{OutputFormat, value_to_rows};

/// Shared command execution context.
pub struct CliContext {
    client: FreeAgentClient,
    format: OutputFormat,
    json_options: JsonOptions,
    limit: usize,
    all: bool,
    allow_writes: bool,
    audit: AuditLogger,
}

impl CliContext {
    /// Creates a new context.
    pub fn new(
        client: FreeAgentClient,
        format: OutputFormat,
        json_options: JsonOptions,
        limit: usize,
        all: bool,
        allow_writes: bool,
        audit: AuditLogger,
    ) -> Self {
        Self {
            client,
            format,
            json_options,
            limit,
            all,
            allow_writes,
            audit,
        }
    }

    /// Returns client.
    pub fn client(&self) -> &FreeAgentClient {
        &self.client
    }

    /// Returns list pagination settings.
    pub fn pagination(&self) -> Pagination {
        if self.all {
            Pagination::all()
        } else {
            Pagination {
                per_page: 100,
                limit: self.limit.min(10_000),
                all: false,
            }
        }
    }

    /// Fails when writes are disabled.
    pub fn require_writes_allowed(&self) -> Result<()> {
        if self.allow_writes {
            Ok(())
        } else {
            Err(ChoSdkError::WriteNotAllowed {
                message:
                    "Write operations are blocked. Set [safety] allow_writes = true in config.toml"
                        .to_string(),
            })
        }
    }

    /// Logs structured command input payload.
    pub fn log_input(&self, tool: &str, input: &serde_json::Value) {
        let _ = self.audit.log_command_input(tool, &input.to_string());
    }

    /// Emits one-item success output.
    pub fn emit_success<T: Serialize>(&self, tool: &str, data: &T, start: Instant) -> Result<()> {
        let value = serialize_transform(data, &self.json_options)?;

        let output = match self.format {
            OutputFormat::Json => envelope::emit_success(tool, value, start, None, None, None),
            OutputFormat::Table => format_table_or_csv(value, OutputFormat::Table),
            OutputFormat::Csv => format_table_or_csv(value, OutputFormat::Csv),
        };

        println!("{output}");
        let _ = self.audit.log_command_output(tool, &output);
        Ok(())
    }

    /// Emits list success output.
    pub fn emit_list(&self, tool: &str, result: &ListResult, start: Instant) -> Result<()> {
        let value = serialize_transform(&result.items, &self.json_options)?;

        let output = match self.format {
            OutputFormat::Json => envelope::emit_success(
                tool,
                value,
                start,
                Some(result.items.len()),
                result.total,
                Some(result.has_more),
            ),
            OutputFormat::Table => format_table_or_csv(value, OutputFormat::Table),
            OutputFormat::Csv => format_table_or_csv(value, OutputFormat::Csv),
        };

        println!("{output}");
        let _ = self.audit.log_command_output(tool, &output);
        Ok(())
    }
}

fn serialize_transform<T: Serialize + ?Sized>(
    value: &T,
    json_options: &JsonOptions,
) -> Result<serde_json::Value> {
    let value = serde_json::to_value(value).map_err(|e| ChoSdkError::Parse {
        message: format!("Failed serializing output payload: {e}"),
    })?;

    Ok(apply_json_options(value, json_options))
}

fn format_table_or_csv(value: serde_json::Value, format: OutputFormat) -> String {
    let (headers, rows) = value_to_rows(&value);

    match format {
        OutputFormat::Table => {
            let columns = headers
                .iter()
                .map(|header| crate::output::table::text_col_static(header))
                .collect::<Vec<_>>();
            crate::output::table::format_table(&columns, &rows)
        }
        OutputFormat::Csv => {
            let refs = headers.iter().map(String::as_str).collect::<Vec<_>>();
            crate::output::csv::format_csv(&refs, &rows)
        }
        OutputFormat::Json => unreachable!("JSON formatting is handled by envelopes"),
    }
}
