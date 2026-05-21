use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    AppEvent,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    common::UtcDateTime,
    list_submitters::ListSubmitter,
    persons::{Person, PersonId},
    political_groups::PoliticalGroup,
    store::{StoreData, StoreEvent},
};

/// Event-sourced domain projection for a single (stream, election) pair.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppStoreData {
    pub(crate) political_group: PoliticalGroup,
    pub(crate) persons: HashMap<PersonId, Person>,
    pub(crate) candidate_lists: HashMap<CandidateListId, CandidateList>,
    pub(crate) authorised_agents: HashMap<AuthorisedAgentId, AuthorisedAgent>,
    pub(crate) list_submitter: ListSubmitter,
    pub(crate) substitute_submitters: Vec<ListSubmitter>,

    // Download path, file name, downloader id
    pub(crate) downloaded_files: Vec<(String, String, CandidateListId)>,
    pub(crate) events: Vec<StoreEvent<AppEvent>>,
}

impl StoreData for AppStoreData {
    type Event = AppEvent;

    fn apply(&mut self, event: StoreEvent<AppEvent>) {
        self.events.push(event.clone());

        let StoreEvent {
            payload,
            created_at,
            ..
        } = event;
        let event_time = UtcDateTime::from(created_at);

        match payload {
            AppEvent::UpdatePoliticalGroup(pg) => self.political_group = pg,

            event @ (AppEvent::CreatePerson(_)
            | AppEvent::CreatePersonPersonalData { .. }
            | AppEvent::UpdatePerson(_)
            | AppEvent::UpdatePersonPersonalData { .. }
            | AppEvent::UpdatePersonAddress { .. }
            | AppEvent::UpdatePersonRepresentative { .. }
            | AppEvent::DeletePerson { .. }) => self.apply_person_event(event, event_time),

            event @ (AppEvent::CreateCandidateList(_)
            | AppEvent::UpdateCandidateListDistricts { .. }
            | AppEvent::UpdateCandidateListOrder { .. }
            | AppEvent::AddCandidateToCandidateList { .. }
            | AppEvent::RemoveCandidateFromCandidateList { .. }
            | AppEvent::DeleteCandidateList(_)) => {
                self.apply_candidate_list_event(event, event_time)
            }

            event @ (AppEvent::CreateAuthorisedAgent(_)
            | AppEvent::UpdateAuthorisedAgent(_)
            | AppEvent::DeleteAuthorisedAgent(_)) => self.apply_authorised_agent_event(event),

            event @ (AppEvent::UpdateListSubmitter(_)
            | AppEvent::CreateSubstituteSubmitter(_)
            | AppEvent::UpdateSubstituteSubmitter(_)
            | AppEvent::DeleteSubstituteSubmitter { .. }) => self.apply_submitter_event(event),

            // Only the serialized event is relevant for logging.
            AppEvent::DeveloperLogin { .. }
            | AppEvent::DownloadFile { .. }
            | AppEvent::ExportCsv { .. }
            | AppEvent::ImportCsv { .. } => {}
        }
    }

    fn last_event_id(&self) -> usize {
        self.events.last().map(|e| e.event_id).unwrap_or(0)
    }

    fn last_event_hash(&self) -> [u8; 32] {
        self.events
            .last()
            .map(|e| e.hash)
            .unwrap_or(crate::store::GENESIS_HASH)
    }
}

impl AppStoreData {
    /// Apply a person-related event. Routed here exclusively by [`Self::apply`].
    fn apply_person_event(&mut self, event: AppEvent, event_time: UtcDateTime) {
        match event {
            AppEvent::CreatePerson(_) | AppEvent::CreatePersonPersonalData { .. } => {
                self.apply_person_create_event(event, event_time)
            }
            AppEvent::UpdatePerson(_)
            | AppEvent::UpdatePersonPersonalData { .. }
            | AppEvent::UpdatePersonAddress { .. }
            | AppEvent::UpdatePersonRepresentative { .. } => {
                self.apply_person_update_event(event, event_time)
            }
            AppEvent::DeletePerson { person_id } => {
                self.candidate_lists
                    .values_mut()
                    .for_each(|list| list.candidates.retain(|id| *id != person_id));

                self.persons.remove(&person_id);
            }
            _ => unreachable!("apply_person_event received a non-person event"),
        }
    }

    /// Apply a person-creation event. Routed here exclusively by [`Self::apply_person_event`].
    fn apply_person_create_event(&mut self, event: AppEvent, event_time: UtcDateTime) {
        match event {
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
            _ => unreachable!("apply_person_create_event received a non-create event"),
        }
    }

    /// Apply a person-update event. Routed here exclusively by [`Self::apply_person_event`].
    fn apply_person_update_event(&mut self, event: AppEvent, event_time: UtcDateTime) {
        match event {
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
            _ => unreachable!("apply_person_update_event received a non-update event"),
        }
    }

    /// Apply a candidate-list event. Routed here exclusively by [`Self::apply`].
    fn apply_candidate_list_event(&mut self, event: AppEvent, event_time: UtcDateTime) {
        match event {
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
            _ => unreachable!("apply_candidate_list_event received a non-candidate-list event"),
        }
    }

    /// Apply an authorised-agent event. Routed here exclusively by [`Self::apply`].
    fn apply_authorised_agent_event(&mut self, event: AppEvent) {
        match event {
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
            _ => unreachable!("apply_authorised_agent_event received a non-agent event"),
        }
    }

    /// Apply a list-submitter event. Routed here exclusively by [`Self::apply`].
    fn apply_submitter_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::UpdateListSubmitter(ls) => {
                self.list_submitter = ls;
            }
            AppEvent::CreateSubstituteSubmitter(ss) => {
                self.substitute_submitters.push(ss);
            }
            AppEvent::UpdateSubstituteSubmitter(ss) => {
                let ss_id = ss.id;
                if let Some(existing) = self
                    .substitute_submitters
                    .iter_mut()
                    .find(|existing| existing.id == ss_id)
                {
                    *existing = ss;
                }
            }
            AppEvent::DeleteSubstituteSubmitter {
                substitute_submitter_id: ss_id,
            } => {
                self.substitute_submitters
                    .retain(|submitter| submitter.id != ss_id);
            }
            _ => unreachable!("apply_submitter_event received a non-submitter event"),
        }
    }
}

impl crate::store::Store<AppStoreData> {
    pub fn current_event_id(&self) -> usize {
        self.data.read().last_event_id()
    }

    pub fn current_event_hash(&self) -> [u8; 32] {
        self.data.read().last_event_hash()
    }
}

#[cfg(test)]
impl crate::store::Store<AppStoreData> {
    pub fn new_for_test() -> Self {
        Self::new_for_test_with_election(crate::ElectionConfig::EK27)
    }

    pub fn new_for_test_with_election(election: crate::ElectionConfig) -> Self {
        let data = AppStoreData {
            political_group: crate::test_utils::sample_political_group(),
            ..AppStoreData::default()
        };

        crate::store::Store {
            stream_id: uuid::Uuid::new_v4(),
            election,
            backend: crate::store::persistence::StoreBackend::Memory,
            data: std::sync::Arc::new(parking_lot::RwLock::new(data)),
        }
    }
}
