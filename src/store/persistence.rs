//! Persistence backends for the event store.

use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};
use std::path::PathBuf;
use url::Url;

use crate::{AppError, ElectionConfig, Scope, StreamId};

use super::{
    Store, StoreData, StoreEvent, chain_hash,
    encryption::EventCipher,
    filesystem::{self, replay_from_file, update_in_filesystem},
    memory::{self, MemoryStore},
};

#[cfg(feature = "database")]
use super::database::{self, load_from_database, update_in_database};

/// Decryption-free metadata about a persisted stream, read from the backend's
/// index without replaying (or warming) it. The political group name is absent:
/// it lives in the encrypted payloads.
#[derive(Clone, Debug)]
pub struct StreamMeta {
    pub stream_id: StreamId,
    pub election: ElectionConfig,
    /// Number of events, i.e. the last event id (events are appended `1..=n`).
    pub event_count: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub last_event_at: Option<DateTime<Utc>>,
}

/// Persistence backend selection for a store.
#[derive(Clone, Debug)]
pub enum StorePersistence {
    /// PostgreSQL-backed persistence using a shared connection pool.
    #[cfg(feature = "database")]
    Database(sqlx::PgPool),
    /// Local filesystem persistence under the provided directory.
    Local(PathBuf),
    /// In-memory only (no durable persistence). Carries a shared index of stream
    /// scopes and event hashes so lookups resolve like the other backends.
    Memory(MemoryStore),
}

impl StorePersistence {
    /// Build a persistence backend from a storage URL.
    pub fn from_storage_url(storage_url: &str) -> Result<Self, AppError> {
        let url = Url::parse(storage_url)
            .map_err(|err| AppError::ConfigLoadError(format!("Invalid storage URL: {err}")))?;

        match url.scheme() {
            "memory" => Ok(StorePersistence::Memory(MemoryStore::default())),
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
                if let Err(err) = database::migrate(pool).await {
                    tracing::warn!(
                        "initial database migration failed; \
                         starting anyway and retrying in the background: {err}"
                    );
                }
                let _ = pool;
            }
            StorePersistence::Local(dir) => {
                filesystem::init_local(dir).await?;
            }
            StorePersistence::Memory(_) => {}
        }

