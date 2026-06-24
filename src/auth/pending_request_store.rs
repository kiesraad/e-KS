//! Storage for outstanding SAML AuthnRequest IDs used by the `InResponseTo`
//! replay check (eID §7.6.3.5 rule 4 / §9.7).
//!
//! Mirrors [`crate::SessionStore`]: in-memory by default, Postgres when
//! `STORAGE_URL` is a `postgres://` URL. Sharing the IDs across instances
//! matters for horizontal scaling: a login started on one server may have its
//! ACS callback handled by another, which must still see the pending ID to
//! validate `InResponseTo` (and to consume it so it cannot be replayed).

use auth_service::PendingRequests;
use url::Url;

use crate::AppError;

#[cfg(feature = "database")]
use crate::auth::pending_request_db;
#[cfg(feature = "database")]
use tracing::error;

/// Pending-request storage backend.
#[derive(Clone)]
pub enum PendingRequestStore {
    /// Process-local, in-memory storage. Cleared on restart.
    InMemory(PendingRequests),
    /// Postgres-backed storage, shared across instances.
    #[cfg(feature = "database")]
    Database(sqlx::PgPool),
}

impl Default for PendingRequestStore {
    fn default() -> Self {
        Self::InMemory(PendingRequests::default())
    }
}

impl PendingRequestStore {
    /// Construct a pending-request store from `STORAGE_URL`, using the same
    /// scheme rules as [`crate::SessionStore`] (disk is not a valid backend and
    /// falls back to in-memory).
    pub fn from_storage_url(storage_url: &str) -> Result<Self, AppError> {
        let url = Url::parse(storage_url)
            .map_err(|err| AppError::ConfigLoadError(format!("Invalid storage URL: {err}")))?;

        match url.scheme() {
            "memory" | "local" => Ok(Self::default()),
            "postgres" | "postgresql" => build_database_backend(storage_url),
            scheme => Err(AppError::ConfigLoadError(format!(
                "Unsupported storage scheme: {scheme}, supported schemes are: memory://, local://, postgres://"
            ))),
        }
    }

    /// Record an outgoing AuthnRequest ID for the later `InResponseTo` check.
    pub async fn register(&self, id: String) {
        match self {
            PendingRequestStore::InMemory(inner) => inner.register(id),
            #[cfg(feature = "database")]
            PendingRequestStore::Database(pool) => {
                if let Err(err) = pending_request_db::register(pool, &id).await {
                    error!("failed to persist pending AuthnRequest: {err}");
                }
            }
        }
    }

    /// Atomically validate and consume a matched AuthnRequest ID (eID §7.6.3.5
    /// rule 4 / §9.7). Returns `true` iff `id` was a still-valid outstanding
    /// request, consuming it in the same step so it cannot be replayed.
    ///
    /// A storage error fails closed (returns `false`), so the `InResponseTo`
    /// check rejects the Assertion rather than letting an unverified one
    /// through.
    pub async fn consume_if_pending(&self, id: &str) -> bool {
        match self {
            PendingRequestStore::InMemory(inner) => inner.consume_if_pending(id),
            #[cfg(feature = "database")]
            PendingRequestStore::Database(pool) => {
                match pending_request_db::consume_if_pending(pool, id).await {
                    Ok(matched) => matched,
                    Err(err) => {
                        error!("failed to consume pending AuthnRequest: {err}");
                        false
                    }
                }
            }
        }
    }
}

#[cfg(feature = "database")]
fn build_database_backend(storage_url: &str) -> Result<PendingRequestStore, AppError> {
    let pool = sqlx::PgPool::connect_lazy(storage_url)?;
    Ok(PendingRequestStore::Database(pool))
}

#[cfg(not(feature = "database"))]
fn build_database_backend(_storage_url: &str) -> Result<PendingRequestStore, AppError> {
    Err(AppError::ConfigLoadError(
        "Database storage disabled (enable feature \"database\")".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn in_memory_register_and_consume_roundtrip() {
        let store = PendingRequestStore::default();
        store.register("req-1".to_string()).await;
        store.register("req-2".to_string()).await;

        // A registered ID is matched and consumed exactly once.
        assert!(store.consume_if_pending("req-1").await);
        assert!(!store.consume_if_pending("req-1").await);
        // An unknown ID never matches.
        assert!(!store.consume_if_pending("never").await);
        // Other registered IDs remain available.
        assert!(store.consume_if_pending("req-2").await);
    }

    #[test]
    fn from_storage_url_memory_is_in_memory() {
        let store = PendingRequestStore::from_storage_url("memory://").unwrap();
        assert!(matches!(store, PendingRequestStore::InMemory(_)));
    }

    #[test]
    fn from_storage_url_local_falls_back_to_memory() {
        let store = PendingRequestStore::from_storage_url("local:///whatever").unwrap();
        assert!(matches!(store, PendingRequestStore::InMemory(_)));
    }

    #[test]
    fn from_storage_url_rejects_unsupported_scheme() {
        match PendingRequestStore::from_storage_url("s3://bucket") {
            Err(AppError::ConfigLoadError(_)) => {}
            Ok(_) => panic!("expected an error for unsupported scheme"),
            Err(err) => panic!("unexpected error variant: {err:?}"),
        }
    }
}
