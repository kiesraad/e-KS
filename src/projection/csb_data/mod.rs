mod event;
mod extractor;
mod getters;

pub use event::CsbEvent;
pub use getters::WithCorrections;

use std::collections::{HashMap, hash_map::Entry};

use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::structs::political_groups::PoliticalGroup;
use crate::{
    PgStoreData, Scope,
    store::{StoreData, StoreEvent},
    structs::{
        common::{Appellation, UtcDateTime},
        csb::{Correction, Omission, OmissionId, PersonCorrection, PersonCorrectionDelta},
        persons::PersonId,
    },
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
    pub(crate) csb_corrected_persons: HashMap<PersonId, PersonCorrectionDelta>,
    pub(crate) csb_corrected_appellation: Option<Appellation>,
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
            CsbEvent::UpdateCorrection(correction) => self.apply_correction(correction),
        }
    }

    fn events(&self) -> &[StoreEvent<Self::Event>] {
        &self.events
    }

    fn scope() -> Scope {
        Scope::ImportedByCsb
    }
}

impl CsbStoreData {
    /// Record a CSB correction. Correcting a value back to the one already in
    /// the paper-corrected projection undoes the correction instead.
    fn apply_correction(&mut self, correction: Correction) {
        match correction {
            Correction::Appellation(appellation) => {
                let appellation = Some(appellation);

                self.csb_corrected_appellation =
                    if self.paper_corrected_data.political_group.appellation == appellation {
                        None
                    } else {
                        appellation
                    };
            }
            Correction::Person(person_id, correction) => {
                self.apply_person_correction(person_id, correction)
            }
        }
    }

    fn apply_person_correction(&mut self, person_id: PersonId, correction: PersonCorrection) {
        // A correction on an unknown person is dropped: there is nothing to
        // correct it against.
        let changes = self
            .paper_corrected_data
            .persons
            .get(&person_id)
            .is_some_and(|person| correction.changes(person));

        if changes {
            self.csb_corrected_persons
                .entry(person_id)
                .or_default()
                .add_correction(correction);
        } else if let Entry::Occupied(mut entry) = self.csb_corrected_persons.entry(person_id) {
            entry.get_mut().remove_correction(&correction);
            if entry.get().get_corrections().is_empty() {
                entry.remove_entry();
            }
        }
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
    use std::{collections::HashSet, str::FromStr};

    use super::*;
    use crate::{
        PgEvent, StreamId,
        structs::{
            common::{Initials, LastName, PlaceOfResidence},
            csb::PersonCorrection,
            persons::Person,
        },
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
            data.paper_corrected_data.political_group.appellation,
            sample_political_group().appellation
        );
    }

    #[test]
    fn paper_corrected_update_applies_only_to_the_corrected_projection() {
        let mut data = CsbStoreData::default();
        data.apply(StoreEvent::new(1, import_event()));

        let mut corrected_group = sample_political_group();
        corrected_group.appellation = Some("Gecorrigeerde Naam".parse().unwrap());
        data.apply(StoreEvent::new(
            2,
            CsbEvent::PaperCorrectedUpdate(Box::new(PgEvent::UpdatePoliticalGroup(
                corrected_group.clone(),
            ))),
        ));

        assert_eq!(
            data.paper_corrected_data.political_group.appellation,
            corrected_group.appellation
        );
        // The imported snapshot stays untouched.
        assert_eq!(
            data.imported_data.political_group.appellation,
            sample_political_group().appellation
        );
    }

    #[test]
    fn undoing_csb_correction_on_person_removes_csb_correction() {
        let mut data = CsbStoreData::default();
        let person = sample_person(PersonId::new());

        data.paper_corrected_data
            .persons
            .insert(person.id, person.clone());
        data.apply(StoreEvent::new(
            1,
            CsbEvent::UpdateCorrection(Correction::Person(
                person.id,
                PersonCorrection::Initials(Initials::from_str("A.B.").unwrap()),
            )),
        ));
        data.apply(StoreEvent::new(
            2,
            CsbEvent::UpdateCorrection(Correction::Person(
                person.id,
                PersonCorrection::LastName(LastName::from_str("Smit").unwrap()),
            )),
        ));

        assert_eq!(data.csb_corrected_persons.len(), 1);
        assert_eq!(
            data.csb_corrected_persons
                .get(&person.id)
                .unwrap()
                .get_corrections()
                .len(),
            2
        );

        data.apply(StoreEvent::new(
            3,
            CsbEvent::UpdateCorrection(Correction::Person(
                person.id,
                PersonCorrection::LastName(person.name.last_name),
            )),
        ));

        assert_eq!(data.csb_corrected_persons.len(), 1);
        assert_eq!(
            data.csb_corrected_persons
                .get(&person.id)
                .unwrap()
                .get_corrections()
                .len(),
            1
        );

        data.apply(StoreEvent::new(
            4,
            CsbEvent::UpdateCorrection(Correction::Person(
                person.id,
                PersonCorrection::Initials(person.name.initials),
            )),
        ));
        assert_eq!(data.csb_corrected_persons.len(), 0);
    }

    #[test]
    fn undoing_csb_correction_on_appellation_removes_csb_correction() {
        let mut data = CsbStoreData::default();
        data.paper_corrected_data.political_group.appellation =
            Some(Appellation::from_str("Partij").unwrap());
        data.csb_corrected_appellation =
            Some(Appellation::from_str("Gecorrigeerde Partij").unwrap());

        data.apply(StoreEvent::new(
            1,
            CsbEvent::UpdateCorrection(Correction::Appellation(
                Appellation::from_str("Partij").unwrap(),
            )),
        ));

        assert_eq!(data.csb_corrected_appellation, None);
    }

    #[test]
    fn correction_appellation_sets_corrected_appellation() {
        let mut data = CsbStoreData::default();
        data.apply(StoreEvent::new(1, import_event()));

        let appellation: Appellation = "Gecorrigeerde Naam".parse().unwrap();
        data.apply(StoreEvent::new(
            2,
            CsbEvent::UpdateCorrection(Correction::Appellation(appellation.clone())),
        ));

        assert_eq!(data.csb_corrected_appellation, Some(appellation));
    }

    #[test]
    fn correction_person_accumulates_multiple_corrections() {
        let person_id = PersonId::new();
        let person = sample_person(person_id);

        let mut data = CsbStoreData::default();
        data.apply(StoreEvent::new(1, import_event_with_person(person)));

        let corrections = [
            PersonCorrection::Initials("X.Y.Z.".parse().unwrap()),
            PersonCorrection::LastName("Bakker".parse().unwrap()),
            PersonCorrection::DateOfBirth("15-06-1985".parse().unwrap()),
            PersonCorrection::PlaceOfResidence(PlaceOfResidence::Known("Amsterdam".to_string())),
        ];

        // After each event the accumulated set contains all corrections so far.
        let mut expected = HashSet::new();
        for (i, correction) in corrections.into_iter().enumerate() {
            data.apply(StoreEvent::new(
                i + 2,
                CsbEvent::UpdateCorrection(Correction::Person(person_id, correction.clone())),
            ));
            expected.insert(correction);

            let corrected = data.csb_corrected_persons.get(&person_id).unwrap();
            assert_eq!(corrected.get_corrections(), expected);
        }
    }
}
