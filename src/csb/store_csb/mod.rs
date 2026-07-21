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
    pub(crate) imported_data: AppStoreData,
    pub(crate) paper_corrected_data: AppStoreData,
    pub(crate) events: Vec<StoreEvent<CsbEvent>>,
    pub(crate) is_examination_finished: bool,
    pub(crate) omissions: HashMap<OmissionId, Omission>,
}

impl StoreData for CsbStoreData {
    type Event = CsbEvent;

    fn apply(&mut self, event: StoreEvent<CsbEvent>) {
        self.events.push(event.clone());

        let event_time = UtcDateTime::from(event.created_at);
        let StoreEvent {
            event_id,
            created_at,
            hash,
            ..
        } = event;

        match event.payload {
            CsbEvent::Import {
                snapshot,
                hash: source_hash,
                ..
            } => {
                self.imported_data = *snapshot;
                self.paper_corrected_data = self.imported_data.clone();

                // Record the import as event #1 of the corrected projection,
                // so the paper-corrections audit log starts with it.
                self.paper_corrected_data.apply(StoreEvent {
                    event_id,
                    payload: crate::AppEvent::Import { hash: source_hash },
                    created_at,
                    hash,
                });
            }
            CsbEvent::PaperCorrectedUpdate(payload) => {
                // Replay the wrapped app event onto the corrected projection,
                // keeping the CSB stream's event metadata.
                self.paper_corrected_data.apply(StoreEvent {
                    event_id,
                    payload: *payload,
                    created_at,
                    hash,
                });
            }
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

    /// Test setters write both projections, mirroring the state right after an
    /// import (which seeds `paper_corrected_data` from the imported snapshot).
    pub fn set_political_group(&self, political_group: PoliticalGroup) {
        let mut data = self.data.write();
        data.imported_data.political_group = political_group.clone();
        data.paper_corrected_data.political_group = political_group;
    }

    pub fn add_candidate_list(&self, list: crate::candidate_lists::CandidateList) {
        let mut data = self.data.write();
        data.imported_data
            .candidate_lists
            .insert(list.id, list.clone());
        data.paper_corrected_data
            .candidate_lists
            .insert(list.id, list);
    }

    /// Test setter writing only the corrected projection, mirroring a list
    /// added during paper corrections.
    pub fn set_paper_corrected_candidate_list(&self, list: crate::candidate_lists::CandidateList) {
        self.data
            .write()
            .paper_corrected_data
            .candidate_lists
            .insert(list.id, list);
    }

    pub fn add_person(&self, person: crate::persons::Person) {
        let mut data = self.data.write();
        data.imported_data.persons.insert(person.id, person.clone());
        data.paper_corrected_data.persons.insert(person.id, person);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppEvent, StreamId, test_utils::sample_political_group};

    fn import_event() -> CsbEvent {
        CsbEvent::Import {
            hash: [42; 32],
            source_stream_id: StreamId::new(),
            snapshot: Box::new(AppStoreData {
                political_group: sample_political_group(),
                ..AppStoreData::default()
            }),
        }
    }

    #[test]
    fn import_seeds_paper_corrected_data_from_the_snapshot() {
        let mut data = CsbStoreData::default();

        data.apply(StoreEvent::new(1, import_event()));

        assert_eq!(
            data.paper_corrected_data.political_group.display_name,
            sample_political_group().display_name
        );
    }

    #[test]
    fn paper_corrected_update_applies_only_to_the_corrected_projection() {
        let mut data = CsbStoreData::default();
        data.apply(StoreEvent::new(1, import_event()));

        let mut corrected_group = sample_political_group();
        corrected_group.display_name = Some("Gecorrigeerde Naam".parse().unwrap());
        data.apply(StoreEvent::new(
            2,
            CsbEvent::PaperCorrectedUpdate(Box::new(AppEvent::UpdatePoliticalGroup(
                corrected_group.clone(),
            ))),
        ));

        assert_eq!(
            data.paper_corrected_data.political_group.display_name,
            corrected_group.display_name
        );
        // The imported snapshot stays untouched.
        assert_eq!(
            data.imported_data.political_group.display_name,
            sample_political_group().display_name
        );
    }
}
