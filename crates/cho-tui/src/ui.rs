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
        Span::styled("pgup/pgdn ", Theme::section_heading()),
        Span::styled("jump ", Theme::muted()),
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
    let area = centered_rect_with_min(50, 50, 56, 16, frame.area());
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
    let selected_visual = selected_source.and_then(|selected| {
        rows.iter().position(|row| match row {
            PaletteRow::Action(index) => *index == selected,
            _ => false,
        })
    });

    let visible_rows = chunks[1].height.max(1) as usize;
    let offset = if let Some(selected_index) = selected_visual {
        if selected_index >= visible_rows {
            selected_index + 1 - visible_rows
        } else {
            0
        }
    } else {
        0
    };
    let end = (offset + visible_rows).min(rows.len());
    let visible_rows = &rows[offset..end];

    let mut items = Vec::new();
    for row in visible_rows {
        match row {
            PaletteRow::Section(section) => {
                items.push(ListItem::new(Line::from(Span::styled(
                    format!("{}:", section.label()),
                    Theme::section_heading(),
                ))));
            }
            PaletteRow::Separator => {
                let line = "─".repeat(chunks[1].width.saturating_sub(1) as usize);
                items.push(ListItem::new(Line::from(Span::styled(
                    line,
                    Theme::muted(),
                ))));
            }
            PaletteRow::Action(index) => {
                let Some(action) = app.palette_actions.get(*index) else {
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
                if Some(*index) == selected_source {
                    style = Theme::selected();
                }

                let line = if action.context.is_empty() {
                    Line::from(Span::styled(left, style))
                } else {
                    let context_style = if Some(*index) == selected_source {
                        style
                    } else {
                        Theme::muted()
                    };
                    palette_two_column_line(
                        &left,
                        &action.context,
                        style,
                        context_style,
                        chunks[1].width.saturating_sub(1) as usize,
                    )
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

    let area = if prompt.options.is_empty() {
        centered_rect_with_min(58, 24, 58, 8, frame.area())
    } else {
        centered_rect_with_min(62, 42, 64, 16, frame.area())
    };
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
            Constraint::Min(1),
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

    if prompt.options.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No suggestions available for this prompt",
                Theme::muted(),
            )),
            chunks[2],
        );
    } else {
        let visible_rows = chunks[2].height.max(1) as usize;
        let selected = prompt
            .selected_option
            .min(prompt.options.len().saturating_sub(1));
        let offset = if selected >= visible_rows {
            selected + 1 - visible_rows
        } else {
            0
        };
        let end = (offset + visible_rows).min(prompt.options.len());

        let mut items = Vec::new();
        for (local_index, option) in prompt.options[offset..end].iter().enumerate() {
            let index = offset + local_index;
            let style = if index == selected {
                Theme::selected()
            } else {
                Theme::text()
            };
            let line = if option.meta.is_empty() {
                Line::from(Span::styled(option.label.clone(), style))
            } else {
                let meta_style = if index == selected {
                    style
                } else {
                    Theme::muted()
                };
                palette_two_column_line(
                    &option.label,
                    &option.meta,
                    style,
                    meta_style,
                    chunks[2].width.saturating_sub(1) as usize,
                )
            };
            items.push(ListItem::new(line));
        }
        frame.render_widget(List::new(items), chunks[2]);
    }

    frame.render_widget(
        Paragraph::new(Span::styled(
            "up/down select | type jump | enter confirm | esc cancel",
            Theme::muted(),
        )),
        chunks[3],
    );
}

fn centered_rect_with_min(
    percent_x: u16,
    percent_y: u16,
    min_width: u16,
    min_height: u16,
    area: Rect,
) -> Rect {
    let desired_width = ((area.width as u32 * percent_x as u32) / 100) as u16;
    let desired_height = ((area.height as u32 * percent_y as u32) / 100) as u16;

    let width = desired_width.max(min_width.min(area.width)).min(area.width);
    let height = desired_height
        .max(min_height.min(area.height))
        .min(area.height);

    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;

    Rect {
        x,
        y,
        width,
        height,
    }
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

    truncate_with_ellipsis(&raw, max_len)
}

fn palette_two_column_line(
    left: &str,
    right: &str,
    left_style: Style,
    right_style: Style,
    width: usize,
) -> Line<'static> {
    if right.is_empty() || width == 0 {
        return Line::from(Span::styled(left.to_string(), left_style));
    }

    let min_gap = 2usize;
    let right_max = width.saturating_sub(min_gap + 1).max(1);
    let right_text = truncate_with_ellipsis(right, right_max);
    let right_width = right_text.chars().count();
    let left_max = width.saturating_sub(right_width + min_gap).max(1);
    let left_text = truncate_with_ellipsis(left, left_max);
    let left_width = left_text.chars().count();
    let gap = width.saturating_sub(left_width + right_width).max(min_gap);

    Line::from(vec![
        Span::styled(left_text, left_style),
        Span::styled(" ".repeat(gap), left_style),
        Span::styled(right_text, right_style),
    ])
}

