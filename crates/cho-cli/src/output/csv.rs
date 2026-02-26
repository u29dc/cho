//! CSV formatter.

/// Formats header + rows as CSV.
pub fn format_csv(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = String::new();

    out.push_str(
        &headers
            .iter()
            .map(|item| escape(item))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push('\n');

    for row in rows {
        out.push_str(
            &row.iter()
                .map(|item| escape(item))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }

    out
}

fn escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
