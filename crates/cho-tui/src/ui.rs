//! UI rendering layer for `cho-tui`.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap};

use crate::api::RoutePayload;
use crate::app::{App, FocusTarget};
use crate::palette::PaletteRow;
use crate::theme::Theme;

/// Renders one full frame.
pub fn render(frame: &mut Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_body(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);

    if app.palette.open {
        render_palette(frame, app);
    }
    if app.prompt.is_some() {
        render_prompt(frame, app);
    }
}

fn render_header(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let route = app.current_route();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(24),
            Constraint::Min(10),
            Constraint::Length(28),
        ])
        .split(area);

    let brand = Paragraph::new(Line::from(vec![
        Span::styled("■ ", Theme::brand()),
        Span::styled(
            format!("cho v{}", env!("CARGO_PKG_VERSION")),
            Theme::brand().add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(brand, chunks[0]);

    let center = Paragraph::new(Line::from(Span::styled(
        format!("{}/{}", route.workspace.label(), route.label),
        Theme::header_meta(),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(center, chunks[1]);

    let right = Paragraph::new(Line::from(Span::styled(
        "cmd+p | ctrl+p command palette",
        Theme::header_meta(),
    )))
    .alignment(Alignment::Right);
    frame.render_widget(right, chunks[2]);
}

fn render_body(frame: &mut Frame<'_>, app: &App, area: Rect) {
    if app.show_tree {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(1)])
            .split(area);
        render_navigation(frame, app, chunks[0]);
        render_main(frame, app, chunks[1]);
    } else {
        render_main(frame, app, area);
    }
}

fn render_navigation(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Navigation ", Theme::muted()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut items = Vec::new();
    let mut current_workspace = None;
    for (index, route) in app.routes.iter().enumerate() {
        if current_workspace != Some(route.workspace) {
            current_workspace = Some(route.workspace);
            items.push(
                ListItem::new(Line::from(Span::styled(
                    format!(" {} ", route.workspace.label()),
                    Theme::section_heading(),
                )))
                .style(Theme::section_heading()),
            );
        }

        let prefix = if index == app.active_route {
            "▸ "
        } else {
            "  "
        };
        let base = format!("{prefix}{}", route.label);

        let style = if index == app.nav_cursor && app.focus == FocusTarget::Navigation {
            Theme::selected()
        } else if index == app.active_route {
            Style::default()
                .fg(Theme::ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Theme::text()
        };
        items.push(ListItem::new(base).style(style));
    }

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_main(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let route = app.current_route();
    let title = format!(" {} ", route.label);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(title, Theme::muted()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(view) = app.current_view() else {
        let paragraph = Paragraph::new("No data loaded").style(Theme::muted());
        frame.render_widget(paragraph, inner);
        return;
    };

    if let Some(error) = &view.error {
        let paragraph = Paragraph::new(error.clone())
            .style(Theme::muted())
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, inner);
        return;
    }

    let Some(payload) = &view.payload else {
        let paragraph = Paragraph::new("Loading...").style(Theme::muted());
        frame.render_widget(paragraph, inner);
        return;
    };

    match payload {
        RoutePayload::Message(text) => {
            let paragraph = Paragraph::new(text.clone())
                .style(Theme::muted())
                .wrap(Wrap { trim: false });
            frame.render_widget(paragraph, inner);
        }
        RoutePayload::Object(value) => {
            let lines = format_object_lines(value, inner.height as usize);
            frame.render_widget(
                Paragraph::new(lines)
                    .style(Theme::text())
                    .wrap(Wrap { trim: false }),
                inner,
            );
        }
        RoutePayload::List {
            items,
            total,
            has_more,
        } => {
            render_list_with_detail(frame, inner, items, view.selected_row, *total, *has_more);
        }
    }
}

fn render_list_with_detail(
    frame: &mut Frame<'_>,
    area: Rect,
    items: &[serde_json::Value],
    selected_row: usize,
    total: Option<usize>,
    has_more: bool,
) {
    if items.is_empty() {
        frame.render_widget(Paragraph::new("No items found").style(Theme::muted()), area);
        return;
    }

    let chunks = if area.height > 14 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(0)])
            .split(area)
    };

    let columns = derive_columns(items, 6);
    let widths = columns
        .iter()
        .map(|_| Constraint::Percentage((100 / columns.len().max(1)) as u16))
        .collect::<Vec<_>>();

    let visible_rows = chunks[0].height.saturating_sub(4) as usize;
    let offset = if selected_row >= visible_rows && visible_rows > 0 {
        selected_row + 1 - visible_rows
    } else {
        0
    };
    let end = if visible_rows == 0 {
        items.len()
    } else {
        (offset + visible_rows).min(items.len())
    };
    let visible_items = &items[offset..end];

    let header_cells = columns
        .iter()
        .map(|column| Cell::from(column.clone()).style(Theme::section_heading()));
    let header = Row::new(header_cells);

    let rows = visible_items.iter().enumerate().map(|(local_index, item)| {
        let row_index = offset + local_index;
        let cells = columns
            .iter()
            .map(|column| Cell::from(compact_cell(item.get(column), 30)));
        let mut row = Row::new(cells);
        if row_index == selected_row {
            row = row.style(Theme::selected());
        }
        row
    });

    let title = format!(
        " items={} total={} has_more={} rows {}-{} ",
        items.len(),
        total
            .map(|count| count.to_string())
            .unwrap_or_else(|| "-".to_string()),
        has_more,
        offset + 1,
        end
    );
    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(title, Theme::muted()))
                .border_style(Style::default().fg(Theme::BORDER)),
        )
        .column_spacing(1);

    frame.render_widget(table, chunks[0]);

    if chunks[1].height == 0 {
        return;
    }

    let selected = selected_row.min(items.len().saturating_sub(1));
    let detail_lines = format_object_lines(&items[selected], chunks[1].height as usize);
    let detail = Paragraph::new(detail_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" selected ", Theme::muted()))
            .border_style(Style::default().fg(Theme::BORDER)),
    );
    frame.render_widget(detail, chunks[1]);
}

