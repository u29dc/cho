//! Output formatting: JSON, table, and CSV.

#[allow(dead_code)]
pub mod csv;
pub mod json;
#[allow(dead_code)]
pub mod table;

/// Output format selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// JSON output (snake_case keys, bare array by default).
    Json,
    /// Table output (comfy-table).
    Table,
    /// CSV output with header row.
    Csv,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Table => write!(f, "table"),
            Self::Csv => write!(f, "csv"),
        }
    }
}
