mod event;
mod extractor;
mod getters;

pub use event::CsbEvent;
pub use getters::WithCorrections;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::structs::political_groups::PoliticalGroup;
use crate::{
    PgStoreData, Scope,
    common::{DisplayName, UtcDateTime},
    persons::{Person, PersonId},
    store::{StoreData, StoreEvent},
    structs::csb::{Correction, Omission, OmissionId},
};

/// Event-sourced domain projection for a single (stream, election) pair on the
/// CSB side.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CsbStoreData {
    pub(crate) imported_data: PgStoreData,
    pub(crate) paper_corrected_data: PgStoreData,
    pub(crate) events: Vec<StoreEvent<CsbEvent>>,
    pub(crate) is_examination_finished: bool,
    pub(crate) omissions: HashMap<OmissionId, Omission>,
    pub(crate) csb_corrected_persons: HashMap<PersonId, Person>,
    pub(crate) csb_corrected_display_name: Option<DisplayName>,
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
                    payload: crate::PgEvent::Import { hash: source_hash },
                    created_at,
                    hash,
                });
            }
            CsbEvent::CreateEmpty => {}
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
            CsbEvent::UpdateCorrection(correction) => match correction {
                Correction::DisplayName(display_name) => {
                    self.csb_corrected_display_name = Some(display_name);
                }
                Correction::Person(person_id, correction) => {
                    let person = self
                        .csb_corrected_persons
                        .entry(person_id)
                        .or_insert_with(|| {
                            self.imported_data
                                .persons
                                .get(&person_id)
                                .cloned()
                                .unwrap_or_default()
                        });
                    correction.apply(person);
                }
            },
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
            backend: crate::store::StoreBackend::Memory {
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

    pub fn add_candidate_list(&self, list: crate::structs::candidate_lists::CandidateList) {
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
    pub fn set_paper_corrected_candidate_list(
        &self,
        list: crate::structs::candidate_lists::CandidateList,
    ) {
        self.data
            .write()
            .paper_corrected_data
            .candidate_lists
            .insert(list.id, list);
    }

    pub fn add_person(&self, person: crate::structs::persons::Person) {
        let mut data = self.data.write();
        data.imported_data.persons.insert(person.id, person.clone());
        data.paper_corrected_data.persons.insert(person.id, person);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        PgEvent, StreamId,
        common::PlaceOfResidence,
        structs::csb::PersonCorrection,
        test_utils::{sample_person, sample_political_group},
    };

    fn import_event() -> CsbEvent {
        CsbEvent::Import {
            hash: [42; 32],
            source_stream_id: StreamId::new(),
            snapshot: Box::new(PgStoreData {
                political_group: sample_political_group(),
                ..PgStoreData::default()
            }),
        }
    }

    fn import_event_with_person(person: Person) -> CsbEvent {
        let mut snapshot = PgStoreData::default();
        snapshot.persons.insert(person.id, person);
        CsbEvent::Import {
            hash: [42; 32],
            source_stream_id: StreamId::new(),
            snapshot: Box::new(snapshot),
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
            CsbEvent::PaperCorrectedUpdate(Box::new(PgEvent::UpdatePoliticalGroup(
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

    #[test]
    fn correction_display_name_sets_corrected_display_name() {
        let mut data = CsbStoreData::default();
        data.apply(StoreEvent::new(1, import_event()));

        let display_name: DisplayName = "Gecorrigeerde Naam".parse().unwrap();
        data.apply(StoreEvent::new(
            2,
            CsbEvent::UpdateCorrection(Correction::DisplayName(display_name.clone())),
        ));

        assert_eq!(data.csb_corrected_display_name, Some(display_name));
    }

    #[test]
    fn correction_person_accumulates_multiple_corrections() {
        let person_id = PersonId::new();
        let person = sample_person(person_id);
        let original_last_name = person.name.last_name.to_string();

        let mut data = CsbStoreData::default();
        data.apply(StoreEvent::new(1, import_event_with_person(person)));

        // initials
        data.apply(StoreEvent::new(
            2,
            CsbEvent::UpdateCorrection(Correction::Person(
                person_id,
                PersonCorrection::Initials("X.Y.Z.".parse().unwrap()),
            )),
        ));
        let corrected = data.csb_corrected_persons.get(&person_id).unwrap();
        assert_eq!(corrected.name.initials.to_string(), "X.Y.Z.");
        assert_eq!(corrected.name.last_name.to_string(), original_last_name);

        // last name
        data.apply(StoreEvent::new(
            3,
            CsbEvent::UpdateCorrection(Correction::Person(
                person_id,
                PersonCorrection::LastName("Bakker".parse().unwrap()),
            )),
        ));
        let corrected = data.csb_corrected_persons.get(&person_id).unwrap();
        assert_eq!(corrected.name.initials.to_string(), "X.Y.Z.");
        assert_eq!(corrected.name.last_name.to_string(), "Bakker");

        // date of birth
        data.apply(StoreEvent::new(
            2,
            CsbEvent::UpdateCorrection(Correction::Person(
                person_id,
                PersonCorrection::DateOfBirth("15-06-1985".parse().unwrap()),
            )),
        ));

        let corrected = data.csb_corrected_persons.get(&person_id).unwrap();
        assert_eq!(corrected.name.initials.to_string(), "X.Y.Z.");
        assert_eq!(corrected.name.last_name.to_string(), "Bakker");
        assert_eq!(
            corrected
                .personal_data
                .date_of_birth
                .as_ref()
                .unwrap()
                .to_string(),
            "1985-06-15"
        );

        // place of residence
        data.apply(StoreEvent::new(
            2,
            CsbEvent::UpdateCorrection(Correction::Person(
                person_id,
                PersonCorrection::PlaceOfResidence(PlaceOfResidence::Known(
                    "Amsterdam".to_string(),
                )),
            )),
        ));

        let corrected = data.csb_corrected_persons.get(&person_id).unwrap();
        assert_eq!(corrected.name.initials.to_string(), "X.Y.Z.");
        assert_eq!(corrected.name.last_name.to_string(), "Bakker");
        assert_eq!(
            corrected
                .personal_data
                .date_of_birth
                .as_ref()
                .unwrap()
                .to_string(),
            "1985-06-15"
        );
        assert_eq!(
            corrected.personal_data.place_of_residence,
            Some(PlaceOfResidence::Known("Amsterdam".to_string()))
        );
    }
}
