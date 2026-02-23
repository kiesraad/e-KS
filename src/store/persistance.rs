use super::{Store, StoreData, StoreEvent, StorePersistence};
use crate::AppError;
use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature = "database")]
use super::database::{load_from_database, update_in_database};
use super::filesystem::{replay_from_file, update_in_filesystem};

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
