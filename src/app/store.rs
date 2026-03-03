use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    AppEvent,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    common::UtcDateTime,
    list_submitters::{ListSubmitter, ListSubmitterId},
    persons::{Person, PersonId},
    political_groups::PoliticalGroup,
    store::{StoreData, StoreEvent},
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
};

/// Event-sourced domain projection for a single political group.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AppStoreData {
    pub(crate) political_group: PoliticalGroup,
    pub(crate) persons: HashMap<PersonId, Person>,
    pub(crate) candidate_lists: HashMap<CandidateListId, CandidateList>,
    pub(crate) authorised_agents: HashMap<AuthorisedAgentId, AuthorisedAgent>,
    pub(crate) list_submitters: HashMap<ListSubmitterId, ListSubmitter>,
    pub(crate) substitute_submitters: HashMap<SubstituteSubmitterId, SubstituteSubmitter>,
    // Track the last event ID applied to this store instance for synchronization purposes
    pub(crate) last_event_id: usize,
}

impl StoreData for AppStoreData {
    type Event = AppEvent;

    fn apply(&mut self, event: StoreEvent<AppEvent>) {
        let StoreEvent {
            event_id,
            payload,
            created_at,
        } = event;
        self.last_event_id = event_id;
        let event_time = UtcDateTime::from(created_at);

        match payload {
            AppEvent::UpdatePoliticalGroup(pg) => {
                self.political_group = pg;
            }
            AppEvent::CreatePerson(mut person) => {
                person.updated_at = event_time;
                self.persons.insert(person.id, person);
            }
            AppEvent::UpdatePerson(mut person) => {
                person.updated_at = event_time;
                let person_id = person.id;
                self.persons.entry(person_id).and_modify(|existing| {
                    *existing = person;
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
        }
    }

    fn last_event_id(&self) -> usize {
        self.last_event_id
    }

    fn set_last_event_id(&mut self, event_id: usize) {
        self.last_event_id = event_id;
    }
}

#[cfg(test)]
impl crate::store::Store<AppStoreData> {
    pub fn new_for_test() -> Self {
        let political_group =
            crate::test_utils::sample_political_group(crate::PoliticalGroupId::new());

        crate::store::Store {
            stream_id: uuid::Uuid::new_v4(),
            persistence: crate::store::StorePersistence::None,
            data: std::sync::Arc::new(parking_lot::RwLock::new(AppStoreData {
                political_group,
                ..Default::default()
            })),
        }
    }
}
