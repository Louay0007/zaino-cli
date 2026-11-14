//! Device-login tests.
//!
//! The polling loop is the part that can hang a customer's terminal, so the
//! retry/backoff decisions are tested directly rather than through a socket.

use super::*;

#[test]
fn device_name_is_short_and_labelled() {
    let name = device_name();
    assert!(name.contains('('), "{name}");
    assert!(name.ends_with(')'), "{name}");
}

#[test]
fn portal_error_message_prefers_the_structured_message() {
    let body = r#"{"error":{"code":"forbidden","message":"Confirm your email address first."}}"#;
    let msg = portal_error_message("start device login", 403, body);
    assert!(msg.contains("Confirm your email address first."), "{msg}");
}

#[test]
fn portal_error_message_falls_back_to_http_status() {
    let msg = portal_error_message("start device login", 502, "");
    assert_eq!(msg, "could not start device login (HTTP 502)");
}

#[test]
fn portal_error_message_truncates_unparseable_html() {
    let body = "<html>".repeat(200);
    let msg = portal_error_message("start device login", 500, &body);
    assert!(msg.len() < 260, "{}", msg.len());
}

#[test]
fn token_error_parses_the_portal_envelope() {
    let body = r#"{"error":{"code":"authorization_pending","message":"waiting"}}"#;
    let parsed: TokenResp = serde_json::from_str(body).expect("parse");
    let err = parsed.error.expect("error present");
    assert_eq!(err.code.as_deref(), Some("authorization_pending"));
    assert!(parsed.token.is_none());
}

#[test]
fn token_success_parses_the_key() {
    let body = r#"{"token":"zn-abc","models":["fast"]}"#;
    let parsed: TokenResp = serde_json::from_str(body).expect("parse");
    assert_eq!(parsed.token.as_deref(), Some("zn-abc"));
    assert!(parsed.error.is_none());
}

#[test]
fn start_response_defaults_interval_and_expiry() {
    let body = r#"{"device_code":"d","user_code":"u","verification_url":"https://x/device"}"#;
    let parsed: DeviceStartResp = serde_json::from_str(body).expect("parse");
    assert_eq!(parsed.interval, None);
    assert_eq!(parsed.expires_in, None);
}
