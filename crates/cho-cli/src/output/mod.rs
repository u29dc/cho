//! Output format helpers.

pub mod csv;
pub mod json;
pub mod table;

use serde_json::Value;

/// Output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// JSON output.
    Json,
    /// Table output.
    Table,
    /// CSV output.
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

/// Converts JSON value to rows for tabular output.
pub fn value_to_rows(value: &Value) -> (Vec<String>, Vec<Vec<String>>) {
    match value {
        Value::Array(items) if !items.is_empty() => {
            if let Some(first) = items[0].as_object() {
                let headers: Vec<String> = first.keys().cloned().collect();
                let rows = items
                    .iter()
                    .map(|item| {
                        headers
                            .iter()
                            .map(|header| item.get(header).map(value_display).unwrap_or_default())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();
                (headers, rows)
            } else {
                (
                    vec!["value".to_string()],
                    items.iter().map(|item| vec![value_display(item)]).collect(),
                )
            }
        }
        Value::Object(object) => {
            let headers: Vec<String> = object.keys().cloned().collect();
            let row = headers
                .iter()
                .map(|header| object.get(header).map(value_display).unwrap_or_default())
                .collect::<Vec<_>>();
            (headers, vec![row])
        }
        other => (vec!["value".to_string()], vec![vec![value_display(other)]]),
    }
}

fn value_display(value: &Value) -> String {
    match value {
        Value::String(inner) => inner.clone(),
        Value::Bool(inner) => inner.to_string(),
        Value::Number(inner) => inner.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}
