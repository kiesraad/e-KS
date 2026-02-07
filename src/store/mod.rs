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
pub struct AppStore {
    pub pool: sqlx::PgPool,
    data: Arc<parking_lot::RwLock<AppStoreData>>,
}

impl AppStore {
    pub fn new(pool: sqlx::PgPool) -> Self {
        AppStore {
            pool,
            data: Default::default(),
        }
    }
}
