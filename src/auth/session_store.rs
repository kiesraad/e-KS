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

use crate::{AppError, Locale, Session, auth::session::hash_token};

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

    /// Returns the session for `token` if it exists and is still valid.
    ///
    /// A storage failure is surfaced as `Err` rather than mapped to "no
    /// session", so a database outage trips the maintenance gate instead of
    /// looking like a signed-out user. A valid-but-expired session is dropped
    /// and reported as `Ok(None)`.
    pub async fn get(&self, token: &str) -> Result<Option<Session>, AppError> {
        // The cookie carries the raw token; the store is keyed by its hash.
        let token_hash = hash_token(token);
        let Some(session) = self.load(&token_hash).await? else {
            return Ok(None);
        };
        if session.is_expired() {
            self.drop_token(&token_hash).await;
            return Ok(None);
        }
        Ok(Some(session))
    }

    /// Like [`Self::get`], but accepts an optional token; `None` yields
    /// `Ok(None)` without touching the store.
    pub async fn get_existing(&self, token: Option<&str>) -> Result<Option<Session>, AppError> {
        match token {
            Some(token) => self.get(token).await,
            None => Ok(None),
        }
    }

    /// Inserts or updates a session in the store.
    pub async fn insert(&self, session: Session) {
        match self {
            SessionStore::InMemory(inner) => {
                inner
                    .write()
                    .insert(session.token_hash().to_string(), session);
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
        let token_hash = hash_token(token);
        match self {
            SessionStore::InMemory(inner) => inner.write().remove(&token_hash),
            #[cfg(feature = "database")]
            SessionStore::Database(pool) => {
                let session = session_db::load(pool, &token_hash).await.ok().flatten();
                let _ = session_db::delete(pool, &token_hash).await;
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

    async fn load(&self, token_hash: &str) -> Result<Option<Session>, AppError> {
        match self {
            SessionStore::InMemory(inner) => Ok(inner.read().get(token_hash).cloned()),
            #[cfg(feature = "database")]
            SessionStore::Database(pool) => session_db::load(pool, token_hash).await,
        }
    }

    async fn drop_token(&self, token_hash: &str) {
        match self {
            SessionStore::InMemory(inner) => {
                inner.write().remove(token_hash);
            }
            #[cfg(feature = "database")]
            SessionStore::Database(pool) => {
                let _ = session_db::delete(pool, token_hash).await;
            }
        }
    }
}

/// Periodically evict expired sessions, bounding how long expired rows linger
/// beyond the lazy per-lookup cleanup (matters for the Postgres backend).
pub async fn run_session_sweeper(sessions: SessionStore) {
    const SWEEP_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5 * 60);

    let mut ticker = tokio::time::interval(SWEEP_INTERVAL);
    loop {
        ticker.tick().await;
        sessions.cleanup_expired().await;
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
        let token = session.token_string();
        store.insert(session.clone()).await;

        let loaded = store
            .get_existing(Some(&token))
            .await
            .expect("load session");

        assert_eq!(loaded, Some(session));
    }

    /// Creates and stores a new session when requested.
    #[tokio::test]
    async fn create_new_inserts_session() {
        let store = SessionStore::default();
        let session = store.create_new(Locale::default()).await;

        let loaded = store
            .get(&session.token_string())
            .await
            .expect("load session");

        assert_eq!(loaded, Some(session));
    }

    /// Removes expired sessions on lookup.
    #[tokio::test]
    async fn get_removes_expired_sessions() {
        let store = SessionStore::default();
        let mut session = Session::new_test();
        session.last_activity = Utc::now() - session_idle_timeout() - Duration::seconds(1);
        let token = session.token_string();
        store.insert(session).await;

        let loaded = store.get(&token).await.expect("load session");

        assert!(loaded.is_none());
    }

    /// Removes expired sessions in bulk cleanup.
    #[tokio::test]
    async fn cleanup_expired_removes_stale_sessions() {
        let store = SessionStore::default();
        let mut expired = Session::new_test();
        expired.last_activity = Utc::now() - session_idle_timeout() - Duration::seconds(1);
        let active = Session::new_test();
        let expired_token = expired.token_string();
        let active_token = active.token_string();
        store.insert(expired).await;
        store.insert(active).await;

        store.cleanup_expired().await;

        assert!(store.get(&expired_token).await.expect("load").is_none());
        assert!(store.get(&active_token).await.expect("load").is_some());
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

    /// Postgres round-trip: hash-at-rest, NameID/UA persistence, `created_at`
    /// fixed across a touch, and `remove`.
    #[cfg(feature = "database")]
    #[cfg_attr(not(feature = "db-tests"), ignore = "requires database")]
    #[sqlx::test(migrations = false)]
    async fn database_backend_roundtrips_session(pool: sqlx::PgPool) -> Result<(), AppError> {
        crate::store::database::migrate(&pool).await?;
        let store = SessionStore::Database(pool);

        let mut session = Session::new_test();
        session.saml_name_id = Some("nid-1".to_string());
        session.set_user_agent_hash("ua-hash-xyz".to_string());
        let token = session.token_string();
        let created_at = session.created_at;
        store.insert(session).await;

        // Look up by the raw cookie token (hashed internally).
        let loaded = store.get(&token).await?.expect("session present");
        assert_eq!(loaded.saml_name_id.as_deref(), Some("nid-1"));
        assert_eq!(loaded.user_agent_hash.as_deref(), Some("ua-hash-xyz"));
        assert_eq!(loaded.token_hash(), hash_token(&token));
        assert!(
            loaded.reveal_token().is_none(),
            "a reloaded session must not carry its raw token"
        );

        // A touch must not reset created_at, even carrying a bogus one.
        let mut touched = loaded;
        touched.last_activity = Utc::now();
        touched.created_at = created_at + Duration::hours(1);
        store.insert(touched).await;
        let reloaded = store.get(&token).await?.expect("still present");
        // Tolerance: Postgres TIMESTAMPTZ truncates the Rust DateTime's nanoseconds.
        assert!(
            (reloaded.created_at - created_at).abs() < Duration::milliseconds(1),
            "created_at must be fixed across touches (got {}, expected ~{created_at})",
            reloaded.created_at,
        );

        store.remove(&token).await;
        assert!(store.get(&token).await?.is_none());
        Ok(())
    }
}
