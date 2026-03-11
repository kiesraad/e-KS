//! Registry for creating and caching stores by stream ID.
//!
//! Ensures each stream has a single shared `Store` instance within the process,
//! and provides an optional initialization hook for first-time loads.

use std::{collections::HashMap, future::Future, sync::Arc};

use parking_lot::RwLock;
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

use super::{Store, StoreData, StorePersistence};
use crate::AppError;

/// Cache of per-stream stores backed by a shared persistence backend.
pub struct StoreRegistry<D>
where
    D: StoreData,
    D::Event: Serialize + DeserializeOwned,
{
    persistence: StorePersistence,
    inner: Arc<RwLock<HashMap<Uuid, Store<D>>>>,
}

impl<D> Clone for StoreRegistry<D>
where
    D: StoreData,
    D::Event: Serialize + DeserializeOwned,
{
    fn clone(&self) -> Self {
        Self {
            persistence: self.persistence.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<D> StoreRegistry<D>
where
    D: StoreData,
    D::Event: Serialize + DeserializeOwned,
{
    /// Create a new registry for stores backed by the given storage URL.
    pub async fn new(storage_url: String) -> Result<Self, AppError> {
        let persistence = StorePersistence::from_storage_url(&storage_url)?;
        persistence.init().await?;

        Ok(Self {
            persistence,
            inner: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Fetch an existing store or create and load it for the given stream.
    pub async fn get_or_create(&self, stream_id: Uuid) -> Result<Store<D>, AppError> {
        self.get_or_create_with_init(stream_id, |_| async { Ok(()) })
            .await
    }

    /// Fetch or create a store, then run a one-time async init hook before caching.
    pub async fn get_or_create_with_init<F, Fut>(
        &self,
        stream_id: Uuid,
        init: F,
    ) -> Result<Store<D>, AppError>
    where
        F: FnOnce(Store<D>) -> Fut,
        Fut: Future<Output = Result<(), AppError>>,
    {
        if let Some(existing) = self.inner.read().get(&stream_id) {
            return Ok(existing.clone());
        }

        let store =
            Store::new_for_stream_with_persistence(self.persistence.clone(), stream_id).await?;
        store.load().await?;
        init(store.clone()).await?;

        let mut stores = self.inner.write();
        let entry = stores.entry(stream_id).or_insert(store);

        Ok(entry.clone())
    }
}
