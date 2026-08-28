//! Short-lived cookie binding the OAuth `state` nonce to the browser that
//! started the flow: a callback whose nonce was minted for another browser is
//! rejected (login-CSRF defence, complementing the one-shot pending-request
//! check). The server-side 15-minute pending TTL bounds the nonce lifetime.

use axum_extra::extract::cookie::{Cookie, SameSite};

/// Name of the state cookie. Like the session cookie, the `__Host-` prefix
/// (production only) forbids a `Domain` and requires `Secure` + `Path=/`.
#[cfg(feature = "dev-features")]
pub(super) const STATE_COOKIE_NAME: &str = "EKS_GITHUB_STATE";
#[cfg(not(feature = "dev-features"))]
pub(super) const STATE_COOKIE_NAME: &str = "__Host-EKS_GITHUB_STATE";

fn apply_state_cookie_attributes(cookie: &mut Cookie<'static>) {
    cookie.set_http_only(true);
    #[cfg(feature = "dev-features")]
    cookie.set_secure(false);
    #[cfg(not(feature = "dev-features"))]
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
}

/// Cookie carrying the `state` nonce for the duration of the OAuth round-trip.
pub(super) fn build_state_cookie(nonce: String) -> Cookie<'static> {
    let mut cookie = Cookie::new(STATE_COOKIE_NAME, nonce);
    apply_state_cookie_attributes(&mut cookie);
    cookie
}

/// Expired twin of the state cookie for clearing it; attributes must match.
pub(super) fn build_state_removal_cookie() -> Cookie<'static> {
    let mut cookie = Cookie::from(STATE_COOKIE_NAME);
    apply_state_cookie_attributes(&mut cookie);
    cookie
}
