use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use url::Url;

use crate::{
    AppError,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    list_submitters::{ListSubmitter, ListSubmitterId},
    persons::{Person, PersonId},
    political_groups::PoliticalGroup,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
};

#[cfg(feature = "database")]
mod database;
mod event;
mod getters;
mod persistance;
mod reducer;

pub use event::AppEvent;

#[cfg(test)]
mod tests;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AppStoreData {
    political_group: PoliticalGroup,
    persons: HashMap<PersonId, Person>,
    candidate_lists: HashMap<CandidateListId, CandidateList>,
    authorised_agents: HashMap<AuthorisedAgentId, AuthorisedAgent>,
    list_submitters: HashMap<ListSubmitterId, ListSubmitter>,
    substitute_submitters: HashMap<SubstituteSubmitterId, SubstituteSubmitter>,
    // Track the last event ID applied to this store instance for synchronization purposes
    last_event_id: usize,
}

#[derive(Clone)]
pub enum AppStorePersistence {
    #[cfg(feature = "database")]
    Database(sqlx::PgPool),
    None,
}

#[derive(Clone)]
pub struct AppStore {
    pub persistence: AppStorePersistence,
    data: Arc<parking_lot::RwLock<AppStoreData>>,
}

impl AppStore {
    pub async fn new(storage_url: &str) -> Result<Self, AppError> {
        let persistence = AppStorePersistence::from_storage_url(storage_url)?;

        let store = AppStore {
            persistence,
            data: Default::default(),
        };

        store.persistence.init().await?;

        Ok(store)
    }

    #[cfg(feature = "database")]
    pub async fn new_with_pool(pool: sqlx::PgPool) -> Result<Self, AppError> {
        let store = AppStore {
            persistence: AppStorePersistence::Database(pool),
            data: Default::default(),
        };

        store.persistence.init().await?;

        Ok(store)
    }

    #[cfg(test)]
    pub async fn new_for_test() -> Self {
        let political_group_id = crate::political_groups::PoliticalGroupId::new();
        let political_group = crate::test_utils::sample_political_group(political_group_id);

        let store = AppStore {
            persistence: AppStorePersistence::None,
            data: Default::default(),
        };

        political_group.update(&store).await.expect("store update");

        store
    }
}

impl AppStorePersistence {
    pub fn from_storage_url(storage_url: &str) -> Result<Self, AppError> {
        let url = Url::parse(storage_url)
            .map_err(|err| AppError::ConfigLoadError(format!("Invalid storage URL: {err}")))?;

        match url.scheme() {
            "memory" => Ok(AppStorePersistence::None),
            "local" => Err(AppError::ConfigLoadError(
                "Local storage is not implemented yet".to_string(),
            )),
            "postgres" | "postgresql" => {
                #[cfg(feature = "database")]
                {
                    let pool = sqlx::PgPool::connect_lazy(storage_url)?;
                    Ok(AppStorePersistence::Database(pool))
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

    pub async fn init(&self) -> Result<(), AppError> {
        match self {
            #[cfg(feature = "database")]
            AppStorePersistence::Database(pool) => {
                #[cfg(feature = "migrations")]
                database::migrate(pool).await?;
            }
            AppStorePersistence::None => {}
        }

        Ok(())
    }
}
