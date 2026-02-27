//! TUI launcher command.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use cho_sdk::error::{ChoSdkError, Result};

/// Launches the `cho-tui` binary and returns its exit code.
pub fn run() -> Result<i32> {
    let bin_name = if cfg!(windows) {
        "cho-tui.exe"
    } else {
        "cho-tui"
    };

    if let Some(path) = current_exe_sibling(bin_name) {
        return run_child(path);
    }

    run_child(PathBuf::from(bin_name)).map_err(|err| {
        if err.to_string().contains("No such file or directory")
            || err.to_string().contains("not found")
        {
            ChoSdkError::Config {
                message: "Could not locate 'cho-tui'. Build/install workspace binaries or ensure 'cho-tui' is on PATH.".to_string(),
            }
        } else {
            err
        }
    })
}

fn current_exe_sibling(bin_name: &str) -> Option<PathBuf> {
    let current = std::env::current_exe().ok()?;
    let dir = current.parent()?;
    let candidate = dir.join(bin_name);
    candidate.exists().then_some(candidate)
}

fn run_child(program: PathBuf) -> Result<i32> {
    let status = Command::new(&program)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|err| ChoSdkError::Config {
            message: format!("Failed launching {}: {err}", program.display()),
        })?;

    Ok(status.code().unwrap_or(1))
}
