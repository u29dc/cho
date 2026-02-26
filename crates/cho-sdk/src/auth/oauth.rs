//! OAuth helpers for browser-based authorization-code flow.

use std::io::ErrorKind;

use rand::Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use url::Url;

use crate::error::{ChoSdkError, Result};

/// Captured authorization callback payload.
#[derive(Debug, Clone)]
pub struct OAuthCallback {
    /// Authorization code.
    pub code: String,
    /// Echoed state.
    pub state: Option<String>,
}

/// Starts callback listener and returns `(listener, redirect_uri)`.
pub async fn start_callback_listener(port: u16) -> Result<(TcpListener, String)> {
    let bind_addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| ChoSdkError::Config {
            message: format!("Failed binding OAuth callback listener on {bind_addr}: {e}"),
        })?;

    let addr = listener.local_addr().map_err(|e| ChoSdkError::Config {
        message: format!("Failed reading OAuth callback listener address: {e}"),
    })?;

    let redirect_uri = format!("http://127.0.0.1:{}/callback", addr.port());
    Ok((listener, redirect_uri))
}

/// Creates a cryptographically random `state` value.
pub fn random_state() -> String {
    const ALPHANUM: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..40)
        .map(|_| ALPHANUM[rng.random_range(0..ALPHANUM.len())] as char)
        .collect()
}

/// Builds authorization URL.
pub fn authorization_url(
    authorize_url: &str,
    client_id: &str,
    redirect_uri: &str,
    state: &str,
) -> Result<Url> {
    let mut url = Url::parse(authorize_url).map_err(|e| ChoSdkError::Config {
        message: format!("Invalid authorize URL '{authorize_url}': {e}"),
    })?;

    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("state", state);

    Ok(url)
}

/// Waits for OAuth callback and extracts code/state.
pub async fn receive_callback(listener: TcpListener, timeout_secs: u64) -> Result<OAuthCallback> {
    let accept_fut = listener.accept();
    let (mut socket, _addr) =
        tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), accept_fut)
            .await
            .map_err(|_| ChoSdkError::AuthRequired {
                message: "OAuth callback timed out. Retry 'cho auth login'.".to_string(),
            })?
            .map_err(|e| ChoSdkError::Config {
                message: format!("OAuth callback listener failed: {e}"),
            })?;

    let mut buffer = [0_u8; 8192];
    let read = socket
        .read(&mut buffer)
        .await
        .map_err(|e| ChoSdkError::Config {
            message: format!("Failed reading OAuth callback request: {e}"),
        })?;

    if read == 0 {
        return Err(ChoSdkError::AuthRequired {
            message: "OAuth callback request was empty".to_string(),
        });
    }

    let request = String::from_utf8_lossy(&buffer[..read]).to_string();
    let first_line = request.lines().next().unwrap_or_default();

    let path = first_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| ChoSdkError::Parse {
            message: format!("Failed parsing callback request line: {first_line}"),
        })?;

    let callback_url =
        Url::parse(&format!("http://localhost{path}")).map_err(|e| ChoSdkError::Parse {
            message: format!("Failed parsing callback URL path '{path}': {e}"),
        })?;

    let mut code: Option<String> = None;
    let mut state: Option<String> = None;
    let mut error: Option<String> = None;

    for (key, value) in callback_url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            "error" => error = Some(value.into_owned()),
            _ => {}
        }
    }

    let response_body = if let Some(ref oauth_error) = error {
        format!("OAuth failed: {oauth_error}. You can close this tab.")
    } else {
        "Authentication complete. You can close this tab and return to your terminal.".to_string()
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response_body.len(),
        response_body
    );

    if let Err(e) = socket.write_all(response.as_bytes()).await
        && e.kind() != ErrorKind::BrokenPipe
    {
        tracing::debug!("failed to write OAuth callback response: {e}");
    }

    if let Some(oauth_error) = error {
        return Err(ChoSdkError::AuthRequired {
            message: format!("FreeAgent authorization failed: {oauth_error}"),
        });
    }

    let code = code.ok_or_else(|| ChoSdkError::AuthRequired {
        message: "OAuth callback did not include an authorization code".to_string(),
    })?;

    Ok(OAuthCallback { code, state })
}
