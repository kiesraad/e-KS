//! Session model and token generation.

use chrono::{DateTime, Duration, Utc};
use rand::{RngExt, distr::Alphanumeric};
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};

use crate::{
    ElectionConfig, Locale, Scope, StreamId, TokenValue,
    form::{csrf_token_matches, generate_csrf_token, hash_csrf_token},
};

/// Idle timeout (in seconds) after which a session is considered expired.
const SESSION_IDLE_TIMEOUT_SECS: i64 = 10 * 60; // 10 minutes, per TVS "Checklist Testen" v2.1 T8: max 15 minutes inactivity.

/// Absolute cap on total session lifetime, regardless of activity (defense in
/// depth; TVS mandates only the idle ceiling). Covers one working day.
const SESSION_ABSOLUTE_TIMEOUT_SECS: i64 = 8 * 60 * 60; // 8 hours

/// Idle timeout after which a session is considered expired.
pub fn session_idle_timeout() -> Duration {
    Duration::seconds(SESSION_IDLE_TIMEOUT_SECS)
}

/// Absolute lifetime cap, checked regardless of activity.
pub fn session_absolute_timeout() -> Duration {
    Duration::seconds(SESSION_ABSOLUTE_TIMEOUT_SECS)
}

/// SHA-256 (hex) of a raw token: the value stored at rest and used as the lookup
/// key, so the bearer token itself is never persisted.
pub(crate) fn hash_token(raw: &str) -> String {
    Sha256::digest(raw.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Opaque session token kept secret until explicitly exposed.
#[derive(Clone)]
pub struct SessionToken(SecretString);

impl SessionToken {
    pub(crate) fn new(value: String) -> Self {
        Self(SecretString::from(value))
    }

    pub(crate) fn expose(&self) -> &str {
        self.0.expose_secret()
    }

    pub(crate) fn to_exposed_string(&self) -> String {
        self.expose().to_string()
    }
}

impl std::fmt::Debug for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionToken([REDACTED])")
    }
}

/// Server-side session data.
///
/// Persisted either in memory or the database depending on `STORAGE_URL`.
/// Carries no BSN/`id_code`: the user's stream id is pre-derived at login
/// and the election is tracked on the session directly.
#[derive(Clone)]
pub struct Session {
    /// SHA-256 (hex) of the token: the storage key and only token material at rest.
    pub(crate) token_hash: String,
    /// Raw token, held only until the `Set-Cookie` is emitted; `None` once loaded
    /// from storage, so a reloaded session can't re-expose it.
    pub(crate) raw_token: Option<SessionToken>,
    /// Hash of the random CSRF token: the only CSRF material at rest.
    pub(crate) csrf_token_hash: String,
    /// Raw CSRF token for form rendering; sourced from the CSRF cookie, never
    /// persisted (like `raw_token`).
    pub(crate) csrf_raw: Option<TokenValue>,
    /// Creation time, for the absolute-lifetime cap.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last activity for idle-timeout validation.
    pub last_activity: DateTime<Utc>,
    /// Truncated SHA-256 of the creating `User-Agent`; when set, the middleware
    /// rejects requests whose UA differs. `None` leaves the session unpinned.
    pub user_agent_hash: Option<String>,
    /// Stream belonging to the user (set on login).
    pub stream_id: Option<StreamId>,
    /// Authorization scope of the session, set on login. Governs which streams
    /// the session may reach (see [`crate::Scope`]).
    pub scope: Scope,
    /// Election the user is currently working on (set after login).
    pub current_election: Option<ElectionConfig>,
    /// Active locale for the session.
    pub locale: Locale,
    /// SAML `NameID` from the authenticating Assertion. Required to build a
    /// `LogoutRequest` for SP-initiated logout (eID §7.7.1).
    pub saml_name_id: Option<String>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("token", &"***")
            .field("created_at", &self.created_at)
            .field("last_activity", &self.last_activity)
            .field("stream_id", &self.stream_id)
            .field("scope", &self.scope)
            .field("current_election", &self.current_election)
            .field("locale", &self.locale)
            .finish()
    }
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.token_hash == other.token_hash
    }
}

impl Eq for Session {}

impl Session {
    /// Creates a new session with a cryptographically strong random token.
    pub fn new() -> Self {
        Self::new_with_locale(Locale::default())
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self::new_with_locale(Locale::default())
    }

    #[cfg(test)]
    pub fn new_test_with_locale(locale: Locale) -> Self {
        Self::new_with_locale(locale)
    }

    /// Creates a new session using the provided locale.
    pub fn new_with_locale(locale: Locale) -> Self {
        let raw_token = generate_session_token();
        let csrf_raw = generate_csrf_token();
        let now = Utc::now();
        Self {
            token_hash: hash_token(raw_token.expose()),
            raw_token: Some(raw_token),
            csrf_token_hash: hash_csrf_token(&csrf_raw.0),
            csrf_raw: Some(csrf_raw),
            created_at: now,
            last_activity: now,
            user_agent_hash: None,
            stream_id: None,
            scope: Scope::default(),
            current_election: None,
            locale,
            saml_name_id: None,
        }
    }

