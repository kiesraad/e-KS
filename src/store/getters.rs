use crate::{
    AppError,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    list_submitters::{ListSubmitter, ListSubmitterId},
    persons::{Person, PersonId},
    political_groups::PoliticalGroup,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
};

use super::AppStore;

impl AppStore {
    pub fn get_candidate_lists(&self) -> Result<Vec<CandidateList>, AppError> {
        let data = self.data.read();

        let mut lists: Vec<CandidateList> = data.candidate_lists.values().cloned().collect();

        lists.sort_unstable_by(|a, b| a.created_at.cmp(&b.created_at));

        Ok(lists)
    }

    pub fn get_political_group(&self) -> Result<PoliticalGroup, AppError> {
        let data = self.data.read();

        Ok(data.political_group.clone())
    }

    pub fn get_persons(&self) -> Result<Vec<Person>, AppError> {
        let data = self.data.read();

        Ok(data.persons.values().cloned().collect())
    }

    pub fn get_authorised_agents(&self) -> Result<Vec<AuthorisedAgent>, AppError> {
        let data = self.data.read();

        Ok(data.authorised_agents.values().cloned().collect())
    }

    pub fn get_list_submitters(&self) -> Result<Vec<ListSubmitter>, AppError> {
        let data = self.data.read();

        Ok(data.list_submitters.values().cloned().collect())
    }

    pub fn get_substitute_submitters(&self) -> Result<Vec<SubstituteSubmitter>, AppError> {
        let data = self.data.read();

        Ok(data.substitute_submitters.values().cloned().collect())
    }

    pub fn get_person_count(&self) -> Result<usize, AppError> {
        let data = self.data.read();

        Ok(data.persons.len())
    }

    pub fn get_candidate_list(&self, list_id: CandidateListId) -> Result<CandidateList, AppError> {
        let data = self.data.read();

        match data.candidate_lists.get(&list_id) {
            Some(list) => Ok(list.clone()),
            None => Err(AppError::GenericNotFound),
        }
    }

    pub fn get_person(&self, person_id: PersonId) -> Result<Person, AppError> {
        let data = self.data.read();

        match data.persons.get(&person_id) {
            Some(person) => Ok(person.clone()),
            None => Err(AppError::GenericNotFound),
        }
    }

    pub fn get_authorised_agent(
        &self,
        authorised_agent_id: AuthorisedAgentId,
    ) -> Result<AuthorisedAgent, AppError> {
        let data = self.data.read();

        match data.authorised_agents.get(&authorised_agent_id) {
            Some(agent) => Ok(agent.clone()),
            None => Err(AppError::GenericNotFound),
        }
    }

    pub fn get_list_submitter(
        &self,
        list_submitter_id: ListSubmitterId,
    ) -> Result<ListSubmitter, AppError> {
        let data = self.data.read();

        match data.list_submitters.get(&list_submitter_id) {
            Some(submitter) => Ok(submitter.clone()),
            None => Err(AppError::GenericNotFound),
        }
    }

    pub fn get_substitute_submitter(
        &self,
        substitute_submitter_id: SubstituteSubmitterId,
    ) -> Result<SubstituteSubmitter, AppError> {
        let data = self.data.read();

        match data.substitute_submitters.get(&substitute_submitter_id) {
            Some(submitter) => Ok(submitter.clone()),
            None => Err(AppError::GenericNotFound),
        }
    }

    pub fn count_candidate_lists(&self, person_id: PersonId) -> Result<usize, AppError> {
        let data = self.data.read();

        let count = data
            .candidate_lists
            .values()
            .filter(|list| list.candidates.contains(&person_id))
            .count();

        Ok(count)
    }

    pub fn get_last_event_id(&self) -> Result<usize, AppError> {
        let data = self.data.read();

        Ok(data.last_event_id)
    }
}
