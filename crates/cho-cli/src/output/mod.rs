//! Output formatting: JSON, table, and CSV.

pub mod csv;
pub mod json;
pub mod table;

use serde_json::Value;

/// Extracts headers and rows from a JSON value for table/CSV rendering.
///
/// For objects: single row with keys as headers.
/// For arrays of objects: keys from the first object as headers, all objects as rows.
/// Falls back to a single-column "value" representation for non-object types.
pub fn value_to_rows(value: &Value) -> (Vec<String>, Vec<Vec<String>>) {
    match value {
        Value::Array(arr) if !arr.is_empty() => {
            // Collect headers from the first object
            let headers = if let Some(obj) = arr[0].as_object() {
                obj.keys().cloned().collect::<Vec<_>>()
            } else {
                return (vec!["value".to_string()], arr.iter().map(|v| vec![value_display(v)]).collect());
            };
            let rows = arr
                .iter()
                .map(|item| {
                    headers
                        .iter()
                        .map(|h| {
                            item.get(h).map(value_display).unwrap_or_default()
                        })
                        .collect()
                })
                .collect();
            (headers, rows)
        }
        Value::Object(obj) => {
            let headers: Vec<String> = obj.keys().cloned().collect();
            let row: Vec<String> = headers.iter().map(|h| obj.get(h).map(value_display).unwrap_or_default()).collect();
            (headers, vec![row])
        }
        other => (vec!["value".to_string()], vec![vec![value_display(other)]]),
    }
}

/// Formats a JSON value as a display string for table/CSV cells.
fn value_display(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        // For nested objects/arrays, fall back to compact JSON
        other => other.to_string(),
    }
}

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
