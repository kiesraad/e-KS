//! Store load/update operations against the configured persistence backend.

use serde::{Serialize, de::DeserializeOwned};

use crate::AppError;

use super::{
    Store, StoreData, StoreEvent, StorePersistence,
    filesystem::{replay_from_file, update_in_filesystem},
};

#[cfg(feature = "database")]
use super::database::{load_from_database, update_in_database};

impl<D> Store<D>
where
    D: StoreData,
    D::Event: Serialize + DeserializeOwned,
{
    /// Load and replay persisted events into the in-memory store.
    pub async fn load(&self) -> Result<(), AppError> {
        match &self.persistence {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => {
                load_from_database(self, pool).await?;
            }
            StorePersistence::Local(dir) => {
                replay_from_file(self, dir).await?;
            }
            StorePersistence::None => {}
        }

        Ok(())
    }

    /// Persist an event and apply it to the in-memory store.
    pub async fn update(&self, event: D::Event) -> Result<(), AppError> {
        match &self.persistence {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => update_in_database(self, pool, event).await,
            StorePersistence::Local(dir) => update_in_filesystem(self, dir, event).await,
            StorePersistence::None => {
                let mut data = self.data.write();
                let event_id = data.last_event_id() + 1;
                let store_event = StoreEvent::new(event_id, event);
                data.apply(store_event);
                data.set_last_event_id(event_id);

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::StorePersistence;
    use uuid::Uuid;

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
            data: std::sync::Arc::new(parking_lot::RwLock::new(TestData::default())),
        }
    }

    #[tokio::test]
    async fn update_in_memory_increments_event_id() -> Result<(), AppError> {
        let store = test_store();

        store.update(10).await?;
        store.update(11).await?;

        let data = store.data.read();
        assert_eq!(data.last_event_id, 2);
        assert_eq!(data.applied, vec![10, 11]);

        Ok(())
    }
}
