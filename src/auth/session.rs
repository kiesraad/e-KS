//! Session model and token generation.

use rand::{RngExt, distr::Alphanumeric};
use secrecy::{ExposeSecret, SecretString};
use std::time::{Duration, Instant};

use crate::{CsrfTokens, ElectionConfig, Locale, PoliticalGroupId};

/// Idle timeout after which a session is considered expired.
pub const SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Server-side session model stored in memory and attached to requests.
#[derive(Clone)]
pub struct SessionToken(SecretString);

impl SessionToken {
    fn new(value: String) -> Self {
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

/// Server-side session data persisted in memory.
#[derive(Clone)]
pub struct Session {
    /// Opaque, random token that identifies the session.
    token: SessionToken,
    /// Timestamp of the last activity for idle-timeout validation.
    pub last_activity: Instant,
    /// Political group associated with this session (set on login).
    pub political_group_id: Option<PoliticalGroupId>,
    /// Active locale for the session.
    pub locale: Locale,
    /// CSRF tokens scoped to this session.
    pub csrf_tokens: CsrfTokens,
    /// Election configuration for the session.
    pub election: ElectionConfig,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("token", &"***")
            .field("last_activity", &self.last_activity)
            .field("political_group_id", &self.political_group_id)
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

    /// Creates a new session using the provided locale.
    pub fn new_with_locale(locale: Locale) -> Self {
        Self {
            token: generate_session_token(),
            last_activity: Instant::now(),
            political_group_id: None,
            locale,
            csrf_tokens: CsrfTokens::default(),
            election: ElectionConfig::EK2027,
        }
    }

    /// Assigns the political group for this session.
    pub fn set_political_group(&mut self, political_group_id: PoliticalGroupId) {
        self.political_group_id = Some(political_group_id);
    }

    /// Returns the session token (kept secret until explicitly exposed).
    pub(crate) fn token(&self) -> &SessionToken {
        &self.token
    }

    /// Returns true when the session has been idle past the configured timeout.
    pub fn is_expired(&self) -> bool {
        self.last_activity.elapsed() >= SESSION_IDLE_TIMEOUT
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates a random session token with ~256 bits of entropy.
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
        let session = Session::new();

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
        let mut session = Session::new();
        session.last_activity = Instant::now() - SESSION_IDLE_TIMEOUT - Duration::from_secs(1);

        assert!(session.is_expired());
    }
}
