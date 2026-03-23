//! Persistence backends for the event store.

use serde::{Serialize, de::DeserializeOwned};
use std::path::PathBuf;
use url::Url;
use uuid::Uuid;

use crate::AppError;

use super::{
    Store, StoreData, StoreEvent,
    filesystem::{self, replay_from_file, update_in_filesystem},
};

#[cfg(feature = "database")]
use super::database::{self, load_from_database, update_in_database};

/// Persistence backend selection for a store.
#[derive(Clone, Debug)]
pub enum StorePersistence {
    /// PostgreSQL-backed persistence using a shared connection pool.
    #[cfg(feature = "database")]
    Database(sqlx::PgPool),
    /// Local filesystem persistence under the provided directory.
    Local(PathBuf),
    /// In-memory only (no persistence).
    None,
}

impl StorePersistence {
    /// Build a persistence backend from a storage URL.
    pub fn from_storage_url(storage_url: &str) -> Result<Self, AppError> {
        let url = Url::parse(storage_url)
            .map_err(|err| AppError::ConfigLoadError(format!("Invalid storage URL: {err}")))?;

        match url.scheme() {
            "memory" => Ok(StorePersistence::None),
            "local" => {
                let path_string = storage_url.strip_prefix("local://").unwrap_or("");
                let path = PathBuf::from(path_string);

                if !path.exists() || !path.is_dir() {
                    return Err(AppError::ConfigLoadError(format!(
                        "Local storage requires a directory path, got: {path_string}"
                    )));
                }

                Ok(StorePersistence::Local(path))
            }
            "postgres" | "postgresql" => {
                #[cfg(feature = "database")]
                {
                    let pool = sqlx::PgPool::connect_lazy(storage_url)?;
                    Ok(StorePersistence::Database(pool))
                }
                #[cfg(not(feature = "database"))]
                {
                    Err(AppError::ConfigLoadError(
                        "Database storage disabled (enable feature \"database\")".to_string(),
                    ))
                }
            }
            scheme => Err(AppError::ConfigLoadError(format!(
                "Unsupported storage scheme: {scheme}, supported schemes are: memory://, local://, postgres://"
            ))),
        }
    }

    /// Initialize the selected persistence backend (migrations, etc).
    pub async fn init(&self) -> Result<(), AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => {
                #[cfg(feature = "migrations")]
                database::migrate(pool).await?;
            }
            StorePersistence::Local(dir) => {
                filesystem::init_local(dir).await?;
            }
            StorePersistence::None => {}
        }

        Ok(())
    }

    /// Ensure the given stream exists in the selected backend.
    pub async fn ensure_stream(&self, stream_id: Uuid) -> Result<(), AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => {
                database::ensure_stream(pool, stream_id).await?;
            }
            StorePersistence::Local(dir) => {
                filesystem::ensure_stream_file(dir, stream_id).await?;
            }
            StorePersistence::None => {}
        }

        Ok(())
    }
}

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
    use std::{fs, path::PathBuf};
    use uuid::Uuid;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("store-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn from_storage_url_accepts_memory() {
        let persistence = StorePersistence::from_storage_url("memory://").unwrap();

        assert!(matches!(persistence, StorePersistence::None));
    }

    #[test]
    fn from_storage_url_accepts_local_directory() {
        let dir = temp_dir();
        let url = format!("local://{}", dir.display());

        let persistence = StorePersistence::from_storage_url(&url).unwrap();

        match persistence {
            StorePersistence::Local(path) => assert_eq!(path, dir),
            _ => panic!("expected local persistence"),
        }
    }

    #[test]
    fn from_storage_url_rejects_missing_local_directory() {
        let dir = std::env::temp_dir().join(format!("missing-{}", Uuid::new_v4()));
        let url = format!("local://{}", dir.display());

        let err = StorePersistence::from_storage_url(&url).unwrap_err();

        match err {
            AppError::ConfigLoadError(_) => {}
            _ => panic!("expected config load error"),
        }
    }

    #[test]
    fn from_storage_url_rejects_invalid_url() {
        let err = StorePersistence::from_storage_url("not a url").unwrap_err();
        match err {
            AppError::ConfigLoadError(_) => {}
            _ => panic!("expected config load error"),
        }
    }

    #[test]
    fn from_storage_url_rejects_unsupported_scheme() {
        let err = StorePersistence::from_storage_url("s3://bucket/key").unwrap_err();
        match err {
            AppError::ConfigLoadError(_) => {}
            _ => panic!("expected config load error"),
        }
    }

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
