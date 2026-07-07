//! Session cookie helpers and request extraction.

use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, header::USER_AGENT, request::Parts},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use sha2::{Digest, Sha256};

use crate::{AppError, Session, TokenValue};

/// Name of the session cookie. The `__Host-` prefix (production only) forbids a
/// `Domain` and requires `Secure` + `Path=/`, blocking sibling-subdomain
/// shadowing. It mandates `Secure`, so dev over http keeps the bare name.
#[cfg(feature = "dev-features")]
pub const SESSION_COOKIE_NAME: &str = "EKS_SESSION_ID";
#[cfg(not(feature = "dev-features"))]
pub const SESSION_COOKIE_NAME: &str = "__Host-EKS_SESSION_ID";

/// CSRF cookie carrying the raw token; `HttpOnly` (server echoes it into forms).
#[cfg(feature = "dev-features")]
pub const CSRF_COOKIE_NAME: &str = "EKS_CSRF";
#[cfg(not(feature = "dev-features"))]
pub const CSRF_COOKIE_NAME: &str = "__Host-EKS_CSRF";

/// Builds the session cookie. Only valid right after creation, while the raw
/// token is still in memory.
pub(crate) fn build_session_cookie(session: &Session) -> Cookie<'static> {
    let token = session
        .reveal_token()
        .expect("build_session_cookie requires a freshly created session with its raw token");
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, token.to_exposed_string());
    apply_session_cookie_attributes(&mut cookie);
    cookie
}

/// Expired twin of the session cookie for clearing it. Attributes must match the
/// set cookie (esp. `Secure` + `Path=/` for the `__Host-` prefix) or it lingers.
pub(crate) fn build_removal_cookie() -> Cookie<'static> {
    let mut cookie = Cookie::from(SESSION_COOKIE_NAME);
    apply_session_cookie_attributes(&mut cookie);
    cookie
}

/// CSRF cookie with the raw token, same attributes as the session cookie.
pub(crate) fn build_csrf_cookie(raw: TokenValue) -> Cookie<'static> {
    let mut cookie = Cookie::new(CSRF_COOKIE_NAME, raw.0);
    apply_session_cookie_attributes(&mut cookie);
    cookie
}

/// Expired twin of the CSRF cookie, for logout.
pub(crate) fn build_csrf_removal_cookie() -> Cookie<'static> {
    let mut cookie = Cookie::from(CSRF_COOKIE_NAME);
    apply_session_cookie_attributes(&mut cookie);
    cookie
}

fn apply_session_cookie_attributes(cookie: &mut Cookie<'static>) {
    cookie.set_http_only(true);
    #[cfg(feature = "dev-features")]
    cookie.set_secure(false);
    #[cfg(not(feature = "dev-features"))]
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
}

/// Truncated (64-bit) hex SHA-256 of the request `User-Agent` for session
/// pinning; a missing UA hashes the empty string.
pub(crate) fn user_agent_hash(headers: &HeaderMap) -> String {
    let ua = headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    Sha256::digest(ua.as_bytes())
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Extracts the current session from request extensions.
impl<S> FromRequestParts<S> for Session
where
    S: Send + Sync,
{
    type Rejection = AppError;

    /// Retrieves the session that was injected by the session middleware.
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Session>()
            .cloned()
            .ok_or(AppError::InternalServerError)
    }
}
