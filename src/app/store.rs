use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    AppEvent, ElectionConfig,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    common::UtcDateTime,
    list_submitters::{ListSubmitter, ListSubmitterId},
    persons::{Person, PersonId},
    political_groups::PoliticalGroup,
    store::{StoreData, StoreEvent},
};

/// Event-sourced domain projection for a single stream.
#[derive(Debug, Serialize, Deserialize)]
pub struct AppStoreData {
    pub(crate) election: ElectionConfig,
    pub(crate) political_group: PoliticalGroup,
    pub(crate) persons: HashMap<PersonId, Person>,
    pub(crate) candidate_lists: HashMap<CandidateListId, CandidateList>,
    pub(crate) authorised_agents: HashMap<AuthorisedAgentId, AuthorisedAgent>,
    pub(crate) list_submitters: HashMap<ListSubmitterId, ListSubmitter>,
    pub(crate) substitute_submitters: HashMap<ListSubmitterId, ListSubmitter>,

    // Download path, file name, downloader id
    pub(crate) downloaded_files: Vec<(String, String, CandidateListId)>,
    pub(crate) events: Vec<StoreEvent<AppEvent>>,
}

impl StoreData for AppStoreData {
    type Event = AppEvent;
    type Init = ElectionConfig;

    fn new(election: ElectionConfig) -> Self {
        Self {
            election,
            political_group: PoliticalGroup::default(),
            persons: HashMap::new(),
            candidate_lists: HashMap::new(),
            authorised_agents: HashMap::new(),
            list_submitters: HashMap::new(),
            substitute_submitters: HashMap::new(),
            downloaded_files: Vec::new(),
            events: Vec::new(),
        }
    }

