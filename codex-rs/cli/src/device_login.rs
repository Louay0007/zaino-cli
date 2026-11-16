//! `zaino login` — the portal's device authorization flow.
//!
//! The portal (not OpenAI's auth server) issues the device code. A developer runs
//! `zaino login`, gets a short user code, approves it in a browser where they are
//! already signed in, and the CLI polls until the portal hands back a one-time
//! `zn-` key. That key is written exactly where `--with-api-key` writes it, so the
//! two login paths converge on one credential and one file.
//!
//! Contract (portal/app/api/auth/device{,/token}/route.ts):
//!   POST {origin}/api/auth/device        {device_name?}
//!     -> {device_code, user_code, verification_url, expires_in, interval}
//!   POST {origin}/api/auth/device/token  {device_code}
//!     -> {token, models, user, organization} | error.code = authorization_pending
//!        | slow_down | expired_token | invalid_grant | access_denied
//!
//! Polling honours the server's `interval` and backs off on `slow_down`; polling
//! faster is what RFC 8628 tells servers to punish.

use codex_http_client::ClientRouteClass;
use codex_http_client::HttpClient;
use serde::Deserialize;
use std::time::Duration;
use std::time::Instant;

/// Where the portal lives. The CLI's provider already targets `api.zaino.soon.it`
/// for inference; the portal that mints keys is the apex origin.
pub const PORTAL_ORIGIN: &str = "https://zaino.soon.it";

#[derive(Debug)]
pub struct DeviceLogin {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub interval: u64,
    pub expires_in: u64,
}

#[derive(Deserialize)]
struct DeviceStartResp {
    device_code: String,
    user_code: String,
    verification_url: String,
    #[serde(default)]
    interval: Option<u64>,
    #[serde(default)]
    expires_in: Option<u64>,
}

#[derive(Deserialize)]
struct TokenResp {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    error: Option<TokenError>,
}

#[derive(Deserialize)]
struct TokenError {
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Deserialize)]
struct Envelope {
    #[serde(default)]
    error: Option<TokenError>,
}

/// A machine name the dashboard can show, so a person approving from a phone can
/// tell which laptop they are authorising.
fn device_name() -> String {
    let host = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .ok()
        .filter(|h| !h.trim().is_empty())
        .unwrap_or_else(|| "unknown-host".to_string());
    let host = host.trim().to_string();
    let host = host.split('.').next().unwrap_or(&host).to_string();
    format!("{host} ({})", std::env::consts::OS)
}

fn client(origin: &str, path: &str) -> std::io::Result<HttpClient> {
    let factory = codex_http_client::HttpClientFactory::new(
        codex_http_client::OutboundProxyPolicy::ReqwestDefault,
    );
    factory
        .build_client_without_request_logging(&format!("{origin}{path}"), ClientRouteClass::Auth)
        .map_err(std::io::Error::other)
}

async fn post_json(client: &HttpClient, url: &str, body: String) -> std::io::Result<(u16, String)> {
    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(std::io::Error::other)?;
    let status = resp.status().as_u16();
    let text = resp.text().await.map_err(std::io::Error::other)?;
    Ok((status, text))
}

/// Ask the portal for a device code and return the instructions to show.
pub async fn request_device_code(origin: &str) -> std::io::Result<DeviceLogin> {
    let client = client(origin, "/api/auth/device")?;
    let body = serde_json::json!({ "device_name": device_name() }).to_string();
    let (status, text) = post_json(&client, &format!("{origin}/api/auth/device"), body).await?;
    if !(200..300).contains(&status) {
        return Err(std::io::Error::other(portal_error_message(
            "start device login",
            status,
            &text,
        )));
    }
    let parsed: DeviceStartResp = serde_json::from_str(&text).map_err(|e| {
        std::io::Error::other(format!("portal returned an unexpected response: {e}"))
    })?;
    Ok(DeviceLogin {
        device_code: parsed.device_code,
        user_code: parsed.user_code,
        verification_url: parsed.verification_url,
        interval: parsed.interval.unwrap_or(5).max(1),
        expires_in: parsed.expires_in.unwrap_or(600),
    })
}

