#![forbid(unsafe_code)]

//! `cho-tui`: terminal UI for FreeAgent data in `cho`.

mod api;
mod app;
mod cache;
mod config;
mod fetch;
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
    let mut app = app::App::new()?;
    let mut session = TerminalSession::enter()?;
    let run_result = app.run(session.terminal_mut());
    let restore_result = session.restore();
    merge_run_and_restore_result(run_result, restore_result)
}

struct TerminalSession {
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
    state: TerminalState,
}

#[derive(Debug, Default, Clone, Copy)]
struct TerminalState {
    raw_mode_enabled: bool,
    alternate_screen_entered: bool,
    keyboard_flags_pushed: bool,
    cursor_restore_needed: bool,
    restored: bool,
}

impl TerminalSession {
    fn enter() -> Result<Self, String> {
        let mut session = Self {
            terminal: None,
            state: TerminalState::default(),
        };
        let setup_result = (|| -> Result<(), String> {
            enable_raw_mode().map_err(|e| format!("failed to enable raw mode: {e}"))?;
            session.state.raw_mode_enabled = true;

            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen)
                .map_err(|e| format!("failed to enter alternate screen: {e}"))?;
            session.state.alternate_screen_entered = true;

            execute!(
                stdout,
                PushKeyboardEnhancementFlags(keyboard_enhancement_flags())
            )
            .map_err(|e| format!("failed to enable keyboard enhancement flags: {e}"))?;
            session.state.keyboard_flags_pushed = true;

            let backend = CrosstermBackend::new(stdout);
            let terminal = Terminal::new(backend)
                .map_err(|e| format!("failed to initialize terminal backend: {e}"))?;
            session.terminal = Some(terminal);
            session.state.cursor_restore_needed = true;
            Ok(())
        })();

        if let Err(err) = setup_result {
            let restore_result = session.restore();
            return Err(match restore_result {
                Ok(()) => err,
                Err(restore_err) => append_secondary_error(
                    err,
                    "additionally failed to restore terminal after setup failure",
                    restore_err,
                ),
            });
        }

        Ok(session)
    }

    fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        self.terminal
            .as_mut()
            .expect("terminal session should own a terminal after successful setup")
    }

    fn restore(&mut self) -> Result<(), String> {
        let mut cleanup = StdTerminalCleanupBackend {
            terminal: self.terminal.as_mut(),
        };
        cleanup_terminal_state(&mut self.state, &mut cleanup)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

trait CleanupBackend {
    fn disable_raw_mode(&mut self) -> Result<(), String>;
    fn pop_keyboard_flags(&mut self) -> Result<(), String>;
    fn leave_alternate_screen(&mut self) -> Result<(), String>;
    fn show_cursor(&mut self) -> Result<(), String>;
}

struct StdTerminalCleanupBackend<'a> {
    terminal: Option<&'a mut Terminal<CrosstermBackend<Stdout>>>,
}

impl CleanupBackend for StdTerminalCleanupBackend<'_> {
    fn disable_raw_mode(&mut self) -> Result<(), String> {
        disable_raw_mode().map_err(|e| format!("failed to disable raw mode: {e}"))
    }

    fn pop_keyboard_flags(&mut self) -> Result<(), String> {
        let mut stdout = io::stdout();
        execute!(stdout, PopKeyboardEnhancementFlags)
            .map_err(|e| format!("failed to pop keyboard enhancement flags: {e}"))
    }

    fn leave_alternate_screen(&mut self) -> Result<(), String> {
        let mut stdout = io::stdout();
        execute!(stdout, LeaveAlternateScreen)
            .map_err(|e| format!("failed to leave alternate screen: {e}"))
    }

    fn show_cursor(&mut self) -> Result<(), String> {
        match self.terminal.as_mut() {
            Some(terminal) => terminal
                .show_cursor()
                .map_err(|e| format!("failed to restore cursor: {e}")),
            None => Ok(()),
        }
    }
}

fn keyboard_enhancement_flags() -> KeyboardEnhancementFlags {
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
        | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
}

fn cleanup_terminal_state<B: CleanupBackend>(
    state: &mut TerminalState,
    backend: &mut B,
) -> Result<(), String> {
    if state.restored {
        return Ok(());
    }
    state.restored = true;

    let mut failures = Vec::new();

    if state.raw_mode_enabled {
        if let Err(err) = backend.disable_raw_mode() {
            failures.push(err);
        }
        state.raw_mode_enabled = false;
    }

    if state.keyboard_flags_pushed {
        if let Err(err) = backend.pop_keyboard_flags() {
            failures.push(err);
        }
        state.keyboard_flags_pushed = false;
    }

    if state.alternate_screen_entered {
        if let Err(err) = backend.leave_alternate_screen() {
            failures.push(err);
        }
        state.alternate_screen_entered = false;
    }

    if state.cursor_restore_needed {
        if let Err(err) = backend.show_cursor() {
            failures.push(err);
        }
        state.cursor_restore_needed = false;
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join(" | "))
    }
}

