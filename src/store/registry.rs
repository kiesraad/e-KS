//! Registry for creating and caching stores by (stream_id, election).
//!
//! Ensures each (stream, election) pair has a single shared `Store` instance within
//! the process, and provides an optional initialization hook for first-time loads.

use std::{
    collections::{HashMap, HashSet},
    future::Future,
    sync::Arc,
};

use parking_lot::RwLock;
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

use super::{Store, StoreData, StorePersistence, encryption::EventEncryption};
use crate::{AppError, ElectionConfig, Scope};

type StoreKey = (Uuid, ElectionConfig);
type StoreMap<D> = Arc<RwLock<HashMap<StoreKey, Store<D>>>>;

/// Cache of per-(stream, election) stores backed by a shared persistence backend.
pub struct StoreRegistry<D>
where
    D: StoreData,
    D::Event: Serialize + DeserializeOwned,
{
    persistence: StorePersistence,
    encryption: EventEncryption,
    /// Scope recorded on every stream row this registry creates. A registry
    /// serves a single store type, which corresponds to a single scope.
    scope: Scope,
    inner: StoreMap<D>,
}

impl<D> Clone for StoreRegistry<D>
where
    D: StoreData,
    D::Event: Serialize + DeserializeOwned,
{
    fn clone(&self) -> Self {
        Self {
            persistence: self.persistence.clone(),
            encryption: self.encryption.clone(),
            scope: self.scope,
            inner: self.inner.clone(),
        }
    }
}

impl<D> StoreRegistry<D>
where
    D: StoreData,
    D::Event: Serialize + DeserializeOwned,
{
    /// Create a new registry for stores backed by the given storage URL. Every
    /// stream row it creates is recorded with `scope`.
    pub async fn new(
        storage_url: String,
        encryption: EventEncryption,
        scope: Scope,
    ) -> Result<Self, AppError> {
        let persistence = StorePersistence::from_storage_url(&storage_url)?;
        persistence.init().await?;

        Ok(Self {
            persistence,
            encryption,
            scope,
            inner: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a registry that shares an already-initialized persistence backend
    /// (e.g. the same Postgres pool) with another registry, but caches a
    /// different `Store<D>` projection and records its own `scope`. Skips
    /// re-initialization since the backend is assumed to be initialized already.
    pub fn with_persistence(
        persistence: StorePersistence,
        encryption: EventEncryption,
        scope: Scope,
    ) -> Self {
        Self {
            persistence,
            encryption,
            scope,
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Expose the underlying persistence backend (used by the app to share a
    /// single PgPool across stores and sessions).
    pub fn persistence(&self) -> &StorePersistence {
        &self.persistence
    }

    /// Fetch an existing store or create and load it for the given (stream, election).
    pub async fn get_or_create(
        &self,
        stream_id: Uuid,
        election: ElectionConfig,
    ) -> Result<Store<D>, AppError> {
        self.get_or_create_with_init(stream_id, election, |_| async { Ok(()) })
            .await
    }

    /// Fetch or create a store, then run a one-time async init hook before caching.
    pub async fn get_or_create_with_init<F, Fut>(
        &self,
        stream_id: Uuid,
        election: ElectionConfig,
        init: F,
    ) -> Result<Store<D>, AppError>
    where
        F: FnOnce(Store<D>) -> Fut,
        Fut: Future<Output = Result<(), AppError>>,
    {
        let key = (stream_id, election);
        if let Some(existing) = self.inner.read().get(&key) {
            return Ok(existing.clone());
        }

        let store = Store::new_for_stream_with_persistence(
            self.persistence.clone(),
            stream_id,
            election,
            self.scope,
            &self.encryption,
        )
        .await?;
        store.load().await?;
        init(store.clone()).await?;

        let mut stores = self.inner.write();
        let entry = stores.entry(key).or_insert(store);

        Ok(entry.clone())
    }

    /// List every `(stream_id, election)` stream with the given scope.
    pub async fn streams_by_scope(
        &self,
        scope: Scope,
    ) -> Result<Vec<(Uuid, ElectionConfig)>, AppError> {
        self.persistence.streams_by_scope(scope).await
    }

    /// Check which of the given stream IDs have data (in any election), using
    /// the in-memory cache first and falling back to the persistence backend.
    pub async fn streams_with_data(&self, stream_ids: &[Uuid]) -> Result<HashSet<Uuid>, AppError> {
        let (mut found, remaining) = {
            let cached = self.inner.read();
            let mut found = HashSet::new();
            let mut remaining: HashSet<Uuid> = stream_ids.iter().copied().collect();

            for ((id, _), store) in cached.iter() {
                if remaining.contains(id) && store.data.read().last_event_id() > 0 {
                    found.insert(*id);
                    remaining.remove(id);
                }
            }

            (found, remaining.into_iter().collect::<Vec<_>>())
        };

        if !remaining.is_empty() {
            let persisted = self.persistence.streams_with_data(&remaining).await?;
            found.extend(persisted);
        }

        Ok(found)
    }

    /// List the elections under the given stream that have persisted events,
    /// consulting the in-memory cache first.
    pub async fn elections_for_stream(
        &self,
        stream_id: Uuid,
    ) -> Result<Vec<ElectionConfig>, AppError> {
        let mut found: HashSet<ElectionConfig> = {
            let cached = self.inner.read();
            cached
                .iter()
                .filter_map(|((id, election), store)| {
                    (*id == stream_id && store.data.read().last_event_id() > 0).then_some(*election)
                })
                .collect()
        };

        let persisted = self.persistence.elections_for_stream(stream_id).await?;
        found.extend(persisted);

        Ok(found.into_iter().collect())
    }
}
