mod event;
mod extractor;
mod getters;

pub use event::CsbEvent;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::political_groups::PoliticalGroup;
use crate::{
    AppStoreData, Scope,
    common::UtcDateTime,
    csb::{Omission, OmissionId},
    store::{StoreData, StoreEvent},
};

/// Event-sourced domain projection for a single (stream, election) pair on the
/// CSB side.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CsbStoreData {
    imported_data: AppStoreData,
    pub(crate) events: Vec<StoreEvent<CsbEvent>>,
    pub(crate) is_examination_finished: bool,
    pub(crate) omissions: HashMap<OmissionId, Omission>,
}

impl StoreData for CsbStoreData {
    type Event = CsbEvent;

    fn apply(&mut self, event: StoreEvent<CsbEvent>) {
        self.events.push(event.clone());

        let event_time = UtcDateTime::from(event.created_at);

        match event.payload {
            CsbEvent::Import { snapshot, .. } => self.imported_data = *snapshot,
            CsbEvent::SetFinished(value) => self.is_examination_finished = value,
            CsbEvent::CreateOmission(mut omission) => {
                omission.updated_at = event_time;
                self.omissions.insert(omission.id, omission);
            }
            CsbEvent::UpdateOmission(mut omission) => {
                omission.updated_at = event_time;
                let omission_id = omission.id;
                self.omissions.entry(omission_id).and_modify(|existing| {
                    *existing = omission;
                });
            }
            CsbEvent::DeleteOmission { omission_id } => {
                self.omissions.remove(&omission_id);
            }
        }
    }

    fn events(&self) -> &[StoreEvent<Self::Event>] {
        &self.events
    }

    fn scope() -> Scope {
        Scope::ImportedByCsb
    }
}

#[cfg(test)]
impl crate::CsbStore {
    pub fn new_for_test() -> Self {
        use crate::StreamId;

        crate::store::Store {
            stream_id: StreamId::new(),
            election: crate::ElectionConfig::EK27,
            backend: crate::store::persistence::StoreBackend::Memory {
                store: crate::store::memory::MemoryStore::default(),
            },
            data: std::sync::Arc::new(parking_lot::RwLock::new(CsbStoreData::default())),
        }
    }

    pub fn set_political_group(&self, political_group: PoliticalGroup) {
        self.data.write().imported_data.political_group = political_group;
    }

    pub fn add_candidate_list(&self, list: crate::candidate_lists::CandidateList) {
        self.data
            .write()
            .imported_data
            .candidate_lists
            .insert(list.id, list);
    }

    pub fn add_person(&self, person: crate::persons::Person) {
        self.data
            .write()
            .imported_data
            .persons
            .insert(person.id, person);
    }
}