        Ok(())
    }

    /// Ensure the given (stream, election) exists in the selected backend,
    /// recording its `scope` when the row is first created.
    pub async fn ensure_stream(
        &self,
        stream_id: StreamId,
        election: ElectionConfig,
        scope: Scope,
    ) -> Result<(), AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => {
                database::ensure_stream(pool, stream_id, election, scope).await?;
            }
            StorePersistence::Local(dir) => {
                filesystem::ensure_stream_file(dir, stream_id, election, scope).await?;
            }
            StorePersistence::Memory(store) => {
                memory::ensure_stream(store, stream_id, election, scope);
            }
        }

        Ok(())
    }

    /// List every `(stream_id, election)` stream with the given scope.
    ///
    /// The database backend reads each stream's recorded scope. Local file
    /// storage only ever holds political-group streams, so it lists every
    /// non-empty stream for [`Scope::PoliticalGroup`] and nothing for any other
    /// scope. The in-memory backend tracks each stream's scope in its shared
    /// index and lists the non-empty streams matching `scope`.
    pub async fn streams_by_scope(
        &self,
        scope: Scope,
    ) -> Result<Vec<(StreamId, ElectionConfig)>, AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => {
                super::database::streams_by_scope(pool, scope).await
            }
            StorePersistence::Local(dir) => Ok(filesystem::streams_by_scope(dir, scope).await),
            StorePersistence::Memory(store) => Ok(memory::streams_by_scope(store, scope)),
        }
    }

    /// List [`StreamMeta`] for every stream with the given scope, reading only
    /// each backend's index. The in-memory backend keeps no timestamps.
    pub async fn stream_metadata_by_scope(
        &self,
        scope: Scope,
    ) -> Result<Vec<StreamMeta>, AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => {
                super::database::stream_metadata_by_scope(pool, scope).await
            }
            StorePersistence::Local(dir) => filesystem::stream_metadata_by_scope(dir, scope).await,
            StorePersistence::Memory(store) => Ok(memory::stream_metadata_by_scope(store, scope)),
        }
    }

    /// List the elections under the given stream that have persisted events.
    pub async fn elections_for_stream(
        &self,
        stream_id: StreamId,
    ) -> Result<Vec<ElectionConfig>, AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => {
                super::database::elections_for_stream(pool, stream_id).await
            }
            StorePersistence::Local(dir) => {
                Ok(filesystem::elections_for_stream(dir, stream_id).await)
            }
            StorePersistence::Memory(store) => Ok(memory::elections_for_stream(store, stream_id)),
        }
    }

    /// Locate the political-group event whose chain hash begins with
    /// `hash_prefix`, returning its `(stream_id, election, event_id)`.
    ///
    /// The database backend indexes events by hash and restricts the lookup to
    /// political-group streams; local file storage only holds political-group
    /// streams, so it scans them directly. The in-memory backend scans its
    /// shared index, likewise restricted to political-group streams.
    pub async fn find_event_by_hash_prefix(
        &self,
        hash_prefix: &[u8],
    ) -> Result<Option<(StreamId, ElectionConfig, usize)>, AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => {
                database::find_event_by_hash_prefix(pool, hash_prefix).await
            }
            StorePersistence::Local(dir) => {
                filesystem::find_event_by_hash_prefix(dir, hash_prefix).await
            }
            StorePersistence::Memory(store) => {
                memory::find_event_by_hash_prefix(store, hash_prefix)
            }
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
            StorePersistence::Memory(_) => Ok(()),
        }
    }

    pub async fn migrate(&self) -> Result<(), AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(_pool) => {
                #[cfg(feature = "migrations")]
                database::migrate(_pool).await?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Verify the persistence backend is ready to serve: reachable with its
    /// schema present.
    pub async fn verify_ready(&self) -> Result<(), AppError> {
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => database::verify_schema(pool).await,
            other => other.health_check().await,
        }
    }
}

/// A store's resolved backend: a persistence target paired with the
/// per-stream [`EventCipher`].
///
/// The persisting variants (`Database`, `Local`) cannot be constructed
/// without a cipher, so events written to disk or database are *always*
/// encrypted. `Memory` carries no cipher because it never writes events out; it
/// keeps only the shared index used to answer cross-stream lookups.
#[derive(Clone, Debug)]
pub(crate) enum StoreBackend {
    /// PostgreSQL-backed, encrypted persistence.
    #[cfg(feature = "database")]
    Database {
        pool: sqlx::PgPool,
        cipher: Box<EventCipher>,
    },
    /// Local filesystem-backed, encrypted persistence.
    Local {
        dir: PathBuf,
        cipher: Box<EventCipher>,
    },
    /// In-memory only: no durable persistence and no encryption, just the shared
    /// index that records event hashes and stream scopes.
    Memory { store: MemoryStore },
}

