//! OAuth 2.0 PKCE (Proof Key for Code Exchange) flow for Xero.
//!
//! Implements the authorization code flow with PKCE as required by Xero's
//! OAuth 2.0 for public clients (no client secret). Opens the user's browser,
//! starts a localhost callback server, and exchanges the authorization code
//! for tokens.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::Rng;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tracing::{debug, info};

use super::token::{TOKEN_ENDPOINT, TokenResponse};

/// Xero authorization endpoint.
const AUTHORIZE_URL: &str = "https://login.xero.com/identity/connect/authorize";

/// Default scopes for cho PKCE flow.
const DEFAULT_SCOPES: &str = "openid offline_access accounting.transactions.read accounting.contacts.read accounting.settings.read accounting.reports.read accounting.journals.read files.read assets.read projects.read payroll.employees payroll.timesheets payroll.settings";

/// PKCE code verifier length (43-128 characters per RFC 7636).
const VERIFIER_LENGTH: usize = 64;

/// Characters allowed in the PKCE code verifier (unreserved URI characters).
const VERIFIER_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

/// A PKCE code verifier and its derived challenge.
#[derive(Debug)]
pub struct PkceChallenge {
    /// The code verifier (random string, sent during token exchange).
    pub verifier: String,

    /// The code challenge (S256 hash of verifier, sent during authorization).
    pub challenge: String,
}

impl Default for PkceChallenge {
    fn default() -> Self {
        Self::new()
    }
}

impl PkceChallenge {
    /// Generates a new random PKCE challenge pair.
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let verifier: String = (0..VERIFIER_LENGTH)
            .map(|_| {
                let idx = rng.random_range(0..VERIFIER_CHARS.len());
                VERIFIER_CHARS[idx] as char
            })
            .collect();

        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        let challenge = URL_SAFE_NO_PAD.encode(hash);

        Self {
            verifier,
            challenge,
        }
    }
}

/// Parameters for the PKCE authorization flow.
pub struct PkceFlowParams {
    /// Xero OAuth 2.0 client ID.
    pub client_id: String,

    /// Port for the localhost callback server (0 = auto-assign).
    pub port: u16,

    /// OAuth scopes to request.
    pub scopes: Option<String>,
}

/// Generates a random state string for CSRF protection (RFC 6749 §10.12).
fn generate_state() -> String {
    let mut rng = rand::rng();
    (0..32)
        .map(|_| {
            let idx = rng.random_range(0..VERIFIER_CHARS.len());
            VERIFIER_CHARS[idx] as char
        })
        .collect()
}

/// Runs the full PKCE authorization flow:
/// 1. Generates PKCE verifier/challenge
/// 2. Starts localhost callback server
/// 3. Opens browser to Xero authorization URL
/// 4. Waits for callback with authorization code
/// 5. Exchanges code for tokens
pub(crate) async fn run_pkce_flow(params: &PkceFlowParams) -> crate::error::Result<TokenResponse> {
    let pkce = PkceChallenge::new();
    let state = generate_state();

    // Start callback server
    let listener = TcpListener::bind(format!("127.0.0.1:{}", params.port))
        .await
        .map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Failed to start callback server: {e}"),
        })?;

    let port = listener
        .local_addr()
        .map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Failed to get callback server address: {e}"),
        })?
        .port();

    let redirect_uri = format!("http://localhost:{port}/callback");
    let scopes = params.scopes.as_deref().unwrap_or(DEFAULT_SCOPES);

    // Build authorization URL with state parameter for CSRF protection
    let auth_url = format!(
        "{AUTHORIZE_URL}?response_type=code&client_id={}&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&state={}",
        urlencoded(&params.client_id),
        urlencoded(&redirect_uri),
        urlencoded(scopes),
        urlencoded(&pkce.challenge),
        urlencoded(&state),
    );

    info!("Opening browser for Xero authorization...");
    debug!("Authorization URL: {auth_url}");

    // Open browser
    if let Err(e) = open::that(&auth_url) {
        // Don't fail — user can copy URL manually
        info!("Could not open browser automatically: {e}");
        info!("Please open this URL in your browser:");
        info!("{auth_url}");
    }

    // Wait for callback and verify state parameter
    let code = wait_for_callback(listener, &state).await?;
    debug!("Received authorization code");

    // Exchange code for tokens
    let client = reqwest::Client::new();
    exchange_code(
        &client,
        &params.client_id,
        &code,
        &redirect_uri,
        &pkce.verifier,
    )
    .await
}

/// Waits for the OAuth callback on the localhost server, verifies the state
/// parameter for CSRF protection, and extracts the authorization code.
async fn wait_for_callback(
    listener: TcpListener,
    expected_state: &str,
) -> crate::error::Result<String> {
    let (stream, _addr) =
        listener
            .accept()
            .await
            .map_err(|e| crate::error::ChoSdkError::Config {
                message: format!("Callback server accept failed: {e}"),
            })?;

    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .await
        .map_err(|e| crate::error::ChoSdkError::Config {
            message: format!("Failed to read callback request: {e}"),
        })?;

    // Parse code and state from: GET /callback?code=XXXX&state=YYYY HTTP/1.1
    let (code, returned_state) = parse_code_and_state_from_request(&request_line)?;

    // Verify state parameter to prevent CSRF attacks (RFC 6749 §10.12)
    if returned_state.as_deref() != Some(expected_state) {
        // Send error response to browser before returning error
        let error_body = "<!DOCTYPE html><html><body><h2>Authorization failed</h2><p>State parameter mismatch — possible CSRF attack.</p></body></html>";
        let error_response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            error_body.len(),
            error_body
        );
        let stream = reader.into_inner();
        let (_, mut writer) = tokio::io::split(stream);
        let _ = writer.write_all(error_response.as_bytes()).await;
        let _ = writer.shutdown().await;

        return Err(crate::error::ChoSdkError::AuthRequired {
            message: "OAuth state parameter mismatch — possible CSRF attack".to_string(),
        });
    }

    // Send response to browser
    let response_body = "<!DOCTYPE html><html><body><h2>Authorization successful!</h2><p>You can close this tab and return to the terminal.</p></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response_body.len(),
        response_body
    );

    let stream = reader.into_inner();
    let (_, mut writer) = tokio::io::split(stream);
    let _ = writer.write_all(response.as_bytes()).await;
    let _ = writer.shutdown().await;

    Ok(code)
}

