//! JSON output formatter.
//!
//! Re-serializes SDK structs (PascalCase) to snake_case JSON for CLI output.
//! Supports `--raw` key preservation and `--precise` money-as-string formatting.

use serde_json::Value;

/// Options controlling JSON output behavior.
#[derive(Debug, Clone, Default)]
pub struct JsonOptions {
    /// When true, skip key transformation (preserve PascalCase Xero-native keys).
    ///
    /// Note: raw MS date (`/Date(epoch)/`) preservation would require SDK-level
    /// changes to serialize dates in their original format. Currently `--raw`
    /// preserves PascalCase keys but dates are still ISO 8601.
    pub raw: bool,
    /// Serialize money as strings instead of numbers.
    pub precise: bool,
}

/// Applies all configured JSON output transforms in a single traversal.
pub fn apply_json_options(value: Value, options: &JsonOptions) -> Value {
    transform_value(value, !options.raw, options.precise)
}

/// Recursively converts all object keys from PascalCase to snake_case.
#[allow(dead_code)]
pub fn pascal_to_snake_keys(value: Value) -> Value {
    transform_value(value, true, false)
}

/// Converts a PascalCase string to snake_case.
fn pascal_to_snake(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::with_capacity(s.len() + 4);
    let mut prev_upper = false;
    let mut prev_was_start = true;

    for i in 0..chars.len() {
        let ch = chars[i];
        if ch.is_uppercase() {
            if !prev_was_start && !prev_upper {
                result.push('_');
            } else if prev_upper && i + 1 < chars.len() {
                // Handle sequences like "ID" -> "id", "UTC" -> "utc"
                // But "IDField" -> "id_field"
                if chars[i + 1].is_lowercase() && !prev_was_start {
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
#[allow(dead_code)]
pub fn money_to_strings(value: Value) -> Value {
    transform_value(value, false, true)
}

fn transform_value(value: Value, transform_keys: bool, precise: bool) -> Value {
    match value {
        Value::Object(map) => {
            let new_map: serde_json::Map<String, Value> = map
                .into_iter()
                .map(|(k, v)| {
                    let key = if transform_keys {
                        pascal_to_snake(&k)
                    } else {
                        k
                    };
                    (key, transform_value(v, transform_keys, precise))
                })
                .collect();
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(|v| transform_value(v, transform_keys, precise))
                .collect(),
        ),
        Value::Number(n) => {
            if precise {
                // Convert numbers with decimals to strings, avoiding f64 intermediate
                // which can lose precision for large Decimal values.
                let s = n.to_string();
                if s.contains('.') {
                    Value::String(s)
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

    #[test]
    fn apply_json_options_combines_transforms() {
        let input = serde_json::json!({
            "InvoiceID": "inv-1",
            "Total": 123.45,
            "Count": 2
        });
        let output = apply_json_options(
            input,
            &JsonOptions {
                raw: false,
                precise: true,
            },
        );
        assert_eq!(output["invoice_id"], "inv-1");
        assert_eq!(output["total"], Value::String("123.45".to_string()));
        assert!(output["count"].is_number());
    }
}