    /// Assigns the stream for this session.
    pub fn set_stream_id(&mut self, stream_id: StreamId) {
        self.stream_id = Some(stream_id);
    }

    /// Assigns the authorization scope for this session.
    pub fn set_scope(&mut self, scope: Scope) {
        self.scope = scope;
    }

    /// Assigns the current election for this session.
    pub fn set_current_election(&mut self, election: ElectionConfig) {
        self.current_election = Some(election);
    }

    /// Pins the session to the hash of the client's `User-Agent`.
    pub fn set_user_agent_hash(&mut self, user_agent_hash: String) {
        self.user_agent_hash = Some(user_agent_hash);
    }

    /// Returns the SHA-256 (hex) storage key for this session.
    pub(crate) fn token_hash(&self) -> &str {
        &self.token_hash
    }

    /// Raw token if still in memory (only between creation and cookie-minting).
    pub(crate) fn reveal_token(&self) -> Option<&SessionToken> {
        self.raw_token.as_ref()
    }

    /// Test helper: raw token of a fresh session (panics on a reloaded one).
    #[cfg(test)]
    pub fn token_string(&self) -> String {
        self.reveal_token()
            .expect("fresh session retains its raw token")
            .to_exposed_string()
    }

    /// True once past the idle timeout or the absolute lifetime cap.
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        now - self.last_activity >= session_idle_timeout()
            || now - self.created_at >= session_absolute_timeout()
    }

    /// Raw CSRF token to embed in forms; empty when no cookie has been sourced
    /// yet (fails the check, the safe default).
    pub fn csrf_token(&self) -> TokenValue {
        self.csrf_raw.clone().unwrap_or_default()
    }

    /// Constant-time check of a submitted CSRF token against the stored hash.
    pub fn csrf_matches(&self, submitted: &str) -> bool {
        csrf_token_matches(submitted, &self.csrf_token_hash)
    }

    /// Set the CSRF token, updating both the stored hash and the raw value.
    pub fn set_csrf(&mut self, raw: TokenValue) {
        self.csrf_token_hash = hash_csrf_token(&raw.0);
        self.csrf_raw = Some(raw);
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates a random session token with ~250 bits of entropy.
fn generate_session_token() -> SessionToken {
    // 62-character alphabet => log2(62) ~= 5.95 bits per char.
    // 42 chars gives ~250 bits of entropy (42 * 5.95 ~= 250) - the answer, obviously.
    let token = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(42)
        .map(char::from)
        .collect();
    SessionToken::new(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures session tokens are 42-char base62 strings (~250-bit entropy).
    #[test]
    fn new_generates_base62_token() {
        let session = Session::new_test();
        let token = session.token_string();

        assert_eq!(token.len(), 42);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    /// Stored key is the token's hash, not the token itself.
    #[test]
    fn token_hash_is_sha256_of_raw_token() {
        let session = Session::new_test();
        let raw = session.token_string();

        assert_eq!(session.token_hash(), hash_token(&raw));
        assert_eq!(session.token_hash().len(), 64); // 32 bytes hex-encoded
        assert_ne!(session.token_hash(), raw);
    }

    /// The CSRF token is random and independent of the session token; only its
    /// hash is kept, and that hash is distinct from the raw token.
    #[test]
    fn csrf_token_is_random_and_independent_of_session_token() {
        let session = Session::new_test();

        assert!(!session.csrf_token().0.is_empty());
        assert_ne!(session.csrf_token().0, session.token_hash());
        assert_ne!(session.csrf_token().0, session.token_string());
        assert_ne!(session.csrf_token().0, session.csrf_token_hash);
        // Distinct sessions get distinct CSRF tokens.
        assert_ne!(session.csrf_token(), Session::new_test().csrf_token());
    }

    /// The stored hash verifies the raw token and rejects other values.
    #[test]
    fn csrf_matches_verifies_submitted_token() {
        let session = Session::new_test();
        let token = session.csrf_token().to_string();

        assert!(session.csrf_matches(&token));
        assert!(!session.csrf_matches("wrong"));
    }

    /// `set_csrf` rotates both the raw token and its stored hash together.
    #[test]
    fn set_csrf_rotates_token_and_hash() {
        let mut session = Session::new_test();
        let old = session.csrf_token().to_string();

        session.set_csrf(crate::form::generate_csrf_token());

        assert!(!session.csrf_matches(&old));
        assert!(session.csrf_matches(&session.csrf_token().to_string()));
    }

    /// Confirms idle timeout invalidates stale sessions.
    #[test]
    fn session_expires_after_idle_timeout() {
        let mut session = Session::new_test();
        session.last_activity = Utc::now() - session_idle_timeout() - Duration::seconds(1);

        assert!(session.is_expired());
    }

    /// A refreshed-but-old session still expires at the absolute cap.
    #[test]
    fn session_expires_after_absolute_timeout() {
        let mut session = Session::new_test();
        session.last_activity = Utc::now(); // not idle
        session.created_at = Utc::now() - session_absolute_timeout() - Duration::seconds(1);

        assert!(session.is_expired());
    }
}
