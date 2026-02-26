//! JSON output transforms.

use serde_json::Value;

/// JSON output options.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonOptions {
    /// Keep source keys untouched.
    pub raw: bool,
    /// Convert decimal-like numbers to strings.
    pub precise: bool,
}

/// Applies output options.
pub fn apply_json_options(value: Value, options: &JsonOptions) -> Value {
    let _ = options.raw;
    transform(value, options.precise)
}

fn transform(value: Value, precise: bool) -> Value {
    match value {
        Value::Object(map) => {
            let mapped = map
                .into_iter()
                .map(|(key, value)| (key, transform(value, precise)))
                .collect();
            Value::Object(mapped)
        }
        Value::Array(items) => {
            Value::Array(items.into_iter().map(|v| transform(v, precise)).collect())
        }
        Value::Number(number) => {
            if precise {
                let as_string = number.to_string();
                if as_string.contains('.') {
                    Value::String(as_string)
                } else {
                    Value::Number(number)
                }
            } else {
                Value::Number(number)
            }
        }
        other => other,
    }
}
