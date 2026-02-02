//! JSON output formatter.
//!
//! Re-serializes SDK structs (PascalCase) to snake_case JSON for CLI output.
//! Supports `--meta` envelope, `--raw` date preservation, and `--precise`
//! money-as-string formatting.

use serde::Serialize;
use serde_json::Value;

/// Options controlling JSON output behavior.
#[derive(Debug, Clone, Default)]
pub struct JsonOptions {
    /// Wrap output in `{"data": [...], "pagination": {...}}` envelope.
    pub meta: bool,
    /// When true, skip key transformation (preserve PascalCase Xero-native keys).
    ///
    /// Note: raw MS date (`/Date(epoch)/`) preservation would require SDK-level
    /// changes to serialize dates in their original format. Currently `--raw`
    /// preserves PascalCase keys but dates are still ISO 8601.
    pub raw: bool,
    /// Serialize money as strings instead of numbers.
    pub precise: bool,
}

/// Formats a serializable value as JSON.
///
/// By default, converts PascalCase keys to snake_case. When `raw` is true,
/// preserves Xero-native PascalCase keys.
pub fn format_json<T: Serialize>(value: &T, options: &JsonOptions) -> Result<String, String> {
    let json_value =
        serde_json::to_value(value).map_err(|e| format!("JSON serialization failed: {e}"))?;

    let transformed = if options.raw {
        json_value
    } else {
        pascal_to_snake_keys(json_value)
    };

    let output = if options.precise {
        money_to_strings(transformed)
    } else {
        transformed
    };

    serde_json::to_string_pretty(&output).map_err(|e| format!("JSON formatting failed: {e}"))
}

/// Formats a list with optional meta envelope.
pub fn format_json_list<T: Serialize>(
    items: &[T],
    pagination: Option<&serde_json::Value>,
    options: &JsonOptions,
) -> Result<String, String> {
    let json_value =
        serde_json::to_value(items).map_err(|e| format!("JSON serialization failed: {e}"))?;

    let transformed = if options.raw {
        json_value
    } else {
        pascal_to_snake_keys(json_value)
    };

    let output = if options.precise {
        money_to_strings(transformed)
    } else {
        transformed
    };

    if options.meta {
        let mut envelope = serde_json::Map::new();
        envelope.insert("data".to_string(), output);
        if let Some(pag) = pagination {
            let pag_value = if options.raw {
                pag.clone()
            } else {
                pascal_to_snake_keys(pag.clone())
            };
            envelope.insert("pagination".to_string(), pag_value);
        }
        serde_json::to_string_pretty(&Value::Object(envelope))
            .map_err(|e| format!("JSON formatting failed: {e}"))
    } else {
        serde_json::to_string_pretty(&output).map_err(|e| format!("JSON formatting failed: {e}"))
    }
}

/// Recursively converts all object keys from PascalCase to snake_case.
pub fn pascal_to_snake_keys(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let new_map: serde_json::Map<String, Value> = map
                .into_iter()
                .map(|(k, v)| (pascal_to_snake(&k), pascal_to_snake_keys(v)))
                .collect();
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(pascal_to_snake_keys).collect()),
        other => other,
    }
}

/// Converts a PascalCase string to snake_case.
fn pascal_to_snake(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut prev_upper = false;
    let mut prev_was_start = true;

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if !prev_was_start && !prev_upper {
                result.push('_');
            } else if prev_upper && i + 1 < s.len() {
                // Handle sequences like "ID" -> "id", "UTC" -> "utc"
                // But "IDField" -> "id_field"
                let next = s.chars().nth(i + 1);
                if let Some(next_ch) = next
                    && next_ch.is_lowercase()
                    && !prev_was_start
                {
                    result.push('_');
                }
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
            prev_upper = true;
        } else {
            result.push(ch);
            prev_upper = false;
        }
        prev_was_start = false;
    }

    result
}

/// Recursively converts numeric values that look like money (have decimal places)
/// to string representations for precise output.
fn money_to_strings(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let new_map: serde_json::Map<String, Value> = map
                .into_iter()
                .map(|(k, v)| (k, money_to_strings(v)))
                .collect();
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(money_to_strings).collect()),
        Value::Number(n) => {
            // Convert numbers with decimals to strings
            if let Some(f) = n.as_f64() {
                if f.fract() != 0.0 || n.to_string().contains('.') {
                    Value::String(n.to_string())
                } else {
                    Value::Number(n)
                }
            } else {
                Value::Number(n)
            }
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pascal_to_snake_basic() {
        assert_eq!(pascal_to_snake("InvoiceID"), "invoice_id");
        assert_eq!(pascal_to_snake("ContactName"), "contact_name");
        assert_eq!(pascal_to_snake("UpdatedDateUTC"), "updated_date_utc");
        assert_eq!(pascal_to_snake("Type"), "type");
        assert_eq!(pascal_to_snake("id"), "id");
    }

    #[test]
    fn pascal_to_snake_keys_transform() {
        let input = serde_json::json!({
            "InvoiceID": "123",
            "ContactName": "Test",
            "LineItems": [{"Description": "Item 1"}]
        });
        let output = pascal_to_snake_keys(input);
        assert!(output.get("invoice_id").is_some());
        assert!(output.get("contact_name").is_some());
        let items = output.get("line_items").unwrap().as_array().unwrap();
        assert!(items[0].get("description").is_some());
    }

    #[test]
    fn money_to_strings_conversion() {
        let input = serde_json::json!({
            "amount": 123.45,
            "count": 5,
            "nested": {"total": 0.01}
        });
        let output = money_to_strings(input);
        assert_eq!(output["amount"], Value::String("123.45".to_string()));
        assert!(output["count"].is_number()); // integers stay as numbers
        assert_eq!(output["nested"]["total"], Value::String("0.01".to_string()));
    }
}