impl StorePersistence {
    /// Pair this persistence target with a stream's cipher to form a
    /// [`StoreBackend`]. The cipher is dropped for [`StorePersistence::Memory`],
    /// since an in-memory store neither persists nor encrypts events.
    pub(crate) fn into_backend(self, cipher: EventCipher) -> StoreBackend {
        let cipher = Box::new(cipher);
        match self {
            #[cfg(feature = "database")]
            StorePersistence::Database(pool) => StoreBackend::Database { pool, cipher },
            StorePersistence::Local(dir) => StoreBackend::Local { dir, cipher },
            StorePersistence::Memory(store) => StoreBackend::Memory { store },
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
        match &self.backend {
            #[cfg(feature = "database")]
            StoreBackend::Database { pool, cipher } => {
                load_from_database(self, pool, cipher).await?;
            }
            StoreBackend::Local { dir, cipher } => {
                replay_from_file(self, dir, cipher).await?;
            }
            // The in-memory projection is the only copy of the events, so there
            // is nothing to replay back into it.
            StoreBackend::Memory { .. } => {}
        }

        Ok(())
    }

    /// Persist an event and apply it to the in-memory store.
    pub async fn update(&self, event: D::Event) -> Result<(), AppError> {
        match &self.backend {
            #[cfg(feature = "database")]
            StoreBackend::Database { pool, cipher } => {
                update_in_database(self, pool, cipher, event).await
            }
            StoreBackend::Local { dir, cipher } => {
                update_in_filesystem(self, dir, cipher, event).await
            }
            StoreBackend::Memory { store } => {
                let mut data = self.data.write();
                let event_id = data.last_event_id() + 1;
                let created_at = Utc::now();
                let prev_hash = data.last_event_hash();
                // Nothing is persisted, so the chain hash is over the plain encoding.
                let body = postcard::to_allocvec(&event).map_err(|e| {
                    AppError::ServerError(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                })?;
                let hash = chain_hash(&prev_hash, event_id, created_at, &body);
                // Record the hash in the shared index so cross-stream lookups
                // (by scope, by hash prefix) resolve like the other backends.
                memory::record_event(store, self.stream_id, self.election, event_id, hash);
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
    use crate::{Scope, store::encryption::EventEncryption};
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

        assert!(matches!(persistence, StorePersistence::Memory(_)));
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
        events: Vec<StoreEvent<usize>>,
        applied: Vec<usize>,
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
            data: std::sync::Arc::new(parking_lot::RwLock::new(TestData::default())),
        }
    }

    #[tokio::test]
    async fn update_in_memory_increments_event_id() -> Result<(), AppError> {
        let store = test_store();

        store.update(10).await?;
        store.update(11).await?;

        let data = store.data.read();
        assert_eq!(data.last_event_id(), 2);
        assert_eq!(data.applied, vec![10, 11]);

        Ok(())
    }

    #[tokio::test]
    async fn correct_key_loads_persisted_events() -> Result<(), AppError> {
        let dir = temp_dir();
        let encryption = test_encryption();
        let stream_id = StreamId::new();
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
        assert_eq!(data.last_event_id(), 2);
        assert_eq!(data.applied, vec![10, 20]);

        Ok(())
    }

    #[tokio::test]
    async fn stream_metadata_by_scope_reports_counts_and_timestamps() -> Result<(), AppError> {
        let dir = temp_dir();
        let encryption = test_encryption();
        let persistence = StorePersistence::Local(dir.clone());
        let stream_id = StreamId::new();

        let store = Store::<TestData>::new_for_stream_with_persistence(
            persistence.clone(),
            stream_id,
            TEST_ELECTION,
            &encryption,
        )
        .await?;
        store.update(10).await?;
        store.update(20).await?;

        let meta = persistence
            .stream_metadata_by_scope(Scope::PoliticalGroup)
            .await?;
        assert_eq!(meta.len(), 1);
        let entry = &meta[0];
        assert_eq!(entry.stream_id, stream_id);
        assert_eq!(entry.event_count, 2);
        let created = entry.created_at.expect("created timestamp");
        let last = entry.last_event_at.expect("last timestamp");
        assert!(created <= last);

        assert!(
            persistence
                .stream_metadata_by_scope(Scope::CentralElectoralCommittee)
                .await?
                .is_empty()
        );

        Ok(())
    }

    #[tokio::test]
    async fn wrong_secret_cannot_load_persisted_events() -> Result<(), AppError> {
        let dir = temp_dir();
        let encryption = test_encryption();
        let stream_id = StreamId::new();
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
        let stream_a = StreamId::new();
        let stream_b = StreamId::new();
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
