//! JSON output transforms.

use serde_json::Value;

const SIGNED_URL_QUERY_REDACTED_SUFFIX: &str = "?[signed-query-redacted]";
const LOGO_SIGNED_URL_PATHS: &[&[&str]] = &[
    &["company", "logo", "url"],
    &["company", "logo", "content_src"],
    &["company", "logo", "content_src_small"],
    &["company", "logo", "content_src_medium"],
];

/// JSON output options.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonOptions {
    /// Convert decimal-like numbers to strings.
    pub precise: bool,
}

/// Applies output options.
pub fn apply_json_options(value: Value, options: &JsonOptions) -> Value {
    transform(value, options.precise, &[])
}

fn transform(value: Value, precise: bool, path: &[String]) -> Value {
    match value {
        Value::Object(map) => {
            let mapped = map
                .into_iter()
                .map(|(key, value)| {
                    let mut child_path = path.to_vec();
                    child_path.push(key.clone());
                    (key, transform(value, precise, &child_path))
                })
                .collect();
            Value::Object(mapped)
        }
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(|value| transform(value, precise, path))
                .collect(),
        ),
        Value::String(text) => {
            if is_logo_signed_url_path(path) {
                Value::String(compact_signed_url(&text))
            } else {
                Value::String(text)
            }
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

fn is_logo_signed_url_path(path: &[String]) -> bool {
    LOGO_SIGNED_URL_PATHS.iter().any(|candidate| {
        if path.len() < candidate.len() {
            return false;
        }

        let offset = path.len() - candidate.len();
        path[offset..]
            .iter()
            .map(String::as_str)
            .zip(candidate.iter().copied())
            .all(|(actual, expected)| actual == expected)
    })
}

fn compact_signed_url(raw: &str) -> String {
    let Ok(mut parsed) = url::Url::parse(raw) else {
        return raw.to_string();
    };

    let had_signed_query = parsed
        .query_pairs()
        .any(|(key, _)| key.starts_with("X-Amz-"));
    if !had_signed_query {
        return raw.to_string();
    }

    parsed.set_query(None);
    format!("{}{}", parsed, SIGNED_URL_QUERY_REDACTED_SUFFIX)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{JsonOptions, apply_json_options};

    #[test]
    fn compacts_only_targeted_company_logo_signed_url_fields() {
        let value = json!({
            "company": {
                "logo": {
                    "url": "https://example.com/logo.png?X-Amz-Signature=abc&X-Amz-Security-Token=def",
                    "content_src": "https://example.com/original.png?X-Amz-Algorithm=AWS4-HMAC-SHA256",
                    "content_src_small": "https://example.com/small.png?X-Amz-Date=20260227T004225Z",
                    "content_src_medium": "https://example.com/medium.png?X-Amz-Expires=900",
                    "content_type": "image/png"
                }
            },
            "website": "https://u29dc.com/?ref=nav",
            "locked_reason": "Some parts are locked and this should remain untouched."
        });

        let transformed = apply_json_options(value, &JsonOptions::default());

        assert_eq!(
            transformed["company"]["logo"]["url"],
            "https://example.com/logo.png?[signed-query-redacted]"
        );
        assert_eq!(
            transformed["company"]["logo"]["content_src"],
            "https://example.com/original.png?[signed-query-redacted]"
        );
        assert_eq!(
            transformed["company"]["logo"]["content_src_small"],
            "https://example.com/small.png?[signed-query-redacted]"
        );
        assert_eq!(
            transformed["company"]["logo"]["content_src_medium"],
            "https://example.com/medium.png?[signed-query-redacted]"
        );

        assert_eq!(transformed["company"]["logo"]["content_type"], "image/png");
        assert_eq!(transformed["website"], "https://u29dc.com/?ref=nav");
        assert_eq!(
            transformed["locked_reason"],
            "Some parts are locked and this should remain untouched."
        );
    }

    #[test]
    fn keeps_non_signed_queries_unchanged_even_on_targeted_paths() {
        let value = json!({
            "company": {
                "logo": {
                    "url": "https://example.com/logo.png?cache=1"
                }
            }
        });

        let transformed = apply_json_options(value, &JsonOptions::default());
        assert_eq!(
            transformed["company"]["logo"]["url"],
            "https://example.com/logo.png?cache=1"
        );
    }

    #[test]
    fn precise_mode_still_converts_decimal_numbers_to_strings() {
        let value = json!({
            "amount": 12.34,
            "count": 2
        });

        let transformed = apply_json_options(value, &JsonOptions { precise: true });
        assert_eq!(transformed["amount"], "12.34");
        assert_eq!(transformed["count"], 2);
    }
}
