//! Session model and token generation.

use rand::{RngExt, distr::Alphanumeric};
use std::time::{Duration, Instant};

use crate::PoliticalGroupId;

/// Idle timeout after which a session is considered expired.
pub const SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Server-side session model stored in memory and attached to requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Opaque, random token that identifies the session.
    pub token: String,
    /// Timestamp of the last activity for idle-timeout validation.
    pub last_activity: Instant,
    /// Political group associated with this session (set on login).
    pub political_group_id: Option<PoliticalGroupId>,
}

impl Session {
    /// Creates a new session with a cryptographically strong random token.
    pub fn new() -> Self {
        Self {
            token: generate_session_token(),
            last_activity: Instant::now(),
            political_group_id: None,
        }
    }

    /// Assigns the political group for this session.
    pub fn set_political_group(&mut self, political_group_id: PoliticalGroupId) {
        self.political_group_id = Some(political_group_id);
    }

    /// Returns the session token as a string slice.
    pub fn token(&self) -> &str {
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
fn generate_session_token() -> String {
    // 62-character alphabet => log2(62) ~= 5.95 bits per char.
    // 42 chars gives ~250 bits of entropy (42 * 5.95 ~= 250) - the answer, obviously.
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(42)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures session tokens are 42-char base62 strings (~250-bit entropy).
    #[test]
    fn new_generates_base62_token() {
        let session = Session::new();

        assert_eq!(session.token.len(), 42);
        assert!(session.token.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    /// Confirms idle timeout invalidates stale sessions.
    #[test]
    fn session_expires_after_idle_timeout() {
        let mut session = Session::new();
        session.last_activity = Instant::now() - SESSION_IDLE_TIMEOUT - Duration::from_secs(1);

        assert!(session.is_expired());
    }
}
