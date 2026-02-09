use super::AppStore;

use crate::{AppEvent, AppStoreData};

impl AppStore {
    pub(super) fn apply(event: AppEvent, data: &mut AppStoreData) {
        match event {
            AppEvent::UpdatePoliticalGroup(pg) => {
                data.political_group = pg.clone();
            }
            AppEvent::CreatePerson(person) => {
                data.persons.insert(person.id, person.clone());
            }
            AppEvent::UpdatePerson(person) => {
                data.persons.entry(person.id).and_modify(|p| *p = person);
            }
            AppEvent::UpdatePersonPersonalInfo(personal_info) => {
                data.persons
                    .entry(personal_info.person_id)
                    .and_modify(|existing| {
                        *existing = existing.update_personal_info(personal_info);
                    });
            }
            AppEvent::UpdatePersonAddress {
                person_id,
                address,
                updated_at,
            } => {
                data.persons.entry(person_id).and_modify(|existing| {
                    existing.address = address;
                    existing.updated_at = updated_at;
                });
            }
            AppEvent::UpdatePersonRepresentative {
                person_id,
                representative,
                updated_at,
            } => {
                data.persons.entry(person_id).and_modify(|existing| {
                    existing.representative = representative;
                    existing.updated_at = updated_at;
                });
            }
            AppEvent::DeletePerson(person_id) => {
                data.persons.remove(&person_id);
            }
            AppEvent::CreateCandidateList(cl) => {
                data.candidate_lists.insert(cl.id, cl);
            }
            AppEvent::UpdateCandidateList(cl) => {
                data.candidate_lists.entry(cl.id).and_modify(|existing| {
                    *existing = cl;
                });
            }
            AppEvent::UpdateCandidateListDistricts {
                list_id,
                electoral_districts,
                updated_at,
            } => {
                data.candidate_lists.entry(list_id).and_modify(|existing| {
                    existing.electoral_districts = electoral_districts;
                    existing.updated_at = updated_at;
                });
            }
            AppEvent::UpdateCandidateListOrder {
                list_id,
                candidates,
                updated_at,
            } => {
                data.candidate_lists.entry(list_id).and_modify(|existing| {
                    existing.candidates = candidates;
                    existing.updated_at = updated_at;
                });
            }
            AppEvent::UpdateCandidateListSubmitters {
                list_id,
                list_submitter_id,
                substitute_list_submitter_ids,
                updated_at,
            } => {
                data.candidate_lists.entry(list_id).and_modify(|existing| {
                    existing.list_submitter_id = list_submitter_id;
                    existing.substitute_list_submitter_ids = substitute_list_submitter_ids;
                    existing.updated_at = updated_at;
                });
            }
            AppEvent::AddCandidateToCandidateList {
                list_id,
                person_id,
                updated_at,
            } => {
                data.candidate_lists.entry(list_id).and_modify(|existing| {
                    if !existing.candidates.contains(&person_id) {
                        existing.candidates.push(person_id);
                        existing.updated_at = updated_at;
                    }
                });
            }
            AppEvent::RemoveCandidateFromCandidateList {
                list_id,
                person_id,
                updated_at,
            } => {
                data.candidate_lists.entry(list_id).and_modify(|existing| {
                    existing.candidates.retain(|id| *id != person_id);
                    existing.updated_at = updated_at;
                });
            }
            AppEvent::RemoveCandidateFromAllCandidateLists {
                person_id,
                updated_at,
            } => {
                for list in data.candidate_lists.values_mut() {
                    let before = list.candidates.len();
                    list.candidates.retain(|id| *id != person_id);
                    if list.candidates.len() != before {
                        list.updated_at = updated_at;
                    }
                }
            }
            AppEvent::DeleteCandidateList(cl_id) => {
                data.candidate_lists.remove(&cl_id);
            }
            AppEvent::CreateAuthorisedAgent(aa) => {
                data.authorised_agents.insert(aa.id, aa);
            }
            AppEvent::UpdateAuthorisedAgent(aa) => {
                data.authorised_agents.entry(aa.id).and_modify(|a| *a = aa);
            }
            AppEvent::DeleteAuthorisedAgent(aa_id) => {
                data.authorised_agents.remove(&aa_id);
            }
            AppEvent::CreateListSubmitter(ls) => {
                data.list_submitters.insert(ls.id, ls);
            }
            AppEvent::UpdateListSubmitter(ls) => {
                data.list_submitters.entry(ls.id).and_modify(|s| *s = ls);
            }
            AppEvent::DeleteListSubmitter(ls_id) => {
                data.list_submitters.remove(&ls_id);
            }
            AppEvent::CreateSubstituteSubmitter(ss) => {
                data.substitute_submitters.insert(ss.id, ss);
            }
            AppEvent::UpdateSubstituteSubmitter(ss) => {
                data.substitute_submitters
                    .entry(ss.id)
                    .and_modify(|s| *s = ss);
            }
            AppEvent::DeleteSubstituteSubmitter(ss_id) => {
                data.substitute_submitters.remove(&ss_id);
            }
        }
    }
}
