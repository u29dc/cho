//! Table formatter.

use comfy_table::{Cell, CellAlignment, ContentArrangement, Table};

/// Table column metadata.
pub struct Column {
    /// Header label.
    pub header: String,
    /// Cell alignment.
    pub alignment: CellAlignment,
    /// Max width before truncation (`0` means unlimited).
    pub max_width: u16,
}

/// Builds a table string.
pub fn format_table(columns: &[Column], rows: &[Vec<String>]) -> String {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(
        columns
            .iter()
            .map(|column| Cell::new(&column.header).set_alignment(column.alignment))
            .collect::<Vec<_>>(),
    );

    for row in rows {
        let rendered = row
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let column = columns.get(index);
                let rendered = if let Some(column) = column {
                    if column.max_width >= 4 && value.len() > column.max_width as usize {
                        let max = column.max_width as usize - 3;
                        format!("{}...", &value[..max])
                    } else {
                        value.clone()
                    }
                } else {
                    value.clone()
                };

                Cell::new(rendered).set_alignment(
                    column
                        .map(|column| column.alignment)
                        .unwrap_or(CellAlignment::Left),
                )
            })
            .collect::<Vec<_>>();

        table.add_row(rendered);
    }

    table.to_string()
}

/// Helper for left-aligned text columns.
pub fn text_col_static(header: &str) -> Column {
    Column {
        header: header.to_string(),
        alignment: CellAlignment::Left,
        max_width: 0,
    }
}