/// Poll until the code is approved, denied, or expired. Returns the `zn-` key.
pub async fn wait_for_api_key(origin: &str, flow: &DeviceLogin) -> std::io::Result<String> {
    let client = client(origin, "/api/auth/device/token")?;
    let url = format!("{origin}/api/auth/device/token");
    let body = serde_json::json!({ "device_code": flow.device_code }).to_string();
    let deadline = Instant::now() + Duration::from_secs(flow.expires_in);
    let mut interval = flow.interval;
    let mut last_error = None;

    while Instant::now() < deadline {
        tokio::time::sleep(Duration::from_secs(interval)).await;
        let (status, text) = match post_json(&client, &url, body.clone()).await {
            Ok(v) => v,
            Err(e) => {
                // A dropped connection is not a denial; keep waiting until the
                // code itself expires, and report the last transport error.
                last_error = Some(e.to_string());
                continue;
            }
        };

        if (200..300).contains(&status) {
            let parsed: TokenResp = serde_json::from_str(&text).map_err(|e| {
                std::io::Error::other(format!("portal returned an unexpected response: {e}"))
            })?;
            if let Some(token) = parsed.token.filter(|t| !t.trim().is_empty()) {
                return Ok(token.trim().to_string());
            }
            return Err(std::io::Error::other(
                "the portal approved the device but returned no key".to_string(),
            ));
        }

        let code = serde_json::from_str::<TokenResp>(&text)
            .ok()
            .and_then(|r| r.error)
            .map(|e| (e.code.unwrap_or_default(), e.message.unwrap_or_default()));

        match code.as_ref().map(|(c, _)| c.as_str()).unwrap_or("") {
            "authorization_pending" => continue,
            // RFC 8628: the client must slow down, by five seconds or more.
            "slow_down" => {
                interval += 5;
                continue;
            }
            "access_denied" => {
                return Err(std::io::Error::other(
                    "the request was denied in the browser",
                ));
            }
            "expired_token" => {
                return Err(std::io::Error::other(
                    "the code expired before it was approved",
                ));
            }
            _ => {
                let detail = code
                    .and_then(|(_, m)| (!m.is_empty()).then_some(m))
                    .unwrap_or_else(|| {
                        portal_error_message("complete device login", status, &text)
                    });
                return Err(std::io::Error::other(detail));
            }
        }
    }

    Err(std::io::Error::other(match last_error {
        Some(e) => format!("timed out waiting for approval (last error: {e})"),
        None => "timed out waiting for approval".to_string(),
    }))
}

/// The plan and allowance behind the current key, as the portal reports it.
///
/// `/api/device/usage` authenticates with the `zn-` key itself (no browser
/// session), which is the only reason a terminal can answer "how much is left"
/// at all. It is rate limited per key, so callers ask once — not per keystroke.
#[derive(Debug, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub plan_code: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub remaining_tokens: Option<i64>,
    #[serde(default)]
    pub pool_tokens: Option<i64>,
    #[serde(default)]
    pub pool_used_tokens: Option<i64>,
    #[serde(default)]
    pub token_exhausted: Vec<String>,
    #[serde(default)]
    pub period_end: Option<String>,
    #[serde(default)]
    pub cadence: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

impl Usage {
    /// Human sentence for "how much is left", or `None` when unlimited.
    pub fn remaining_summary(&self) -> Option<String> {
        let remaining = self.remaining_tokens?;
        let total = self.pool_tokens;
        let fmt = format_tokens;
        Some(match total {
            Some(total) if total > 0 => {
                format!("{} of {} left", fmt(remaining), fmt(total))
            }
            _ => format!("{} left", fmt(remaining)),
        })
    }

    pub fn reset_summary(&self) -> Option<String> {
        let end = self.period_end.as_deref()?;
        let date = end.split('T').next()?;
        Some(format!(
            "resets {date} ({})",
            self.cadence.clone().unwrap_or_else(|| "period".to_string())
        ))
    }
}

/// 1_234_567 -> "1.2M", 999 -> "999". Marketing-facing, so round to a readable
/// figure rather than showing an exact token count nobody needs.
pub fn format_tokens(n: i64) -> String {
    let n = n.max(0);
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Read the allowance for a key. `origin` is the portal apex.
pub async fn fetch_usage(origin: &str, api_key: &str) -> std::io::Result<Usage> {
    let path = "/api/device/usage";
    let client = client(origin, path)?;
    let resp = client
        .get(format!("{origin}{path}"))
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await
        .map_err(std::io::Error::other)?;
    let status = resp.status().as_u16();
    let text = resp.text().await.map_err(std::io::Error::other)?;
    if !(200..300).contains(&status) {
        return Err(std::io::Error::other(portal_error_message(
            "read usage",
            status,
            &text,
        )));
    }
    serde_json::from_str(&text)
        .map_err(|e| std::io::Error::other(format!("portal returned an unexpected response: {e}")))
}

fn portal_error_message(action: &str, status: u16, body: &str) -> String {
    let detail = serde_json::from_str::<Envelope>(body)
        .ok()
        .and_then(|r| r.error)
        .and_then(|e| e.message)
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| body.trim().chars().take(200).collect());
    if detail.is_empty() {
        format!("could not {action} (HTTP {status})")
    } else {
        format!("could not {action}: {detail}")
    }
}

#[cfg(test)]
#[path = "device_login_tests.rs"]
mod tests;
