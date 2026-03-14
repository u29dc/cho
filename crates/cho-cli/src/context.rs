//! Shared CLI execution context.

use std::time::Instant;

use cho_sdk::client::FreeAgentClient;
use cho_sdk::error::{ChoSdkError, Result};
use cho_sdk::models::{ListResult, Pagination};
use serde::Serialize;

use crate::audit::AuditLogger;
use crate::envelope;
use crate::output::json::{JsonOptions, apply_json_options};
use crate::output::{OutputFormat, OutputMode, format_value};

/// Shared command execution context.
pub struct CliContext {
    client: FreeAgentClient,
    output_mode: OutputMode,
    json_options: JsonOptions,
    limit: usize,
    explicit_limit: bool,
    all: bool,
    allow_writes: bool,
    audit: AuditLogger,
}

impl CliContext {
    /// Creates a new context.
    pub fn new(
        client: FreeAgentClient,
        output_mode: OutputMode,
        json_options: JsonOptions,
        limit: usize,
        all: bool,
        allow_writes: bool,
        audit: AuditLogger,
    ) -> Self {
        Self {
            client,
            output_mode,
            json_options,
            limit,
            explicit_limit: false,
            all,
            allow_writes,
            audit,
        }
    }

    /// Marks whether the global limit came from an explicit CLI flag.
    pub fn with_explicit_limit(mut self, explicit_limit: bool) -> Self {
        self.explicit_limit = explicit_limit;
        self
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

    /// Returns the configured item limit regardless of `--all`.
    pub fn limit(&self) -> usize {
        self.limit.min(10_000)
    }

    /// Returns a compact summary slice unless the user explicitly requested a limit.
    pub fn summary_limit(&self, default_limit: usize) -> usize {
        if self.explicit_limit {
            self.limit()
        } else {
            self.limit().min(default_limit.max(1))
        }
    }

    /// Returns true when `--all` was requested.
    pub fn all_requested(&self) -> bool {
        self.all
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

        let output = match self.output_mode {
            OutputMode::Json => envelope::emit_success(tool, value, start, None, None, None),
            OutputMode::Text | OutputMode::Table => format_value(&value, OutputFormat::Table),
            OutputMode::Csv => format_value(&value, OutputFormat::Csv),
        };

        println!("{output}");
        let _ = self.audit.log_command_output(tool, &output);
        Ok(())
    }

    /// Emits list success output.
    pub fn emit_list(&self, tool: &str, result: &ListResult, start: Instant) -> Result<()> {
        let value = serialize_transform(&result.items, &self.json_options)?;

        let output = match self.output_mode {
            OutputMode::Json => envelope::emit_success(
                tool,
                value,
                start,
                Some(result.items.len()),
                result.total,
                Some(result.has_more),
            ),
            OutputMode::Text | OutputMode::Table => format_value(&value, OutputFormat::Table),
            OutputMode::Csv => format_value(&value, OutputFormat::Csv),
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
