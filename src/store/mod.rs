use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use url::Url;
use uuid::Uuid;

use crate::AppError;

#[cfg(feature = "database")]
pub(crate) mod database;

mod filesystem;
mod persistance;
pub mod registry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreEvent<E> {
    pub event_id: usize,
    pub payload: E,
    pub created_at: DateTime<Utc>,
}

impl<E> StoreEvent<E> {
    /// Construct a new store event with the given ID and payload.
    /// `created_at` is set to the current UTC time.
    pub fn new(event_id: usize, payload: E) -> Self {
        Self {
            event_id,
            payload,
            created_at: Utc::now(),
        }
    }

    pub fn new_at(event_id: usize, payload: E, created_at: DateTime<Utc>) -> Self {
        Self {
            event_id,
            payload,
            created_at,
        }
    }
}

pub trait StoreData: Default + Send + Sync + 'static {
    type Event;

    /// Apply a fully wrapped store event to the data projection.
    fn apply(&mut self, event: StoreEvent<Self::Event>);
    /// Return the last applied event ID for this data instance.
    fn last_event_id(&self) -> usize;
    /// Update the last applied event ID for this data instance.
    fn set_last_event_id(&mut self, event_id: usize);
}

#[derive(Clone)]
pub enum StorePersistence {
    #[cfg(feature = "database")]
    Database(sqlx::PgPool),
    Local(PathBuf),
    None,
}

pub struct Store<D> {
    pub stream_id: Uuid,
    pub persistence: StorePersistence,
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

        let store = Store {
            stream_id,
            persistence,
            data: Default::default(),
        };

        store.persistence.init().await?;
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
        let store = Store {
            stream_id,
            persistence: StorePersistence::Database(pool),
            data: Default::default(),
        };

        store.persistence.init().await?;
        store.persistence.ensure_stream(store.stream_id).await?;

        Ok(store)
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
                "Unsupported storage scheme: {scheme}"
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