fn render_footer(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(60)])
        .split(area);

    let hints = Paragraph::new(Line::from(vec![
        Span::styled("tab ", Theme::section_heading()),
        Span::styled("focus ", Theme::muted()),
        Span::styled("cmd/ctrl+p ", Theme::section_heading()),
        Span::styled("palette ", Theme::muted()),
        Span::styled("enter ", Theme::section_heading()),
        Span::styled("open ", Theme::muted()),
        Span::styled("r ", Theme::section_heading()),
        Span::styled("refresh ", Theme::muted()),
        Span::styled("q ", Theme::section_heading()),
        Span::styled("quit", Theme::muted()),
    ]));
    frame.render_widget(hints, chunks[0]);

    let auth = if app.api.is_authenticated() {
        "ok"
    } else {
        "off"
    };
    let writes = if app.api.writes_allowed() {
        "on"
    } else {
        "off"
    };
    let status = format!(
        "auth:{auth} | writes:{writes} | route:{}/{} | {}",
        app.active_route + 1,
        app.routes.len(),
        app.status
    );
    let right = Paragraph::new(Span::styled(status, Theme::muted())).alignment(Alignment::Right);
    frame.render_widget(right, chunks[1]);
}

fn render_palette(frame: &mut Frame<'_>, app: &App) {
    let area = centered_rect(62, 60, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::ACCENT))
        .title(Span::styled(" Command Palette ", Theme::section_heading()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    let input = Paragraph::new(Line::from(vec![
        Span::styled("> ", Theme::section_heading()),
        Span::styled(app.palette.query.clone(), Theme::text()),
    ]));
    frame.render_widget(input, chunks[0]);

    let selected_source = app.palette_filtered.get(app.palette.selected).copied();
    let rows = app.palette_rows();
    let mut items = Vec::new();
    for row in rows {
        match row {
            PaletteRow::Section(section) => {
                items.push(ListItem::new(Line::from(Span::styled(
                    format!(" {} ", section.label()),
                    Theme::section_heading(),
                ))));
            }
            PaletteRow::Separator => {
                items.push(ListItem::new(Line::from(Span::styled(
                    "────────────────────────────",
                    Theme::muted(),
                ))));
            }
            PaletteRow::Action(index) => {
                let Some(action) = app.palette_actions.get(index) else {
                    continue;
                };
                let left = if let Some(reason) = &action.disabled_reason {
                    format!("{} (disabled: {reason})", action.title)
                } else {
                    action.title.clone()
                };
                let mut style = if action.disabled_reason.is_some() {
                    Theme::disabled()
                } else {
                    Theme::text()
                };
                if Some(index) == selected_source {
                    style = Theme::selected();
                }

                let line = if action.context.is_empty() {
                    Line::from(Span::styled(left, style))
                } else {
                    Line::from(vec![
                        Span::styled(left, style),
                        Span::styled("  ", style),
                        Span::styled(action.context.clone(), Theme::muted()),
                    ])
                };
                items.push(ListItem::new(line));
            }
        }
    }

    let list = List::new(items);
    frame.render_widget(list, chunks[1]);
}

fn render_prompt(frame: &mut Frame<'_>, app: &App) {
    let Some(prompt) = &app.prompt else {
        return;
    };

    let area = centered_rect(58, 24, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::ACCENT))
        .title(Span::styled(
            format!(" {} ", prompt.title),
            Theme::section_heading(),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Span::styled(prompt.hint.clone(), Theme::muted())),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("> ", Theme::section_heading()),
            Span::styled(prompt.value.clone(), Theme::text()),
        ])),
        chunks[1],
    );
    frame.render_widget(
        Paragraph::new(Span::styled("enter confirm | esc cancel", Theme::muted())),
        chunks[2],
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}

