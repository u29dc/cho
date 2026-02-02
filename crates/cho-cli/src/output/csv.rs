//! CSV output formatter.

/// Formats data as CSV with a header row.
///
/// Uses simple comma-separated output with quoting for fields containing
/// commas, quotes, or newlines.
pub fn format_csv(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut output = String::new();

    // Header row
    output.push_str(
        &headers
            .iter()
            .map(|h| csv_escape(h))
            .collect::<Vec<_>>()
            .join(","),
    );
    output.push('\n');

    // Data rows
    for row in rows {
        output.push_str(
            &row.iter()
                .map(|v| csv_escape(v))
                .collect::<Vec<_>>()
                .join(","),
        );
        output.push('\n');
    }

    output
}

/// Escapes a CSV field value, quoting if necessary.
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_basic() {
        let output = format_csv(
            &["Name", "Amount"],
            &[
                vec!["Test".to_string(), "100.00".to_string()],
                vec!["Other".to_string(), "200.50".to_string()],
            ],
        );
        assert_eq!(output, "Name,Amount\nTest,100.00\nOther,200.50\n");
    }

    #[test]
    fn csv_escape_commas() {
        assert_eq!(csv_escape("hello, world"), "\"hello, world\"");
        assert_eq!(csv_escape("normal"), "normal");
        assert_eq!(csv_escape("with\"quote"), "\"with\"\"quote\"");
    }
}
