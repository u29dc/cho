//! Auth commands: login, status, refresh, tenants.

use std::time::Instant;

use clap::Subcommand;

use crate::context::CliContext;

/// Auth subcommands.
#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    /// Login via OAuth 2.0 PKCE flow (opens browser).
    Login {
        /// Use client credentials (Custom Connections) instead of PKCE.
        #[arg(long)]
        client_credentials: bool,

        /// Port for localhost callback server (0 = auto).
        #[arg(long, default_value = "0")]
        port: u16,
    },
    /// Show current authentication status.
    Status,
    /// Force token refresh.
    Refresh,
    /// List connected organisations (tenants).
    Tenants,
}

/// Returns the tool name for an auth subcommand.
pub fn tool_name(cmd: &AuthCommands) -> &'static str {
    match cmd {
        AuthCommands::Login { .. } => "auth.login",
        AuthCommands::Status => "auth.status",
        AuthCommands::Refresh => "auth.refresh",
        AuthCommands::Tenants => "auth.tenants",
    }
}

/// Runs an auth subcommand.
pub async fn run(
    cmd: &AuthCommands,
    ctx: &CliContext,
    start: Instant,
) -> cho_sdk::error::Result<()> {
    match cmd {
        AuthCommands::Login {
            client_credentials,
            port,
        } => {
            if *client_credentials {
                let secret = std::env::var("CHO_CLIENT_SECRET").map_err(|_| {
                    cho_sdk::error::ChoSdkError::Config {
                        message:
                            "CHO_CLIENT_SECRET environment variable required for client credentials"
                                .to_string(),
                    }
                })?;
                ctx.client()
                    .auth()
                    .login_client_credentials(secrecy::SecretString::from(secret))
                    .await?;
                eprintln!("Authenticated via client credentials.");
            } else {
                ctx.client().auth().login_pkce(*port).await?;
                eprintln!("Login successful!");
            }

            // List tenants after login
            let connections = ctx.client().identity().connections().await?;
            eprintln!("Connected organisations:");
            for conn in &connections {
                eprintln!(
                    "  {} ({})",
                    conn.tenant_name.as_deref().unwrap_or("Unknown"),
                    conn.tenant_id.map(|id| id.to_string()).unwrap_or_default()
                );
            }

            ctx.emit_success(
                "auth.login",
                &serde_json::json!({"authenticated": true}),
                start,
            )?;
            Ok(())
        }
        AuthCommands::Status => {
            let authenticated = ctx.client().auth().is_authenticated().await;
            if authenticated {
                eprintln!("Authenticated: yes");
                eprintln!("Tenant ID: {}", ctx.client().tenant_id());
            } else {
                eprintln!("Authenticated: no");
                eprintln!("Run 'cho auth login' to authenticate.");
            }
            ctx.emit_success(
                "auth.status",
                &serde_json::json!({
                    "authenticated": authenticated,
                    "tenant_id": ctx.client().tenant_id(),
                }),
                start,
            )?;
            Ok(())
        }
        AuthCommands::Refresh => {
            ctx.client().auth().refresh().await?;
            eprintln!("Token refreshed successfully.");
            ctx.emit_success(
                "auth.refresh",
                &serde_json::json!({"refreshed": true}),
                start,
            )?;
            Ok(())
        }
        AuthCommands::Tenants => {
            let connections = ctx.client().identity().connections().await?;
            ctx.emit_items("auth.tenants", &connections, start)?;
            Ok(())
        }
    }
}
