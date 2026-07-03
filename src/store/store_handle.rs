//! Store handle and constructors for event-sourced data.

use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::{AppError, ElectionConfig, StreamId};

use super::{
    StoreData, StoreEvent, StorePersistence, encryption::EventEncryption, memory::MemoryStore,
    persistence::StoreBackend,
};

/// Event-sourced store handle for a single (stream, election) pair.
pub struct Store<D> {
    /// Stream identifier. One stream per user; events are partitioned by
    /// `(stream_id, election)`.
    pub stream_id: StreamId,
    /// Election this store instance is scoped to.
    pub election: ElectionConfig,
    /// Persistence target paired with its cipher. Persisting backends are
    /// always encrypted; see [`StoreBackend`].
    pub(crate) backend: StoreBackend,
    /// In-memory projection for the stream.
    pub(crate) data: Arc<parking_lot::RwLock<D>>,
}

impl<D> Clone for Store<D> {
    /// Clone the store handle, sharing the same underlying data and persistence.
    fn clone(&self) -> Self {
        Self {
            stream_id: self.stream_id,
            election: self.election,
            backend: self.backend.clone(),
            data: self.data.clone(),
        }
    }
}

impl<D> Store<D>
where
    D: StoreData,
{
    /// Create a temporary, in-memory store with no persistence.
    ///
    /// An in-memory store never writes events out, so it has no cipher
    /// (see [`StoreBackend::Memory`]).
    pub fn new_for_temp_stream(election: ElectionConfig) -> Self {
        Store {
            stream_id: StreamId::new(),
            election,
            backend: StoreBackend::Memory {
                store: MemoryStore::default(),
            },
            data: Arc::new(parking_lot::RwLock::new(D::default())),
        }
    }

    /// Create a new store scoped to a specific (stream_id, election) pair.
    pub async fn new_for_stream(
        storage_url: &str,
        stream_id: StreamId,
        election: ElectionConfig,
        encryption: &EventEncryption,
    ) -> Result<Self, AppError> {
        let persistence = StorePersistence::from_storage_url(storage_url)?;
        persistence.init().await?;
        Self::new_for_stream_with_persistence(persistence, stream_id, election, encryption).await
    }

    /// Create a new store for a stream using an already-initialized persistence
    /// backend. `scope` is recorded on the stream row when it is first created.
    pub async fn new_for_stream_with_persistence(
        persistence: StorePersistence,
        stream_id: StreamId,
        election: ElectionConfig,
        encryption: &EventEncryption,
    ) -> Result<Self, AppError> {
        persistence
            .ensure_stream(stream_id, election, D::scope())
            .await?;

        let cipher = encryption.derive_cipher(stream_id, election);
        Ok(Store {
            stream_id,
            election,
            backend: persistence.into_backend(cipher),
            data: Arc::new(parking_lot::RwLock::new(D::default())),
        })
    }

    #[cfg(feature = "database")]
    /// Create a new store backed by the provided database pool for a (stream, election).
    pub async fn new_with_pool_for_stream(
        pool: sqlx::PgPool,
        stream_id: StreamId,
        election: ElectionConfig,
        encryption: &EventEncryption,
    ) -> Result<Self, AppError> {
        let persistence = StorePersistence::Database(pool);
        persistence.init().await?;
        Self::new_for_stream_with_persistence(persistence, stream_id, election, encryption).await
    }

    /// Apply a single event to the in-memory projection.
    ///
    /// No-op if the projection is already at or past this event: another
    /// instance of the application may have processed and applied it before
    /// this caller acquired the write lock.
    pub fn apply_event(&self, store_event: StoreEvent<D::Event>) {
        let mut data = self.data.write();

        if data.last_event_id() >= store_event.event_id {
            return;
        }

        data.apply(store_event);
    }

    /// Build a [`StoreEvent`] for a freshly persisted event and apply it to the
    /// in-memory projection via [`Store::apply_event`].
    pub(crate) fn apply_persisted_event(
        &self,
        event_id: usize,
        payload: D::Event,
        created_at: DateTime<Utc>,
        hash: [u8; 32],
    ) {
        self.apply_event(StoreEvent {
            event_id,
            payload,
            created_at,
            hash,
        });
    }

    /// Last event ID applied to the in-memory projection, or 0 if none.
    pub fn current_event_id(&self) -> usize {
        self.data.read().last_event_id()
    }

    /// Chain hash of the last applied event, or
    /// [`GENESIS_HASH`](crate::store::GENESIS_HASH) if none.
    pub fn current_event_hash(&self) -> [u8; 32] {
        self.data.read().last_event_hash()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Event, Scope};

    use super::*;

    const TEST_ELECTION: ElectionConfig = ElectionConfig::EK27;

    #[derive(Default)]
    struct TestData {
        events: Vec<StoreEvent<usize>>,
        applied: Vec<usize>,
    }

    impl Event for usize {
        fn category(&self) -> &'static str {
            "number"
        }

        fn key(&self) -> &'static str {
            ""
        }

        fn description(&self, _locale: crate::Locale) -> String {
            "a number".to_string()
        }

        fn details(&self) -> String {
            self.to_string()
        }
    }

    impl StoreData for TestData {
        type Event = usize;

        fn apply(&mut self, event: StoreEvent<Self::Event>) {
            self.applied.push(event.payload);
            self.events.push(event);
        }

        fn events(&self) -> &[StoreEvent<Self::Event>] {
            &self.events
        }

        fn scope() -> Scope {
            Scope::PoliticalGroup
        }
    }

    fn test_store() -> Store<TestData> {
        Store {
            stream_id: StreamId::new(),
            election: TEST_ELECTION,
            backend: StoreBackend::Memory {
                store: MemoryStore::default(),
            },
            data: Arc::new(parking_lot::RwLock::new(TestData::default())),
        }
    }

    #[test]
    fn apply_event_updates_projection_and_last_event_id() {
        let store = test_store();

        store.apply_event(StoreEvent::new(1, 42));

        let data = store.data.read();
        assert_eq!(data.last_event_id(), 1);
        assert_eq!(data.applied, vec![42]);
    }

    #[test]
    fn apply_event_skips_when_already_up_to_date() {
        let store = test_store();

        {
            let mut data = store.data.write();
            data.events.push(StoreEvent::new(2, 0));
        }

        store.apply_event(StoreEvent::new(1, 7));

        let data = store.data.read();
        assert_eq!(data.last_event_id(), 2);
        assert!(data.applied.is_empty());
    }
}
