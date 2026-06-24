use axum::{http::HeaderMap, response::Response};
use axum_extra::extract::CookieJar;
use secrecy::SecretString;

use crate::error::Result;

/// SAML state shared by the auth-service handlers. In the full implementation
/// this holds the DV/RD keys, the verified RD descriptor, and the cached signed
/// SP metadata; here it is an empty placeholder. Cheap to clone, so an embedding
/// application can store one instance on its own state and let the handlers
/// extract it via `FromRef`.
#[derive(Clone, Default)]
pub struct AuthServiceState;

impl AuthServiceState {
    /// Build the state from the environment. In the full implementation this
    /// loads DV keys from disk and fetches the IdP metadata over HTTP; the
    /// placeholder simply yields an empty state.
    pub async fn new_from_env() -> Result<Self> {
        Ok(Self)
    }

    /// Build a minimal, empty state. Intended for embedding applications' tests
    /// (and dev-login-only boots) that never perform a live SAML round-trip.
    pub fn new_empty() -> Self {
        Self
    }
}

/// Application-side flow callbacks the auth-service router needs from the
/// embedding application. SAML state (keys, config) lives in
/// [`AuthServiceState`], which the handlers extract directly via `FromRef`;
/// this trait covers what is application-specific: creating a session after a
/// successful login, tearing one down on logout, and storing the outstanding
/// AuthnRequest IDs used for the `InResponseTo` replay check (eID §9.7), the
/// latter is the application's concern so the IDs can be shared across
/// instances (see [`crate::PendingRequests`] for a ready-made in-memory
/// implementation).
pub trait AuthState: Clone + Send + Sync + 'static {
    /// Called after a SAML Assertion has been fully validated and an acting
    /// SubjectID has been established. The embedding application creates its
    /// own session, sets whatever cookie it uses, and returns the response the
    /// browser should receive, typically a redirect to the post-login landing
    /// page.
    ///
    /// Only the two values the application needs cross this boundary: the
    /// acting `subject_id` (the authenticated identity) and the SAML `name_id`,
    /// which the application persists so it can later build the `LogoutRequest`
    /// (eID §7.7.1).
    fn on_authenticated(
        &self,
        subject_id: SubjectId,
        name_id: Option<String>,
        jar: CookieJar,
        headers: &HeaderMap,
    ) -> impl std::future::Future<Output = Response> + Send;

    /// Called when a SAML authentication attempt does not result in a session:
    /// the user cancelled at the IdP ([`AuthFailure::Cancelled`], TVS T3) or
    /// something went wrong while resolving/validating the response
    /// ([`AuthFailure::Error`], TVS L10). The embedding application is
    /// responsible for the user-facing page and for tearing down any existing
    /// local session (TVS L10). `headers` is provided for locale negotiation;
    /// `jar` lets the application clear its cookie.
    fn on_authentication_failed(
        &self,
        failure: AuthFailure,
        jar: CookieJar,
        headers: &HeaderMap,
    ) -> impl std::future::Future<Output = Response> + Send;

    /// Tear down the current session for SP-initiated logout. Returns the SAML
    /// `NameID` that was recorded at login (needed to build the
    /// `LogoutRequest`, eID §7.7.1) together with a cookie jar in which the
    /// session cookie has been cleared. Returns `None` when no active session
    /// is present, callers should then redirect the browser home.
    fn logout_session(
        &self,
        jar: CookieJar,
    ) -> impl std::future::Future<Output = Option<(String, CookieJar)>> + Send;

    /// Persist an outgoing AuthnRequest ID for the later `InResponseTo` replay
    /// check (eID §7.6.3.5 rule 4 / §9.7).
    fn register_pending_request(&self, id: String) -> impl std::future::Future<Output = ()> + Send;

    /// Atomically validate and consume an incoming Assertion's `InResponseTo`
    /// against the outstanding AuthnRequest IDs (eID §7.6.3.5 rule 4 / §9.7).
    ///
    /// Returns `true` iff `id` was a still-valid outstanding request: one
    /// registered via [`register_pending_request`](Self::register_pending_request)
    /// and not yet past the retention window. In that case the ID is consumed in
    /// the same step so it can never be matched again, closing the replay window
    /// without a separate check-then-consume round-trip. Returns `false` for an
    /// unknown, expired, or already-consumed ID (implementations should likewise
    /// fail closed on a storage error); the caller must then reject the
    /// Assertion.
    fn consume_if_pending(&self, id: String) -> impl std::future::Future<Output = bool> + Send;
}

#[derive(Debug, Clone)]
pub struct SubjectId {
    /// The subject identifier (BSN / pseudonym, PII). Wrapped so it never
    /// reaches `Debug`/logs and is zeroized on drop; call `.expose_secret()`
    /// to read it.
    pub value: SecretString,
    pub name_qualifier: String,
}

/// Why a SAML authentication attempt ended without an authenticated session.
///
/// Passed to [`AuthState::on_authentication_failed`] so the embedding
/// application can show the appropriate user-facing page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthFailure {
    Cancelled,
    Error,
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty_constructs() {
        let _state = AuthServiceState::new_empty();
    }
}