    fn apply(&mut self, event: StoreEvent<AppEvent>) {
        self.events.push(event.clone());

        let StoreEvent {
            payload,
            created_at,
            ..
        } = event;
        let event_time = UtcDateTime::from(created_at);

        match payload {
            AppEvent::UpdatePoliticalGroup(pg) => {
                self.political_group = pg;
            }
            AppEvent::CreatePerson(mut person) => {
                person.updated_at = event_time;
                self.persons.insert(person.id, person);
            }
            AppEvent::CreatePersonPersonalData {
                person_id,
                name,
                personal_data,
            } => {
                let person = Person {
                    id: person_id,
                    name,
                    personal_data,
                    updated_at: event_time,
                    ..Default::default()
                };
                self.persons.insert(person_id, person);
            }
            AppEvent::UpdatePerson(mut person) => {
                person.updated_at = event_time;
                let person_id = person.id;
                self.persons.entry(person_id).and_modify(|existing| {
                    *existing = person;
                });
            }
            AppEvent::UpdatePersonPersonalData {
                person_id,
                name,
                personal_data,
            } => {
                self.persons.entry(person_id).and_modify(|existing| {
                    existing.name = name;
                    existing.personal_data = personal_data;
                    existing.updated_at = event_time;
                });
            }
            AppEvent::UpdatePersonAddress { person_id, address } => {
                self.persons.entry(person_id).and_modify(|existing| {
                    existing.address = address;
                    existing.updated_at = event_time;
                });
            }
            AppEvent::UpdatePersonRepresentative {
                person_id,
                representative,
            } => {
                self.persons.entry(person_id).and_modify(|existing| {
                    existing.representative = representative;
                    existing.updated_at = event_time;
                });
            }
            AppEvent::DeletePerson { person_id } => {
                self.candidate_lists
                    .values_mut()
                    .for_each(|list| list.candidates.retain(|id| *id != person_id));

                self.persons.remove(&person_id);
            }
            AppEvent::CreateCandidateList(mut cl) => {
                cl.created_at = event_time;
                self.candidate_lists.insert(cl.id, cl);
            }
            AppEvent::UpdateCandidateListDistricts {
                list_id,
                electoral_districts,
            } => {
                self.candidate_lists.entry(list_id).and_modify(|existing| {
                    existing.electoral_districts = electoral_districts;
                });
            }
            AppEvent::UpdateCandidateListOrder {
                list_id,
                candidates,
            } => {
                self.candidate_lists.entry(list_id).and_modify(|existing| {
                    existing.candidates = candidates;
                });
            }
            AppEvent::UpdateCandidateListSubmitters {
                list_id,
                list_submitter_id,
                substitute_list_submitter_ids,
            } => {
                self.candidate_lists.entry(list_id).and_modify(|existing| {
                    existing.list_submitter_id = list_submitter_id;
                    existing.substitute_list_submitter_ids = substitute_list_submitter_ids;
                });
            }
            AppEvent::AddCandidateToCandidateList { list_id, person_id } => {
                self.candidate_lists.entry(list_id).and_modify(|existing| {
                    if !existing.candidates.contains(&person_id) {
                        existing.candidates.push(person_id);
                    }
                });
            }
            AppEvent::RemoveCandidateFromCandidateList { list_id, person_id } => {
                self.candidate_lists.entry(list_id).and_modify(|existing| {
                    existing.candidates.retain(|id| *id != person_id);
                });
            }
            AppEvent::DeleteCandidateList(cl_id) => {
                self.candidate_lists.remove(&cl_id);
            }
            AppEvent::CreateAuthorisedAgent(aa) => {
                self.authorised_agents.insert(aa.id, aa);
            }
            AppEvent::UpdateAuthorisedAgent(aa) => {
                let aa_id = aa.id;
                self.authorised_agents.entry(aa_id).and_modify(|existing| {
                    *existing = aa;
                });
            }
            AppEvent::DeleteAuthorisedAgent(aa_id) => {
                self.authorised_agents.remove(&aa_id);
            }
            AppEvent::CreateListSubmitter(ls) => {
                self.list_submitters.insert(ls.id, ls);
            }
            AppEvent::UpdateListSubmitter(ls) => {
                let ls_id = ls.id;
                self.list_submitters.entry(ls_id).and_modify(|existing| {
                    *existing = ls;
                });
            }
            AppEvent::DeleteListSubmitter {
                list_submitter_id: ls_id,
            } => {
                self.candidate_lists
                    .values_mut()
                    .filter(|list| list.list_submitter_id == Some(ls_id))
                    .for_each(|list| {
                        list.list_submitter_id = None;
                    });

                self.list_submitters.remove(&ls_id);
            }
            AppEvent::CreateSubstituteSubmitter(ss) => {
                self.substitute_submitters.insert(ss.id, ss);
            }
            AppEvent::UpdateSubstituteSubmitter(ss) => {
                let ss_id = ss.id;
                self.substitute_submitters
                    .entry(ss_id)
                    .and_modify(|existing| {
                        *existing = ss;
                    });
            }
            AppEvent::DeleteSubstituteSubmitter {
                substitute_submitter_id: ss_id,
            } => {
                self.candidate_lists
                    .values_mut()
                    .filter(|list| list.substitute_list_submitter_ids.contains(&ss_id))
                    .for_each(|list| {
                        list.substitute_list_submitter_ids.retain(|id| *id != ss_id);
                    });

                self.substitute_submitters.remove(&ss_id);
            }

            AppEvent::DeveloperLogin { .. }
            | AppEvent::DownloadFile { .. }
            | AppEvent::ExportCsv { .. }
            | AppEvent::ImportCsv { .. } => {
                // Only the serialized event are relevant for logging
            }
        }
    }

    fn last_event_id(&self) -> usize {
        self.events.last().map(|e| e.event_id).unwrap_or(0)
    }
}

#[cfg(test)]
impl crate::store::Store<AppStoreData> {
    pub fn new_for_test() -> Self {
        Self::new_for_test_with_election(crate::ElectionConfig::EK27)
    }

    pub fn new_for_test_with_election(election: crate::ElectionConfig) -> Self {
        let mut data = AppStoreData::new(election);
        data.political_group = crate::test_utils::sample_political_group();

        let encryption =
            crate::store::EventEncryption::new(&secrecy::SecretString::from("test-encryption-key"));
        let stream_id = uuid::Uuid::new_v4();
        crate::store::Store {
            stream_id,
            election,
            persistence: crate::store::StorePersistence::None,
            cipher: encryption.derive_cipher(stream_id, election),
            data: std::sync::Arc::new(parking_lot::RwLock::new(data)),
        }
    }
}
