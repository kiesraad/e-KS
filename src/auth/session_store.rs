//! In-memory session storage for the application.

use parking_lot::RwLock;
use secrecy::SecretString;
use std::{collections::HashMap, sync::Arc};

use crate::Session;

/// In-memory, thread-safe storage for active sessions.
#[derive(Clone, Default)]
pub struct SessionStore {
    inner: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionStore {
    /// Creates an empty session store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a session for the provided token, if it exists and is still valid.
    pub fn get(&self, token: &str) -> Option<Session> {
        let session = self.inner.read().get(token).cloned()?;
        if session.is_expired() {
            self.inner.write().remove(token);
            return None;
        }
        Some(session)
    }

    /// Inserts a session into the store.
    pub fn insert(&self, session: Session) {
        self.inner
            .write()
            .insert(session.token().to_exposed_string(), session);
    }

    /// Removes a session by its token. Returns the removed session, if any.
    pub fn remove(&self, token: &str) -> Option<Session> {
        self.inner.write().remove(token)
    }

    /// Returns an existing session if it is still valid.
    pub fn get_existing(&self, token: Option<&str>) -> Option<Session> {
        token.and_then(|token| self.get(token))
    }

    /// Creates, stores, and returns a new session.
    pub fn create_new(&self, id_code: &SecretString) -> Session {
        let session = Session::new(id_code);
        self.insert(session.clone());
        session
    }

    /// Removes all expired sessions from the store.
    pub fn cleanup_expired(&self) {
        let mut sessions = self.inner.write();
        sessions.retain(|_, session| !session.is_expired());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// Returns an existing session by token.
    #[test]
    fn get_existing_returns_session() {
        let store = SessionStore::new();
        let session = Session::new_test();
        let token = session.token().to_exposed_string();
        store.insert(session.clone());

        let loaded = store.get_existing(Some(&token));

        assert_eq!(loaded, Some(session));
    }

    /// Creates and stores a new session when requested.
    #[test]
    fn create_new_inserts_session() {
        let store = SessionStore::new();
        let session = store.create_new(&"test_id_code".into());

        let loaded = store.get(session.token().expose());

        assert_eq!(loaded, Some(session));
    }

    /// Removes expired sessions on lookup.
    #[test]
    fn get_removes_expired_sessions() {
        let store = SessionStore::new();
        let mut session = Session::new_test();
        session.last_activity =
            Instant::now() - crate::SESSION_IDLE_TIMEOUT - Duration::from_secs(1);
        let token = session.token().to_exposed_string();
        store.insert(session);

        let loaded = store.get(&token);

        assert!(loaded.is_none());
    }

    /// Removes expired sessions in bulk cleanup.
    #[test]
    fn cleanup_expired_removes_stale_sessions() {
        let store = SessionStore::new();
        let mut expired = Session::new_test();
        expired.last_activity =
            Instant::now() - crate::SESSION_IDLE_TIMEOUT - Duration::from_secs(1);
        let active = Session::new_test();
        let expired_token = expired.token().to_exposed_string();
        let active_token = active.token().to_exposed_string();
        store.insert(expired);
        store.insert(active);

        store.cleanup_expired();

        assert!(store.get(&expired_token).is_none());
        assert!(store.get(&active_token).is_some());
    }
}
