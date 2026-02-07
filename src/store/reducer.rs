use super::AppStore;

use crate::{AppEvent, AppStoreData};

impl AppStore {
    pub(super) fn apply(event: &AppEvent, data: &mut AppStoreData) {
        match event {
            AppEvent::UpdatePoliticalGroup(pg) => {
                data.political_group = pg.clone();
            }
            AppEvent::CreatePerson(person) => {
                data.persons.insert(person.id, person.clone());
            }
            AppEvent::UpdatePerson(person) => {
                if let Some(existing) = data.persons.get_mut(&person.id) {
                    *existing = person.clone();
                }
            }
            AppEvent::DeletePerson(person_id) => {
                data.persons.remove(person_id);
            }
            AppEvent::CreateCandidateList(cl) => {
                data.candidate_lists.insert(cl.id, cl.clone());
            }
            AppEvent::UpdateCandidateList(cl) => {
                if let Some(existing) = data.candidate_lists.get_mut(&cl.id) {
                    *existing = cl.clone();
                }
            }
            AppEvent::DeleteCandidateList(cl_id) => {
                data.candidate_lists.remove(cl_id);
            }
            AppEvent::CreateAuthorisedAgent(aa) => {
                data.authorised_agents.insert(aa.id, aa.clone());
            }
            AppEvent::UpdateAuthorisedAgent(aa) => {
                if let Some(existing) = data.authorised_agents.get_mut(&aa.id) {
                    *existing = aa.clone();
                }
            }
            AppEvent::DeleteAuthorisedAgent(aa_id) => {
                data.authorised_agents.remove(aa_id);
            }
            AppEvent::CreateListSubmitter(ls) => {
                data.list_submitters.insert(ls.id, ls.clone());
            }
            AppEvent::UpdateListSubmitter(ls) => {
                if let Some(existing) = data.list_submitters.get_mut(&ls.id) {
                    *existing = ls.clone();
                }
            }
            AppEvent::DeleteListSubmitter(ls_id) => {
                data.list_submitters.remove(ls_id);
            }
            AppEvent::CreateSubstituteSubmitter(ss) => {
                data.substitute_submitters.insert(ss.id, ss.clone());
            }
            AppEvent::UpdateSubstituteSubmitter(ss) => {
                if let Some(existing) = data.substitute_submitters.get_mut(&ss.id) {
                    *existing = ss.clone();
                }
            }
            AppEvent::DeleteSubstituteSubmitter(ss_id) => {
                data.substitute_submitters.remove(ss_id);
            }
        }
    }
}
