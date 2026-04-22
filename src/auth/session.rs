//! Session model and token generation.

use chrono::{DateTime, Duration, Utc};
use rand::{RngExt, distr::Alphanumeric};
use secrecy::{ExposeSecret, SecretString};

use crate::{AppError, CsrfTokens, ElectionConfig, Locale, StreamId, TokenValue};

/// Idle timeout (in seconds) after which a session is considered expired.
const SESSION_IDLE_TIMEOUT_SECS: i64 = 10 * 60;

/// Idle timeout after which a session is considered expired.
pub fn session_idle_timeout() -> Duration {
    Duration::seconds(SESSION_IDLE_TIMEOUT_SECS)
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

impl PartialEq for SessionToken {
    fn eq(&self, other: &Self) -> bool {
        self.expose() == other.expose()
    }
}

impl Eq for SessionToken {}

impl std::hash::Hash for SessionToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.expose().hash(state);
    }
}

/// Server-side session data.
///
/// Persisted either in memory or the database depending on `STORAGE_URL`.
/// Carries no BSN/`id_code`: the user's stream id is pre-derived at login
/// and the election is tracked on the session directly.
#[derive(Clone)]
pub struct Session {
    /// Opaque, random token that identifies the session.
    pub(crate) token: SessionToken,
    /// Timestamp of the last activity for idle-timeout validation.
    pub last_activity: DateTime<Utc>,
    /// Stream belonging to the user (set on login).
    pub stream_id: Option<StreamId>,
    /// Election the user is currently working on (set after login).
    pub current_election: Option<ElectionConfig>,
    /// Active locale for the session.
    pub locale: Locale,
    /// CSRF tokens scoped to this session.
    pub csrf_tokens: CsrfTokens,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("token", &"***")
            .field("last_activity", &self.last_activity)
            .field("stream_id", &self.stream_id)
            .field("current_election", &self.current_election)
            .field("locale", &self.locale)
            .finish()
    }
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
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
        Self {
            token: generate_session_token(),
            last_activity: Utc::now(),
            stream_id: None,
            current_election: None,
            locale,
            csrf_tokens: CsrfTokens::default(),
        }
    }

    /// Assigns the stream for this session.
    pub fn set_stream_id(&mut self, stream_id: StreamId) {
        self.stream_id = Some(stream_id);
    }

    /// Assigns the current election for this session.
    pub fn set_current_election(&mut self, election: ElectionConfig) {
        self.current_election = Some(election);
    }

    /// Returns the session token (kept secret until explicitly exposed).
    pub(crate) fn token(&self) -> &SessionToken {
        &self.token
    }

    /// Returns true when the session has been idle past the configured timeout.
    pub fn is_expired(&self) -> bool {
        Utc::now() - self.last_activity >= session_idle_timeout()
    }

    /// Consume a CSRF token from the session, returning
    /// [`AppError::CsrfTokenInvalid`] if it is not recognised.
    pub fn consume_csrf(&self, token: &str) -> Result<(), AppError> {
        if self.csrf_tokens.consume(&TokenValue(token.to_string())) {
            Ok(())
        } else {
            Err(AppError::CsrfTokenInvalid)
        }
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

        assert_eq!(session.token().expose().len(), 42);
        assert!(
            session
                .token()
                .expose()
                .chars()
                .all(|c| c.is_ascii_alphanumeric())
        );
    }

    /// Confirms idle timeout invalidates stale sessions.
    #[test]
    fn session_expires_after_idle_timeout() {
        let mut session = Session::new_test();
        session.last_activity = Utc::now() - session_idle_timeout() - Duration::seconds(1);

        assert!(session.is_expired());
    }
}