fn truncate_with_ellipsis(text: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }
    if text.chars().count() <= max_len {
        return text.to_string();
    }
    if max_len == 1 {
        return "…".to_string();
    }

    let mut out = String::new();
    for (index, ch) in text.chars().enumerate() {
        if index + 1 >= max_len {
            break;
        }
        out.push(ch);
    }
    out.push('…');
    out
}

fn format_object_lines(value: &serde_json::Value, max_lines: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if max_lines == 0 {
        return lines;
    }

    let mut truncated = false;
    write_value_block(&mut lines, None, value, 0, max_lines, &mut truncated);

    if truncated {
        if lines.len() >= max_lines {
            lines.pop();
        }
        lines.push(Line::from(Span::styled(
            "… output truncated",
            Theme::muted(),
        )));
    }

    lines
}

fn write_value_block(
    lines: &mut Vec<Line<'static>>,
    key: Option<&str>,
    value: &serde_json::Value,
    indent: usize,
    max_lines: usize,
    truncated: &mut bool,
) {
    match value {
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                if let Some(key) = key {
                    push_key_value_line(lines, key, "{}", indent, max_lines, truncated);
                } else {
                    push_plain_line(lines, "{}", Theme::text(), indent, max_lines, truncated);
                }
                return;
            }

            let child_indent = if let Some(key) = key {
                push_key_line(lines, key, indent, max_lines, truncated);
                indent + 2
            } else {
                indent
            };
            if *truncated {
                return;
            }

            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            for child_key in keys {
                let Some(child_value) = map.get(&child_key) else {
                    continue;
                };
                write_value_block(
                    lines,
                    Some(&child_key),
                    child_value,
                    child_indent,
                    max_lines,
                    truncated,
                );
                if *truncated {
                    return;
                }
            }
        }
        serde_json::Value::Array(items) => {
            if items.is_empty() {
                if let Some(key) = key {
                    push_key_value_line(lines, key, "[]", indent, max_lines, truncated);
                } else {
                    push_plain_line(lines, "[]", Theme::text(), indent, max_lines, truncated);
                }
                return;
            }

            let header = format!("[{}]", items.len());
            if let Some(key) = key {
                push_key_value_line(lines, key, &header, indent, max_lines, truncated);
            } else {
                push_plain_line(lines, &header, Theme::text(), indent, max_lines, truncated);
            }
            if *truncated {
                return;
            }

            for item in items {
                match item {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        push_plain_line(
                            lines,
                            "-",
                            Theme::muted(),
                            indent + 2,
                            max_lines,
                            truncated,
                        );
                        if *truncated {
                            return;
                        }
                        write_value_block(lines, None, item, indent + 4, max_lines, truncated);
                    }
                    _ => {
                        let scalar = scalar_to_text(item);
                        push_plain_line(
                            lines,
                            &format!("- {scalar}"),
                            Theme::text(),
                            indent + 2,
                            max_lines,
                            truncated,
                        );
                    }
                }
                if *truncated {
                    return;
                }
            }
        }
        _ => {
            let scalar = scalar_to_text(value);
            if let Some(key) = key {
                push_key_value_line(lines, key, &scalar, indent, max_lines, truncated);
            } else {
                push_plain_line(lines, &scalar, Theme::text(), indent, max_lines, truncated);
            }
        }
    }
}

fn push_key_line(
    lines: &mut Vec<Line<'static>>,
    key: &str,
    indent: usize,
    max_lines: usize,
    truncated: &mut bool,
) {
    let line = Line::from(vec![
        Span::raw(" ".repeat(indent)),
        Span::styled(format!("{key}:"), Theme::section_heading()),
    ]);
    push_line(lines, line, max_lines, truncated);
}

fn push_key_value_line(
    lines: &mut Vec<Line<'static>>,
    key: &str,
    value: &str,
    indent: usize,
    max_lines: usize,
    truncated: &mut bool,
) {
    let line = Line::from(vec![
        Span::raw(" ".repeat(indent)),
        Span::styled(format!("{key}: "), Theme::section_heading()),
        Span::styled(value.to_string(), Theme::text()),
    ]);
    push_line(lines, line, max_lines, truncated);
}

fn push_plain_line(
    lines: &mut Vec<Line<'static>>,
    text: &str,
    style: Style,
    indent: usize,
    max_lines: usize,
    truncated: &mut bool,
) {
    let line = Line::from(vec![
        Span::raw(" ".repeat(indent)),
        Span::styled(text.to_string(), style),
    ]);
    push_line(lines, line, max_lines, truncated);
}

fn push_line(
    lines: &mut Vec<Line<'static>>,
    line: Line<'static>,
    max_lines: usize,
    truncated: &mut bool,
) {
    if lines.len() >= max_lines {
        *truncated = true;
        return;
    }
    lines.push(line);
}

fn scalar_to_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Number(number) => number.to_string(),
        serde_json::Value::Bool(flag) => flag.to_string(),
        serde_json::Value::Null => "null".to_string(),
        other => compact_cell(Some(other), 180),
    }
}
