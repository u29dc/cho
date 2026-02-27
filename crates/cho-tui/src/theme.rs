//! Shared visual tokens for `cho-tui`.

use ratatui::style::{Color, Modifier, Style};

/// Color and style tokens.
#[derive(Debug, Clone, Copy)]
pub struct Theme;

impl Theme {
    /// Accent color.
    pub const ACCENT: Color = Color::Cyan;
    /// Primary text.
    pub const FG: Color = Color::Gray;
    /// Muted text.
    pub const MUTED: Color = Color::DarkGray;
    /// Border color.
    pub const BORDER: Color = Color::DarkGray;

    /// Header brand style.
    pub fn brand() -> Style {
        Style::default()
            .fg(Self::ACCENT)
            .add_modifier(Modifier::BOLD)
    }

    /// Header muted style.
    pub fn header_meta() -> Style {
        Style::default().fg(Self::MUTED)
    }

    /// Default text style.
    pub fn text() -> Style {
        Style::default().fg(Self::FG)
    }

    /// Muted text style.
    pub fn muted() -> Style {
        Style::default().fg(Self::MUTED)
    }

    /// Highlight style for selected rows.
    pub fn selected() -> Style {
        Style::default()
            .bg(Self::ACCENT)
            .fg(Color::Rgb(255, 255, 255))
            .add_modifier(Modifier::BOLD)
    }

    /// Section heading style.
    pub fn section_heading() -> Style {
        Style::default()
            .fg(Self::ACCENT)
            .add_modifier(Modifier::BOLD)
    }

    /// Disabled command style.
    pub fn disabled() -> Style {
        Style::default().fg(Self::MUTED)
    }
}
