//! Table output formatter using comfy-table.

use comfy_table::{Cell, CellAlignment, ContentArrangement, Table};

/// A column definition for table output.
pub struct Column {
    /// Header label.
    pub header: String,
    /// Alignment (left for text, right for numbers).
    pub alignment: CellAlignment,
    /// Maximum width before truncation (0 = no limit).
    pub max_width: u16,
}

/// Formats data as a table with the given columns.
///
/// `rows` is a Vec of Vec<String> where each inner Vec corresponds to one row.
pub fn format_table(columns: &[Column], rows: &[Vec<String>]) -> String {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Header row
    let headers: Vec<Cell> = columns
        .iter()
        .map(|c| Cell::new(&c.header).set_alignment(c.alignment))
        .collect();
    table.set_header(headers);

    // Data rows
    for row in rows {
        let cells: Vec<Cell> = row
            .iter()
            .enumerate()
            .map(|(i, val)| {
                let alignment = columns
                    .get(i)
                    .map(|c| c.alignment)
                    .unwrap_or(CellAlignment::Left);
                let display = if let Some(col) = columns.get(i) {
                    if col.max_width >= 4 && val.len() > col.max_width as usize {
                        let max = col.max_width as usize - 3;
                        let boundary = val
                            .char_indices()
                            .take_while(|(i, _)| *i < max)
                            .last()
                            .map_or(0, |(i, ch)| i + ch.len_utf8());
                        format!("{}...", &val[..boundary])
                    } else {
                        val.clone()
                    }
                } else {
                    val.clone()
                };
                Cell::new(display).set_alignment(alignment)
            })
            .collect();
        table.add_row(cells);
    }

    table.to_string()
}

/// Helper: creates a left-aligned text column.
pub fn text_col_static(header: &str) -> Column {
    Column {
        header: header.to_string(),
        alignment: CellAlignment::Left,
        max_width: 0,
    }
}
