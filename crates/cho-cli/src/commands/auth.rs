//! Auth commands.

use std::time::Instant;

use cho_sdk::error::Result;
use clap::Subcommand;

use crate::context::CliContext;

/// Auth subcommands.
#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    /// Run OAuth login flow.
    Login {
        /// Callback port (0 = auto).
        #[arg(long, default_value = "0")]
        port: u16,
        /// Do not automatically open browser.
        #[arg(long)]
        no_browser: bool,
    },
    /// Show auth status.
    Status,
    /// Refresh current token.
    Refresh,
    /// Logout and clear stored tokens.
    Logout,
}

/// Tool name for auth subcommand.
pub fn tool_name(command: &AuthCommands) -> &'static str {
    match command {
        AuthCommands::Login { .. } => "auth.login",
        AuthCommands::Status => "auth.status",
        AuthCommands::Refresh => "auth.refresh",
        AuthCommands::Logout => "auth.logout",
    }
}

/// Runs auth command.
pub async fn run(command: &AuthCommands, ctx: &CliContext, start: Instant) -> Result<()> {
    match command {
        AuthCommands::Login { port, no_browser } => {
            let result = ctx
                .client()
                .auth()
                .login_browser(*port, !*no_browser)
                .await?;
            let payload = serde_json::json!({
                "authenticated": true,
                "authorize_url": result.authorize_url,
                "redirect_uri": result.redirect_uri,
            });
            ctx.emit_success("auth.login", &payload, start)
        }
        AuthCommands::Status => {
            let _ = ctx.client().auth().load_stored_tokens().await;
            let status = ctx.client().auth().status().await;
            ctx.emit_success("auth.status", &status, start)
        }
        AuthCommands::Refresh => {
            ctx.client().auth().refresh().await?;
            let payload = serde_json::json!({ "refreshed": true });
            ctx.emit_success("auth.refresh", &payload, start)
        }
        AuthCommands::Logout => {
            ctx.client().auth().logout().await?;
            let payload = serde_json::json!({ "authenticated": false });
            ctx.emit_success("auth.logout", &payload, start)
        }
    }
}
