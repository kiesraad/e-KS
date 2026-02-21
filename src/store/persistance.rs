use crate::{AppError, AppEvent, AppStore, store::AppStorePersistence};

#[cfg(feature = "database")]
use super::database::{load_from_database, update_in_database};

impl AppStore {
    pub async fn load(&self) -> Result<(), AppError> {
        match &self.persistence {
            #[cfg(feature = "database")]
            AppStorePersistence::Database(pool) => load_from_database(self, pool).await,
            AppStorePersistence::None => Ok(()),
        }
    }

    pub async fn update(&self, event: AppEvent) -> Result<(), AppError> {
        match &self.persistence {
            #[cfg(feature = "database")]
            AppStorePersistence::Database(pool) => update_in_database(self, pool, event).await,
            AppStorePersistence::None => {
                let mut data = self.data.write();
                AppStore::apply(event, &mut data);

                Ok(())
            }
        }
    }
}
