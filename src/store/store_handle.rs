//! Store handle and constructors for event-sourced data.

use std::sync::Arc;

use secrecy::SecretString;
use uuid::Uuid;

use crate::{AppError, ElectionConfig};

use super::{
    StoreData, StoreEvent, StorePersistence,
    encryption::{EventCipher, EventEncryption},
};

/// Event-sourced store handle for a single (stream, election) pair.
pub struct Store<D> {
    /// Stream identifier. One stream per user; events are partitioned by
    /// `(stream_id, election)`.
    pub stream_id: Uuid,
    /// Election this store instance is scoped to.
    pub election: ElectionConfig,
    /// Persistence backend used for load/update operations.
    pub persistence: StorePersistence,
    /// Per-(stream, election) cipher for encrypting/decrypting event payloads.
    pub(crate) cipher: EventCipher,
    /// In-memory projection for the stream.
    pub(crate) data: Arc<parking_lot::RwLock<D>>,
}

impl<D> Clone for Store<D> {
    /// Clone the store handle, sharing the same underlying data and persistence.
    fn clone(&self) -> Self {
        Self {
            stream_id: self.stream_id,
            election: self.election,
            persistence: self.persistence.clone(),
            cipher: self.cipher.clone(),
            data: self.data.clone(),
        }
    }
}

impl<D> Store<D>
where
    D: StoreData,
{
    /// Create a temporary store in memory
    pub async fn new_for_temp_stream(election: ElectionConfig) -> Self {
        let stream_id = Uuid::new_v4();
        Store {
            stream_id,
            election,
            persistence: StorePersistence::None,
            cipher: EventEncryption::new(&SecretString::from("temp"))
                .derive_cipher(stream_id, election),
            data: Arc::new(parking_lot::RwLock::new(D::default())),
        }
    }

    /// Create a new store scoped to a specific (stream_id, election) pair.
    pub async fn new_for_stream(
        storage_url: &str,
        stream_id: Uuid,
        election: ElectionConfig,
        encryption: &EventEncryption,
    ) -> Result<Self, AppError> {
        let persistence = StorePersistence::from_storage_url(storage_url)?;
        persistence.init().await?;
        Self::new_for_stream_with_persistence(persistence, stream_id, election, encryption).await
    }

    /// Create a new store for a stream using an already-initialized persistence backend.
    pub async fn new_for_stream_with_persistence(
        persistence: StorePersistence,
        stream_id: Uuid,
        election: ElectionConfig,
        encryption: &EventEncryption,
    ) -> Result<Self, AppError> {
        let cipher = encryption.derive_cipher(stream_id, election);
        let store = Store {
            stream_id,
            election,
            persistence,
            cipher,
            data: Arc::new(parking_lot::RwLock::new(D::default())),
        };

        store.persistence.ensure_stream(stream_id, election).await?;

        Ok(store)
    }

    #[cfg(feature = "database")]
    /// Create a new store backed by the provided database pool for a (stream, election).
    pub async fn new_with_pool_for_stream(
        pool: sqlx::PgPool,
        stream_id: Uuid,
        election: ElectionConfig,
        encryption: &EventEncryption,
    ) -> Result<Self, AppError> {
        let persistence = StorePersistence::Database(pool);
        persistence.init().await?;
        Self::new_for_stream_with_persistence(persistence, stream_id, election, encryption).await
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    const TEST_ELECTION: ElectionConfig = ElectionConfig::EK27;

    #[derive(Default)]
    struct TestData {
        last_event_id: usize,
        last_event_hash: [u8; 32],
        applied: Vec<usize>,
    }

    impl StoreData for TestData {
        type Event = usize;

        fn apply(&mut self, event: StoreEvent<Self::Event>) {
            self.last_event_id = event.event_id;
            self.last_event_hash = event.hash;
            self.applied.push(event.payload);
        }

        fn last_event_id(&self) -> usize {
            self.last_event_id
        }

        fn last_event_hash(&self) -> [u8; 32] {
            self.last_event_hash
        }
    }

    fn test_encryption() -> EventEncryption {
        EventEncryption::new(&SecretString::from("test-secret"))
    }

    fn test_store() -> Store<TestData> {
        let encryption = test_encryption();
        let stream_id = Uuid::new_v4();
        Store {
            stream_id,
            election: TEST_ELECTION,
            persistence: StorePersistence::None,
            cipher: encryption.derive_cipher(stream_id, TEST_ELECTION),
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
