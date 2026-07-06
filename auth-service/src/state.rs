use axum::{http::HeaderMap, response::Response};
use axum_extra::extract::CookieJar;
use secrecy::SecretString;

use crate::error::Result;

/// SAML state shared by the auth-service handlers (placeholder: currently empty).
#[derive(Clone, Default)]
pub struct AuthServiceState;

impl AuthServiceState {
    /// Build state from the environment (placeholder: yields empty state).
    pub async fn new_from_env() -> Result<Self> {
        Ok(Self)
    }

    /// Empty state for tests and dev-login-only boots.
    pub fn new_empty() -> Self {
        Self
    }
}

/// Application callbacks required by the auth-service router: session
/// creation/teardown and AuthnRequest ID storage for the `InResponseTo` replay
/// check (eID §9.7). See [`crate::PendingRequests`] for a ready-made
/// in-memory implementation.
pub trait AuthState: Clone + Send + Sync + 'static {
    /// Called after a validated SAML assertion. The application creates its
    /// session and returns the browser response (typically a redirect). `name_id`
    /// should be persisted to build the `LogoutRequest` later (eID §7.7.1).
    fn on_authenticated(
        &self,
        subject_id: SubjectId,
        name_id: Option<String>,
        jar: CookieJar,
        headers: &HeaderMap,
    ) -> impl std::future::Future<Output = Response> + Send;

    /// Called when authentication ends without a session (cancelled or error).
    /// The application shows the appropriate page and clears any existing session.
    fn on_authentication_failed(
        &self,
        failure: AuthFailure,
        jar: CookieJar,
        headers: &HeaderMap,
    ) -> impl std::future::Future<Output = Response> + Send;

    /// Terminate the local session for SP-initiated logout. Always returns the
    /// jar with the session cookie cleared, plus the SAML `NameID` if one was
    /// recorded (needed for the `LogoutRequest`, eID §7.7.1).
    fn logout_session(
        &self,
        jar: CookieJar,
    ) -> impl std::future::Future<Output = (CookieJar, Option<String>)> + Send;

    /// Persist an outgoing AuthnRequest ID for the `InResponseTo` check (eID §7.6.3.5 rule 4 / §9.7).
    fn register_pending_request(&self, id: String) -> impl std::future::Future<Output = ()> + Send;

    /// Validate and consume an `InResponseTo` ID (eID §7.6.3.5 / §9.7).
    /// Returns `true` once for a valid pending ID; `false` for unknown, expired,
    /// or already-consumed IDs. Implementations should fail closed on errors.
    fn consume_if_pending(&self, id: String) -> impl std::future::Future<Output = bool> + Send;
}

#[derive(Debug, Clone)]
pub struct SubjectId {
    /// BSN / pseudonym (PII). Never reaches `Debug`/logs; zeroized on drop.
    pub value: SecretString,
    pub name_qualifier: String,
}

/// Reason a SAML authentication attempt ended without a session.
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
