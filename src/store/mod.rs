use chrono::{DateTime, Utc};
use std::sync::Arc;
use url::Url;

use crate::AppError;

#[cfg(feature = "database")]
pub(crate) mod database;

mod persistance;

#[derive(Debug, Clone)]
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
    None,
}

pub struct Store<D> {
    pub persistence: StorePersistence,
    pub(crate) data: Arc<parking_lot::RwLock<D>>,
}

impl<D> Clone for Store<D> {
    /// Clone the store handle, sharing the same underlying data and persistence.
    fn clone(&self) -> Self {
        Self {
            persistence: self.persistence.clone(),
            data: self.data.clone(),
        }
    }
}

impl<D> Store<D>
where
    D: Default,
{
    /// Create a new store and initialize persistence from the provided storage URL.
    pub async fn new(storage_url: &str) -> Result<Self, AppError> {
        let persistence = StorePersistence::from_storage_url(storage_url)?;

        let store = Store {
            persistence,
            data: Default::default(),
        };

        store.persistence.init().await?;

        Ok(store)
    }

    #[cfg(feature = "database")]
    /// Create a new store backed by the provided database pool.
    pub async fn new_with_pool(pool: sqlx::PgPool) -> Result<Self, AppError> {
        let store = Store {
            persistence: StorePersistence::Database(pool),
            data: Default::default(),
        };

        store.persistence.init().await?;

        Ok(store)
    }
}

impl StorePersistence {
    /// Build a persistence backend from a storage URL.
    pub fn from_storage_url(storage_url: &str) -> Result<Self, AppError> {
        let url = Url::parse(storage_url)
            .map_err(|err| AppError::ConfigLoadError(format!("Invalid storage URL: {err}")))?;

        match url.scheme() {
            "memory" => Ok(StorePersistence::None),
            "local" => Err(AppError::ConfigLoadError(
                "Local storage is not implemented yet".to_string(),
            )),
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
            StorePersistence::None => {}
        }

        Ok(())
    }
}