/// Extracts the authorization code and state parameter from an HTTP request line.
fn parse_code_and_state_from_request(
    request_line: &str,
) -> crate::error::Result<(String, Option<String>)> {
    // Expected format: GET /callback?code=XXXX&state=YYYY HTTP/1.1
    let path = request_line.split_whitespace().nth(1).ok_or_else(|| {
        crate::error::ChoSdkError::Config {
            message: "Invalid callback request".to_string(),
        }
    })?;

    // Check for error response
    if path.contains("error=") {
        let error = extract_query_param(path, "error").unwrap_or("unknown".to_string());
        let description = extract_query_param(path, "error_description").unwrap_or_default();
        return Err(crate::error::ChoSdkError::AuthRequired {
            message: format!("Authorization denied: {error} - {description}"),
        });
    }

    let code =
        extract_query_param(path, "code").ok_or_else(|| crate::error::ChoSdkError::Config {
            message: "No authorization code in callback".to_string(),
        })?;
    let state = extract_query_param(path, "state");

    Ok((code, state))
}

/// Extracts a query parameter value from a URL path.
fn extract_query_param(path: &str, key: &str) -> Option<String> {
    let query = path.split('?').nth(1)?;
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next()? == key {
            return parts.next().map(urldecoded);
        }
    }
    None
}

/// Exchanges an authorization code for tokens at the Xero token endpoint.
async fn exchange_code(
    client: &reqwest::Client,
    client_id: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> crate::error::Result<TokenResponse> {
    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", client_id),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("code_verifier", code_verifier),
    ];

    let response = client
        .post(TOKEN_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(crate::error::ChoSdkError::Network)?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(crate::error::ChoSdkError::AuthRequired {
            message: format!("Token exchange failed (HTTP {status}): {body}"),
        });
    }

    response
        .json::<TokenResponse>()
        .await
        .map_err(|e| crate::error::ChoSdkError::Parse {
            message: format!("Failed to parse token response: {e}"),
        })
}

/// Minimal percent-encoding for URL query parameters.
fn urlencoded(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push('%');
                result.push_str(&format!("{byte:02X}"));
            }
        }
    }
    result
}

/// Basic percent-decoding.
fn urldecoded(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(byte) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            result.push(byte);
            i += 3;
            continue;
        }
        if bytes[i] == b'+' {
            result.push(b' ');
        } else {
            result.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&result).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_generation() {
        let pkce = PkceChallenge::new();
        assert_eq!(pkce.verifier.len(), VERIFIER_LENGTH);
        // Challenge is base64url(sha256(verifier)) = 43 chars for 256-bit hash
        assert_eq!(pkce.challenge.len(), 43);
        // Verify the challenge matches the verifier
        let mut hasher = Sha256::new();
        hasher.update(pkce.verifier.as_bytes());
        let hash = hasher.finalize();
        let expected = URL_SAFE_NO_PAD.encode(hash);
        assert_eq!(pkce.challenge, expected);
    }

    #[test]
    fn pkce_verifier_uses_valid_chars() {
        let pkce = PkceChallenge::new();
        for ch in pkce.verifier.chars() {
            assert!(
                ch.is_ascii_alphanumeric() || ch == '-' || ch == '.' || ch == '_' || ch == '~',
                "Invalid verifier char: {ch}"
            );
        }
    }

    #[test]
    fn pkce_unique_each_time() {
        let a = PkceChallenge::new();
        let b = PkceChallenge::new();
        assert_ne!(a.verifier, b.verifier);
        assert_ne!(a.challenge, b.challenge);
    }

    #[test]
    fn parse_code_and_state_from_valid_request() {
        let request = "GET /callback?code=abc123&state=mystate&scope=openid HTTP/1.1";
        let (code, state) = parse_code_and_state_from_request(request).unwrap();
        assert_eq!(code, "abc123");
        assert_eq!(state.as_deref(), Some("mystate"));
    }

    #[test]
    fn parse_code_without_state() {
        let request = "GET /callback?code=abc123&scope=openid HTTP/1.1";
        let (code, state) = parse_code_and_state_from_request(request).unwrap();
        assert_eq!(code, "abc123");
        assert_eq!(state, None);
    }

    #[test]
    fn parse_code_from_error_request() {
        let request = "GET /callback?error=access_denied&error_description=User+denied HTTP/1.1";
        let err = parse_code_and_state_from_request(request).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("access_denied"));
    }

    #[test]
    fn generate_state_is_unique() {
        let a = generate_state();
        let b = generate_state();
        assert_ne!(a, b);
        assert_eq!(a.len(), 32);
    }

    #[test]
    fn urlencoded_basic() {
        assert_eq!(urlencoded("hello world"), "hello%20world");
        assert_eq!(urlencoded("a=b&c=d"), "a%3Db%26c%3Dd");
        assert_eq!(urlencoded("simple"), "simple");
    }

    #[test]
    fn urldecoded_basic() {
        assert_eq!(urldecoded("hello%20world"), "hello world");
        assert_eq!(urldecoded("a+b"), "a b");
        assert_eq!(urldecoded("simple"), "simple");
    }
}