fn merge_run_and_restore_result(
    run_result: Result<(), String>,
    restore_result: Result<(), String>,
) -> Result<(), String> {
    match (run_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(err), Ok(())) => Err(err),
        (Ok(()), Err(restore_err)) => Err(format!("failed to restore terminal: {restore_err}")),
        (Err(err), Err(restore_err)) => Err(append_secondary_error(
            err,
            "additionally failed to restore terminal",
            restore_err,
        )),
    }
}

fn append_secondary_error(primary: String, context: &str, secondary: String) -> String {
    format!("{primary} | {context}: {secondary}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockCleanupBackend {
        calls: Vec<&'static str>,
        disable_raw_mode_error: Option<String>,
        pop_keyboard_flags_error: Option<String>,
        leave_alternate_screen_error: Option<String>,
        show_cursor_error: Option<String>,
    }

    impl CleanupBackend for MockCleanupBackend {
        fn disable_raw_mode(&mut self) -> Result<(), String> {
            self.calls.push("disable_raw_mode");
            self.disable_raw_mode_error.clone().map_or(Ok(()), Err)
        }

        fn pop_keyboard_flags(&mut self) -> Result<(), String> {
            self.calls.push("pop_keyboard_flags");
            self.pop_keyboard_flags_error.clone().map_or(Ok(()), Err)
        }

        fn leave_alternate_screen(&mut self) -> Result<(), String> {
            self.calls.push("leave_alternate_screen");
            self.leave_alternate_screen_error
                .clone()
                .map_or(Ok(()), Err)
        }

        fn show_cursor(&mut self) -> Result<(), String> {
            self.calls.push("show_cursor");
            self.show_cursor_error.clone().map_or(Ok(()), Err)
        }
    }

    #[test]
    fn cleanup_terminal_state_attempts_all_enabled_steps() {
        let mut state = TerminalState {
            raw_mode_enabled: true,
            alternate_screen_entered: true,
            keyboard_flags_pushed: true,
            cursor_restore_needed: true,
            restored: false,
        };
        let mut backend = MockCleanupBackend {
            disable_raw_mode_error: Some("raw".to_string()),
            leave_alternate_screen_error: Some("screen".to_string()),
            ..Default::default()
        };

        let err = cleanup_terminal_state(&mut state, &mut backend)
            .expect_err("cleanup should surface aggregated failures");

        assert_eq!(
            backend.calls,
            vec![
                "disable_raw_mode",
                "pop_keyboard_flags",
                "leave_alternate_screen",
                "show_cursor"
            ]
        );
        assert!(err.contains("raw"));
        assert!(err.contains("screen"));
        assert!(state.restored);
        assert!(!state.raw_mode_enabled);
        assert!(!state.keyboard_flags_pushed);
        assert!(!state.alternate_screen_entered);
        assert!(!state.cursor_restore_needed);
    }

    #[test]
    fn cleanup_terminal_state_is_idempotent_after_restore() {
        let mut state = TerminalState {
            raw_mode_enabled: true,
            ..TerminalState::default()
        };
        let mut backend = MockCleanupBackend::default();

        cleanup_terminal_state(&mut state, &mut backend).expect("cleanup should succeed");
        backend.calls.clear();
        cleanup_terminal_state(&mut state, &mut backend).expect("second cleanup should no-op");

        assert!(backend.calls.is_empty());
    }

    #[test]
    fn merge_run_and_restore_result_preserves_run_error() {
        let result = merge_run_and_restore_result(
            Err("run failed".to_string()),
            Err("cursor restore failed".to_string()),
        )
        .expect_err("combined error should fail");

        assert_eq!(
            result,
            "run failed | additionally failed to restore terminal: cursor restore failed"
        );
    }

    #[test]
    fn merge_run_and_restore_result_surfaces_restore_error_when_run_succeeds() {
        let result = merge_run_and_restore_result(Ok(()), Err("restore failed".to_string()))
            .expect_err("restore failure should be surfaced");

        assert_eq!(result, "failed to restore terminal: restore failed");
    }
}
