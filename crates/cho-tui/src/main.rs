#![forbid(unsafe_code)]

//! `cho-tui`: terminal UI for FreeAgent data in `cho`.

mod api;
mod app;
mod config;
mod palette;
mod routes;
mod theme;
mod ui;

use std::io::{self, Stdout};

use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut terminal = setup_terminal()?;
    let mut app = app::App::new()?;
    let result = app.run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, String> {
    enable_raw_mode().map_err(|e| format!("failed to enable raw mode: {e}"))?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        )
    )
    .map_err(|e| format!("failed to enter alternate screen: {e}"))?;

    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(|e| format!("failed to initialize terminal backend: {e}"))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), String> {
    disable_raw_mode().map_err(|e| format!("failed to disable raw mode: {e}"))?;
    execute!(
        terminal.backend_mut(),
        PopKeyboardEnhancementFlags,
        LeaveAlternateScreen
    )
    .map_err(|e| format!("failed to restore terminal: {e}"))?;
    terminal
        .show_cursor()
        .map_err(|e| format!("failed to restore cursor: {e}"))?;
    Ok(())
}
