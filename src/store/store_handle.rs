//! Store handle and constructors for event-sourced data.

use std::sync::Arc;

use uuid::Uuid;

use crate::AppError;

use super::{StoreData, StoreEvent, StorePersistence};

/// Event-sourced store handle for a single stream.
pub struct Store<D> {
    /// Stream identifier used to partition events.
    pub stream_id: Uuid,
    /// Persistence backend used for load/update operations.
    pub persistence: StorePersistence,
    /// In-memory projection for the stream.
    pub(crate) data: Arc<parking_lot::RwLock<D>>,
}

impl<D> Clone for Store<D> {
    /// Clone the store handle, sharing the same underlying data and persistence.
    fn clone(&self) -> Self {
        Self {
            stream_id: self.stream_id,
            persistence: self.persistence.clone(),
            data: self.data.clone(),
        }
    }
}

impl<D> Store<D>
where
    D: StoreData,
{
    /// Create a new store and initialize persistence from the provided storage URL.
    pub async fn new(storage_url: &str) -> Result<Self, AppError> {
        Self::new_for_stream(storage_url, Uuid::new_v4()).await
    }

    /// Create a new store scoped to a specific stream ID.
    pub async fn new_for_stream(storage_url: &str, stream_id: Uuid) -> Result<Self, AppError> {
        let persistence = StorePersistence::from_storage_url(storage_url)?;
        persistence.init().await?;
        Self::new_for_stream_with_persistence(persistence, stream_id).await
    }

    /// Create a new store for a stream using an already-initialized persistence backend.
    pub async fn new_for_stream_with_persistence(
        persistence: StorePersistence,
        stream_id: Uuid,
    ) -> Result<Self, AppError> {
        let store = Store {
            stream_id,
            persistence,
            data: Default::default(),
        };

        store.persistence.ensure_stream(stream_id).await?;

        Ok(store)
    }

    #[cfg(feature = "database")]
    /// Create a new store backed by the provided database pool.
    pub async fn new_with_pool(pool: sqlx::PgPool) -> Result<Self, AppError> {
        Self::new_with_pool_for_stream(pool, Uuid::new_v4()).await
    }

    #[cfg(feature = "database")]
    /// Create a new store backed by the provided database pool for a stream.
    pub async fn new_with_pool_for_stream(
        pool: sqlx::PgPool,
        stream_id: Uuid,
    ) -> Result<Self, AppError> {
        let persistence = StorePersistence::Database(pool);
        persistence.init().await?;
        Self::new_for_stream_with_persistence(persistence, stream_id).await
    }

    /// Synchronize the in-memory store with the persistence by replaying any missing events.
    pub fn apply_event(&self, next_id: usize, store_event: StoreEvent<D::Event>) {
        let mut data = self.data.write();

        if data.last_event_id() >= next_id {
            // This can happen if another instance of the application processed events concurrently
            // and updated the store before this instance could acquire the write lock. In that case,
            // the store is already up-to-date and we can skip applying the event again.
            return;
        }

        data.apply(store_event);
        data.set_last_event_id(next_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestData {
        last_event_id: usize,
        applied: Vec<usize>,
    }

    impl StoreData for TestData {
        type Event = usize;

        fn apply(&mut self, event: StoreEvent<Self::Event>) {
            self.applied.push(event.payload);
        }

        fn last_event_id(&self) -> usize {
            self.last_event_id
        }

        fn set_last_event_id(&mut self, event_id: usize) {
            self.last_event_id = event_id;
        }
    }

    fn test_store() -> Store<TestData> {
        Store {
            stream_id: Uuid::new_v4(),
            persistence: StorePersistence::None,
            data: Arc::new(parking_lot::RwLock::new(TestData::default())),
        }
    }

    #[test]
    fn apply_event_updates_projection_and_last_event_id() {
        let store = test_store();

        store.apply_event(1, StoreEvent::new(1, 42));

        let data = store.data.read();
        assert_eq!(data.last_event_id, 1);
        assert_eq!(data.applied, vec![42]);
    }

    #[test]
    fn apply_event_skips_when_already_up_to_date() {
        let store = test_store();

        {
            let mut data = store.data.write();
            data.last_event_id = 2;
        }

        store.apply_event(1, StoreEvent::new(1, 7));

        let data = store.data.read();
        assert_eq!(data.last_event_id, 2);
        assert!(data.applied.is_empty());
    }
}
