//! Session storage with pluggable in-memory or database backends.
//!
//! The backend is selected by `STORAGE_URL`:
//! - `memory://...` → in-memory (default).
//! - `local://...` → in-memory (disk is **not** a valid session backend; we
//!   treat the filesystem as less trusted than application memory).
//! - `postgres://...` / `postgresql://` → Postgres.

use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
#[cfg(feature = "database")]
use tracing::error;
use tracing::info;
use url::Url;

use crate::{AppError, Locale, Session};

#[cfg(feature = "database")]
use crate::auth::session_db;

/// Session storage backend.
#[derive(Clone)]
pub enum SessionStore {
    /// Process-local, in-memory storage. Cleared on restart.
    InMemory(Arc<RwLock<HashMap<String, Session>>>),
    /// Postgres-backed storage, survives restarts.
    #[cfg(feature = "database")]
    Database(sqlx::PgPool),
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::InMemory(Arc::new(RwLock::new(HashMap::new())))
    }
}

impl SessionStore {
    /// Construct a session store from `STORAGE_URL`.
    ///
    /// Disk-backed storage (`local://`) is not a valid session backend and
    /// falls back to an in-memory store (with a log line noting the fallback).
    pub fn from_storage_url(storage_url: &str) -> Result<Self, AppError> {
        let url = Url::parse(storage_url)
            .map_err(|err| AppError::ConfigLoadError(format!("Invalid storage URL: {err}")))?;

        match url.scheme() {
            "memory" => Ok(Self::default()),
            "local" => {
                info!(
                    "sessions: STORAGE_URL is local://; falling back to in-memory \
                     (disk-backed sessions are not supported)"
                );
                Ok(Self::default())
            }
            "postgres" | "postgresql" => build_database_backend(storage_url),
            scheme => Err(AppError::ConfigLoadError(format!(
                "Unsupported storage scheme: {scheme}, supported schemes are: memory://, local://, postgres://"
            ))),
        }
    }

    /// Returns a session for the provided token, if it exists and is still valid.
    pub async fn get(&self, token: &str) -> Option<Session> {
        let session = self.load(token).await?;
        if session.is_expired() {
            self.drop_token(token).await;
            return None;
        }
        Some(session)
    }

    /// Returns an existing session if it is still valid.
    pub async fn get_existing(&self, token: Option<&str>) -> Option<Session> {
        match token {
            Some(token) => self.get(token).await,
            None => None,
        }
    }

    /// Inserts or updates a session in the store.
    pub async fn insert(&self, session: Session) {
        match self {
            SessionStore::InMemory(inner) => {
                inner
                    .write()
                    .insert(session.token().to_exposed_string(), session);
            }
            #[cfg(feature = "database")]
            SessionStore::Database(pool) => {
                if let Err(err) = session_db::upsert(pool, &session).await {
                    error!("failed to persist session: {err}");
                }
            }
        }
    }

    /// Removes a session by its token. Returns the removed session, if any.
    pub async fn remove(&self, token: &str) -> Option<Session> {
        match self {
            SessionStore::InMemory(inner) => inner.write().remove(token),
            #[cfg(feature = "database")]
            SessionStore::Database(pool) => {
                let session = session_db::load(pool, token).await.ok().flatten();
                let _ = session_db::delete(pool, token).await;
                session
            }
        }
    }

    /// Creates, stores, and returns a new session.
    pub async fn create_new(&self, locale: Locale) -> Session {
        let session = Session::new_with_locale(locale);
        self.insert(session.clone()).await;
        session
    }

    /// Sync only the `csrf_tokens` state of a session to the backend.
    ///
    /// Used by the session middleware after a handler runs so CSRF tokens
    /// issued or consumed during the request (which mutate a shared
    /// `Arc<RwLock<…>>`) are durable even when the handler itself doesn't
    /// re-insert the session. In-memory is a no-op because the `Arc` is
    /// already shared across requests.
    pub async fn sync_csrf_tokens(
        &self,
        #[cfg_attr(not(feature = "database"), allow(unused_variables))] session: &Session,
    ) {
        match self {
            SessionStore::InMemory(_) => {
                // Arc sharing already keeps the in-memory copy in sync.
            }
            #[cfg(feature = "database")]
            SessionStore::Database(pool) => {
                if let Err(err) = session_db::sync_csrf(pool, session).await {
                    error!("failed to sync csrf tokens: {err}");
                }
            }
        }
    }

