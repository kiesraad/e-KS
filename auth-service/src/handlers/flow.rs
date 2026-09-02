//! Browser-binding of the SSO flow to defeat login-CSRF / forced login.
//!
//! `/login` sets a short-lived cookie holding the AuthnRequest ID plus a hash
//! of the browser's `User-Agent`; `GET /saml/sp/acs` requires that cookie to
//! match the validated assertion's `InResponseTo` (and the same User-Agent)
//! before a session is created. Because the cookie cannot be set on another
//! browser cross-origin, an attacker cannot make a victim's browser complete a
//! flow the attacker started (the assertion's `InResponseTo` would still be
//! outstanding, but the victim's browser carries no matching cookie).
//!
//! This lives entirely in the auth-service crate and needs no
//! embedding-application API: both `handle_login` and `handle_acs` already
//! receive the [`CookieJar`] and request [`HeaderMap`].
//!
//! Note: only one SSO flow per browser is in flight at a time (a second `/login`
//! overwrites the cookie); completing the older flow then fails closed and the
//! user simply re-authenticates. That is acceptable for an SSO entry point.

use crate::types::{EndpointUrl, MessageId};
use axum::http::{HeaderMap, header::USER_AGENT};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use sha2::{Digest, Sha256};

/// Cookie name in https deployments. The `__Host-` prefix forbids a `Domain`
/// attribute and requires `Secure` + `Path=/`, so a sibling subdomain cannot
/// plant it (defeats cookie fixation of the binding value).
const FLOW_COOKIE_HOST: &str = "__Host-eks-saml-flow";
/// Cookie name for plain-http local development: the `__Host-` prefix requires
/// `Secure`, which a browser refuses to honor over http.
const FLOW_COOKIE_DEV: &str = "eks-saml-flow";

/// Lifetime of the binding cookie. Derived from the pending-request window
/// ([`crate::PENDING_REQUEST_TTL`]) so an abandoned flow's cookie does not linger.
const FLOW_COOKIE_TTL_MINUTES: i64 = (crate::PENDING_REQUEST_TTL.as_secs() / 60) as i64;

fn cookie_name(secure: bool) -> &'static str {
    if secure {
        FLOW_COOKIE_HOST
    } else {
        FLOW_COOKIE_DEV
    }
}

