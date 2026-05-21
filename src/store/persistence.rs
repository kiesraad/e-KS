//! Persistence backends for the event store.

use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};
use std::path::PathBuf;
use url::Url;
use uuid::Uuid;

use crate::{AppError, ElectionConfig};

use super::{
    Store, StoreData, StoreEvent, chain_hash,
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

    /// Ensure the given (stream, election) exists in the selected backend.
    pub async fn ensure_stream(
        &self,
        stream_id: Uuid,
        election: ElectionConfig,
    ) -> Result<(), AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => {
                database::ensure_stream(pool, stream_id, election).await?;
            }
            StorePersistence::Local(dir) => {
                filesystem::ensure_stream_file(dir, stream_id, election).await?;
            }
            StorePersistence::None => {}
        }

        Ok(())
    }

    /// Check which of the given stream IDs have any persisted events (in any election).
    pub async fn streams_with_data(
        &self,
        stream_ids: &[Uuid],
    ) -> Result<std::collections::HashSet<Uuid>, AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => {
                super::database::streams_with_data(pool, stream_ids).await
            }
            StorePersistence::Local(dir) => {
                Ok(filesystem::streams_with_data(dir, stream_ids).await)
            }
            StorePersistence::None => Ok(std::collections::HashSet::new()),
        }
    }

    /// List the elections under the given stream that have persisted events.
    pub async fn elections_for_stream(
        &self,
        stream_id: Uuid,
    ) -> Result<Vec<ElectionConfig>, AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => {
                super::database::elections_for_stream(pool, stream_id).await
            }
            StorePersistence::Local(dir) => {
                Ok(filesystem::elections_for_stream(dir, stream_id).await)
            }
            StorePersistence::None => Ok(Vec::new()),
        }
    }

    /// Verify the persistence backend is reachable. For the database backend
    /// this acquires a connection and runs `SELECT 1`; for the local backend
    /// it checks the storage directory still exists; the in-memory backend is
    /// always reachable.
    pub async fn health_check(&self) -> Result<(), AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => {
                sqlx::query("SELECT 1").execute(pool).await?;
                Ok(())
            }
            StorePersistence::Local(dir) => {
                if dir.is_dir() {
                    Ok(())
                } else {
                    Err(AppError::ConfigLoadError(format!(
                        "Local storage directory missing: {}",
                        dir.display()
                    )))
                }
            }
            StorePersistence::None => Ok(()),
        }
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
                let created_at = Utc::now();
                let prev_hash = data.last_event_hash();
                // Nothing is persisted, so the chain hash is over the plain encoding.
                let body = postcard::to_allocvec(&event).map_err(|e| {
                    AppError::ServerError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                })?;
                let hash = chain_hash(&prev_hash, event_id, created_at, &body);
                data.apply(StoreEvent {
                    event_id,
                    payload: event,
                    created_at,
                    hash,
                });

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::encryption::EventEncryption;
    use secrecy::SecretString;
    use std::{fs, path::PathBuf};
    use uuid::Uuid;

    const TEST_ELECTION: ElectionConfig = ElectionConfig::EK27;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("store-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn test_encryption() -> EventEncryption {
        EventEncryption::new(&SecretString::from("test-secret"))
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

    fn test_store() -> Store<TestData> {
        let encryption = test_encryption();
        let stream_id = Uuid::new_v4();
        Store {
            stream_id,
            election: TEST_ELECTION,
            persistence: StorePersistence::None,
            cipher: encryption.derive_cipher(stream_id, TEST_ELECTION),
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

    #[tokio::test]
    async fn correct_key_loads_persisted_events() -> Result<(), AppError> {
        let dir = temp_dir();
        let encryption = test_encryption();
        let stream_id = Uuid::new_v4();
        let persistence = StorePersistence::Local(dir.clone());

        let store = Store::<TestData>::new_for_stream_with_persistence(
            persistence.clone(),
            stream_id,
            TEST_ELECTION,
            &encryption,
        )
        .await?;

        store.update(10).await?;
        store.update(20).await?;

        let fresh = Store::<TestData>::new_for_stream_with_persistence(
            persistence,
            stream_id,
            TEST_ELECTION,
            &encryption,
        )
        .await?;
        fresh.load().await?;

        let data = fresh.data.read();
        assert_eq!(data.last_event_id, 2);
        assert_eq!(data.applied, vec![10, 20]);

        Ok(())
    }

    #[tokio::test]
    async fn wrong_secret_cannot_load_persisted_events() -> Result<(), AppError> {
        let dir = temp_dir();
        let encryption = test_encryption();
        let stream_id = Uuid::new_v4();
        let persistence = StorePersistence::Local(dir.clone());

        let store = Store::<TestData>::new_for_stream_with_persistence(
            persistence.clone(),
            stream_id,
            TEST_ELECTION,
            &encryption,
        )
        .await?;

        store.update(10).await?;
        store.update(20).await?;

        let wrong_encryption = EventEncryption::new(&SecretString::from("wrong-secret"));
        let wrong_store = Store::<TestData>::new_for_stream_with_persistence(
            persistence,
            stream_id,
            TEST_ELECTION,
            &wrong_encryption,
        )
        .await?;

        let err = wrong_store
            .load()
            .await
            .expect_err("load must fail with the wrong secret");
        assert!(matches!(err, AppError::EventDecodeError(_)));
        assert!(wrong_store.data.read().applied.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn wrong_stream_cannot_load_events_from_other_stream() -> Result<(), AppError> {
        let dir = temp_dir();
        let encryption = test_encryption();
        let stream_a = Uuid::new_v4();
        let stream_b = Uuid::new_v4();
        let persistence = StorePersistence::Local(dir.clone());

        let store_a = Store::<TestData>::new_for_stream_with_persistence(
            persistence.clone(),
            stream_a,
            TEST_ELECTION,
            &encryption,
        )
        .await?;

        store_a.update(42).await?;

        // Manually copy stream_a's file to stream_b's path so stream_b
        // tries to read data encrypted with stream_a's key.
        let src = dir.join(format!("{stream_a}_{}.bin", TEST_ELECTION.stable_id()));
        let dst = dir.join(format!("{stream_b}_{}.bin", TEST_ELECTION.stable_id()));
        std::fs::copy(&src, &dst).expect("copy stream file");

        let store_b = Store::<TestData>::new_for_stream_with_persistence(
            persistence,
            stream_b,
            TEST_ELECTION,
            &encryption,
        )
        .await?;

        let err = store_b
            .load()
            .await
            .expect_err("load must fail for events encrypted with another stream's key");
        assert!(matches!(err, AppError::EventDecodeError(_)));
        assert!(store_b.data.read().applied.is_empty());

        Ok(())
    }
}