    /// Removes all expired sessions from the store.
    pub async fn cleanup_expired(&self) {
        match self {
            SessionStore::InMemory(inner) => {
                let mut sessions = inner.write();
                sessions.retain(|_, session| !session.is_expired());
            }
            #[cfg(feature = "database")]
            SessionStore::Database(pool) => {
                if let Err(err) = session_db::cleanup_expired(pool).await {
                    error!("failed to cleanup expired sessions: {err}");
                }
            }
        }
    }

    async fn load(&self, token: &str) -> Option<Session> {
        match self {
            SessionStore::InMemory(inner) => inner.read().get(token).cloned(),
            #[cfg(feature = "database")]
            SessionStore::Database(pool) => session_db::load(pool, token).await.ok().flatten(),
        }
    }

    async fn drop_token(&self, token: &str) {
        match self {
            SessionStore::InMemory(inner) => {
                inner.write().remove(token);
            }
            #[cfg(feature = "database")]
            SessionStore::Database(pool) => {
                let _ = session_db::delete(pool, token).await;
            }
        }
    }
}

#[cfg(feature = "database")]
fn build_database_backend(storage_url: &str) -> Result<SessionStore, AppError> {
    let pool = sqlx::PgPool::connect_lazy(storage_url)?;
    Ok(SessionStore::Database(pool))
}

#[cfg(not(feature = "database"))]
fn build_database_backend(_storage_url: &str) -> Result<SessionStore, AppError> {
    Err(AppError::ConfigLoadError(
        "Database storage disabled (enable feature \"database\")".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::session::session_idle_timeout;
    use chrono::{Duration, Utc};

    /// Returns an existing session by token.
    #[tokio::test]
    async fn get_existing_returns_session() {
        let store = SessionStore::default();
        let session = Session::new_test();
        let token = session.token().to_exposed_string();
        store.insert(session.clone()).await;

        let loaded = store.get_existing(Some(&token)).await;

        assert_eq!(loaded, Some(session));
    }

    /// Creates and stores a new session when requested.
    #[tokio::test]
    async fn create_new_inserts_session() {
        let store = SessionStore::default();
        let session = store.create_new(Locale::default()).await;

        let loaded = store.get(session.token().expose()).await;

        assert_eq!(loaded, Some(session));
    }

    /// Removes expired sessions on lookup.
    #[tokio::test]
    async fn get_removes_expired_sessions() {
        let store = SessionStore::default();
        let mut session = Session::new_test();
        session.last_activity = Utc::now() - session_idle_timeout() - Duration::seconds(1);
        let token = session.token().to_exposed_string();
        store.insert(session).await;

        let loaded = store.get(&token).await;

        assert!(loaded.is_none());
    }

    /// Removes expired sessions in bulk cleanup.
    #[tokio::test]
    async fn cleanup_expired_removes_stale_sessions() {
        let store = SessionStore::default();
        let mut expired = Session::new_test();
        expired.last_activity = Utc::now() - session_idle_timeout() - Duration::seconds(1);
        let active = Session::new_test();
        let expired_token = expired.token().to_exposed_string();
        let active_token = active.token().to_exposed_string();
        store.insert(expired).await;
        store.insert(active).await;

        store.cleanup_expired().await;

        assert!(store.get(&expired_token).await.is_none());
        assert!(store.get(&active_token).await.is_some());
    }

    #[test]
    fn from_storage_url_memory_is_in_memory() {
        let store = SessionStore::from_storage_url("memory://").unwrap();
        assert!(matches!(store, SessionStore::InMemory(_)));
    }

    #[test]
    fn from_storage_url_local_falls_back_to_memory() {
        // local:// with any path — disk is not a valid session backend, so we
        // expect an in-memory fallback regardless of directory presence.
        let store = SessionStore::from_storage_url("local:///does-not-matter").unwrap();
        assert!(matches!(store, SessionStore::InMemory(_)));
    }

    #[test]
    fn from_storage_url_rejects_unsupported_scheme() {
        match SessionStore::from_storage_url("s3://bucket") {
            Err(AppError::ConfigLoadError(_)) => {}
            Ok(_) => panic!("expected an error for unsupported scheme"),
            Err(err) => panic!("unexpected error variant: {err:?}"),
        }
    }
}