/// Short (64-bit) hex hash of the request `User-Agent`. Pins the flow to the
/// browser's UA without storing the raw header; a missing UA hashes the empty
/// string (still consistent between `/login` and the ACS callback).
fn ua_hash(headers: &HeaderMap) -> String {
    let ua = headers
        .get(USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let digest = Sha256::digest(ua.as_bytes());
    digest.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// The bound cookie value: the AuthnRequest ID and the User-Agent hash. The ID
/// is a [`MessageId`], i.e. an XML `NCName`, so it cannot carry the `;` or `,`
/// that would break out of the cookie value.
fn bound_value(authn_id: &MessageId, headers: &HeaderMap) -> String {
    format!("{authn_id}.{}", ua_hash(headers))
}

/// Build the `Set-Cookie` that binds an SSO flow to this browser, set by
/// `handle_login`. `acs_url` selects the cookie name and `Secure` flag.
pub(crate) fn flow_cookie(
    acs_url: &EndpointUrl,
    authn_id: &MessageId,
    headers: &HeaderMap,
) -> Cookie<'static> {
    let secure = acs_url.is_https();
    Cookie::build((cookie_name(secure), bound_value(authn_id, headers)))
        .http_only(true)
        .secure(secure)
        // Lax (not Strict): the RD returns the artifact via a top-level cross-site
        // GET redirect to the ACS, and Lax sends the cookie on exactly that.
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(cookie::time::Duration::minutes(FLOW_COOKIE_TTL_MINUTES))
        .build()
}

/// Verify, at the ACS callback, that this browser started the flow for
/// `expected_authn_id` (matching cookie value and User-Agent), and return the jar
/// with the one-shot cookie removed. `false` when the cookie is absent or does
/// not match. Always removes the cookie so it cannot be reused.
pub(crate) fn verify_and_clear(
    jar: CookieJar,
    acs_url: &EndpointUrl,
    expected_authn_id: &MessageId,
    headers: &HeaderMap,
) -> (bool, CookieJar) {
    let secure = acs_url.is_https();
    let name = cookie_name(secure);
    let expected = bound_value(expected_authn_id, headers);
    let ok = jar.get(name).is_some_and(|c| c.value() == expected);
    // The removal cookie must carry the same Path (and Secure for the __Host-
    // prefix) the browser stored it with, or the browser keeps the original.
    let removal = Cookie::build((name, "")).path("/").secure(secure).build();
    (ok, jar.remove(removal))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers_with_ua(ua: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(USER_AGENT, ua.parse().unwrap());
        h
    }

    fn acs(url: &str) -> EndpointUrl {
        EndpointUrl::from_base_url(url, "ACS").expect("test ACS URL")
    }

    fn id(value: &str) -> MessageId {
        MessageId::parse(value).expect("test message id")
    }

    fn jar_with(name: &str, value: &str) -> CookieJar {
        CookieJar::new().add(Cookie::build((name.to_string(), value.to_string())).build())
    }

    #[test]
    fn https_uses_host_prefixed_secure_cookie() {
        let h = headers_with_ua("agent/1");
        let c = flow_cookie(&acs("https://dv.example.com/saml/sp/acs"), &id("_abc"), &h);
        assert_eq!(c.name(), FLOW_COOKIE_HOST);
        assert_eq!(c.secure(), Some(true));
        assert_eq!(c.same_site(), Some(SameSite::Lax));
        assert_eq!(c.http_only(), Some(true));
        assert!(c.value().starts_with("_abc."));
    }

    #[test]
    fn http_dev_uses_plain_non_secure_cookie() {
        let h = headers_with_ua("agent/1");
        let c = flow_cookie(&acs("http://localhost:3000/saml/sp/acs"), &id("_abc"), &h);
        assert_eq!(c.name(), FLOW_COOKIE_DEV);
        assert_eq!(c.secure(), Some(false));
    }

    #[test]
    fn verify_accepts_matching_browser_and_ua() {
        let h = headers_with_ua("agent/1");
        let acs = acs("https://dv.example.com/saml/sp/acs");
        let value = flow_cookie(&acs, &id("_abc"), &h).value().to_string();
        let jar = jar_with(FLOW_COOKIE_HOST, &value);
        let (ok, _) = verify_and_clear(jar, &acs, &id("_abc"), &h);
        assert!(ok);
    }

    #[test]
    fn verify_rejects_missing_cookie() {
        let h = headers_with_ua("agent/1");
        let acs = acs("https://dv.example.com/saml/sp/acs");
        let (ok, _) = verify_and_clear(CookieJar::new(), &acs, &id("_abc"), &h);
        assert!(!ok, "absent flow cookie must be rejected (login CSRF)");
    }

    #[test]
    fn verify_rejects_wrong_authn_id() {
        // Models forced login: the victim's browser carries a cookie for a
        // different (or no) flow than the assertion's InResponseTo.
        let h = headers_with_ua("agent/1");
        let acs = acs("https://dv.example.com/saml/sp/acs");
        let value = flow_cookie(&acs, &id("_attacker"), &h).value().to_string();
        let jar = jar_with(FLOW_COOKIE_HOST, &value);
        let (ok, _) = verify_and_clear(jar, &acs, &id("_victim-request"), &h);
        assert!(!ok);
    }

    #[test]
    fn verify_rejects_changed_user_agent() {
        let acs = acs("https://dv.example.com/saml/sp/acs");
        let value = flow_cookie(&acs, &id("_abc"), &headers_with_ua("agent/1"))
            .value()
            .to_string();
        let jar = jar_with(FLOW_COOKIE_HOST, &value);
        let (ok, _) = verify_and_clear(jar, &acs, &id("_abc"), &headers_with_ua("agent/2"));
        assert!(!ok, "a different User-Agent must not satisfy the binding");
    }
}
