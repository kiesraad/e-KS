use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::{
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    list_submitters::{ListSubmitter, ListSubmitterId},
    persons::{Person, PersonId},
    political_groups::PoliticalGroup,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
};

mod database;
mod event;
mod getters;
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
    Database(sqlx::PgPool),
    None,
}

#[derive(Clone)]
pub struct AppStore {
    pub persistence: AppStorePersistence,
    data: Arc<parking_lot::RwLock<AppStoreData>>,
}

impl AppStore {
    pub fn new(pool: sqlx::PgPool) -> Self {
        AppStore {
            persistence: AppStorePersistence::Database(pool),
            data: Default::default(),
        }
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