fn derive_columns(items: &[serde_json::Value], max_columns: usize) -> Vec<String> {
    let mut columns = Vec::<String>::new();
    let priority = [
        "name",
        "description",
        "contact_name",
        "organisation_name",
        "status",
        "dated_on",
        "due_on",
        "period_ends_on",
        "total_value",
        "net_value",
        "gross_value",
        "url",
    ];

    for key in priority {
        if items
            .iter()
            .any(|item| item.as_object().is_some_and(|map| map.contains_key(key)))
            && !columns.contains(&key.to_string())
        {
            columns.push(key.to_string());
        }
        if columns.len() >= max_columns {
            return columns;
        }
    }

    if let Some(object) = items.iter().find_map(serde_json::Value::as_object) {
        for key in object.keys() {
            if !columns.contains(key) {
                columns.push(key.clone());
            }
            if columns.len() >= max_columns {
                break;
            }
        }
    }

    if columns.is_empty() {
        columns.push("value".to_string());
    }

    columns
}

fn compact_cell(value: Option<&serde_json::Value>, max_len: usize) -> String {
    let raw = match value {
        Some(serde_json::Value::String(text)) => text.clone(),
        Some(serde_json::Value::Number(number)) => number.to_string(),
        Some(serde_json::Value::Bool(flag)) => flag.to_string(),
        Some(serde_json::Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    };

    if raw.chars().count() <= max_len {
        return raw;
    }

    let mut out = String::new();
    for (index, ch) in raw.chars().enumerate() {
        if index + 1 >= max_len.saturating_sub(1) {
            break;
        }
        out.push(ch);
    }
    out.push('…');
    out
}

fn format_object_lines(value: &serde_json::Value, max_lines: usize) -> Vec<Line<'static>> {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            let mut lines = Vec::new();
            for key in keys.into_iter().take(max_lines.saturating_sub(1)) {
                let cell = compact_cell(map.get(&key), 120);
                lines.push(Line::from(vec![
                    Span::styled(format!("{key}: "), Theme::section_heading()),
                    Span::styled(cell, Theme::text()),
                ]));
            }
            if map.len() > lines.len() {
                lines.push(Line::from(Span::styled(
                    format!("… {} more fields", map.len() - lines.len()),
                    Theme::muted(),
                )));
            }
            lines
        }
        serde_json::Value::Array(items) => {
            let mut lines = Vec::new();
            lines.push(Line::from(Span::styled(
                format!("Array[{}]", items.len()),
                Theme::section_heading(),
            )));
            for item in items.iter().take(max_lines.saturating_sub(2)) {
                lines.push(Line::from(Span::styled(
                    compact_cell(Some(item), 120),
                    Theme::text(),
                )));
            }
            lines
        }
        _ => vec![Line::from(Span::styled(
            compact_cell(Some(value), 180),
            Theme::text(),
        ))],
    }
}
