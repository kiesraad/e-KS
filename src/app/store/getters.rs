use crate::{
    AppError, AppStore, ElectionConfig,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    list_submitters::{ListSubmitter, ListSubmitterId},
    persons::{Person, PersonId},
    political_groups::PoliticalGroup,
};

use crate::store::StoreEvent;

impl AppStore {
    pub fn get_election(&self) -> ElectionConfig {
        self.election
    }

    pub fn get_candidate_lists(&self) -> Vec<CandidateList> {
        let data = self.data.read();

        let mut lists: Vec<CandidateList> = data.candidate_lists.values().cloned().collect();

        lists.sort_unstable_by_key(|l| l.created_at);

        lists
    }

    pub fn get_political_group(&self) -> PoliticalGroup {
        let data = self.data.read();

        data.political_group.clone()
    }

    pub fn get_persons(&self) -> Vec<Person> {
        let data = self.data.read();

        data.persons.values().cloned().collect()
    }

    pub fn get_sorted_persons(&self) -> Vec<Person> {
        let data = self.data.read();

        let mut persons: Vec<Person> = data.persons.values().cloned().collect();

        persons.sort();

        persons
    }

    pub fn get_authorised_agents(&self) -> Vec<AuthorisedAgent> {
        let data = self.data.read();

        data.authorised_agents.values().cloned().collect()
    }

    pub fn get_substitute_submitters(&self) -> Vec<ListSubmitter> {
        let data = self.data.read();

        data.substitute_submitters
            .iter()
            .cloned()
            .map(|mut submitter| {
                submitter.is_substitute = true;
                submitter
            })
            .collect()
    }

    pub fn get_person_count(&self) -> usize {
        let data = self.data.read();

        data.persons.len()
    }

    pub fn get_candidate_list_count(&self) -> usize {
        let data = self.data.read();

        data.candidate_lists.len()
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

    pub fn get_list_submitter(&self) -> ListSubmitter {
        let data = self.data.read();

        let mut submitter = data.list_submitter.clone();
        submitter.is_substitute = false;
        submitter
    }

    pub fn get_substitute_submitter(
        &self,
        substitute_submitter_id: ListSubmitterId,
    ) -> Result<ListSubmitter, AppError> {
        let data = self.data.read();

        match data
            .substitute_submitters
            .iter()
            .find(|submitter| submitter.id == substitute_submitter_id)
        {
            Some(submitter) => {
                let mut submitter = submitter.clone();
                submitter.is_substitute = true;
                Ok(submitter)
            }
            None => Err(AppError::GenericNotFound),
        }
    }

    pub fn count_candidate_lists(&self, person_id: PersonId) -> usize {
        let data = self.data.read();

        data.candidate_lists
            .values()
            .filter(|list| list.candidates.contains(&person_id))
            .count()
    }

    pub fn get_events(&self) -> Vec<StoreEvent<crate::AppEvent>> {
        let data = self.data.read();
        data.events.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{AppStore, list_submitters::ListSubmitterId, test_utils::sample_list_submitter};

    #[tokio::test]
    async fn substitute_submitters_remain_in_order() {
        let store = AppStore::new_for_test();

        for i in 0..100 {
            let mut sub_submitter = sample_list_submitter(ListSubmitterId::new());
            sub_submitter.name.last_name = i.to_string().parse().unwrap();
            sub_submitter.create_substitute(&store).await.unwrap();
        }

        for (i, s) in store.get_substitute_submitters().iter().enumerate() {
            assert_eq!(s.name.last_name.to_string(), i.to_string());
        }
    }

    #[tokio::test]
    async fn getters_set_substitute_flag() {
        let store = AppStore::new_for_test();

        let main_submitter = sample_list_submitter(ListSubmitterId::new());
        let substitute_submitter = sample_list_submitter(ListSubmitterId::new());

        main_submitter.update(&store).await.unwrap();
        substitute_submitter
            .create_substitute(&store)
            .await
            .unwrap();

        assert!(!store.get_list_submitter().is_substitute);
        assert!(store.get_substitute_submitters()[0].is_substitute);
        assert!(
            store
                .get_substitute_submitter(substitute_submitter.id)
                .unwrap()
                .is_substitute
        );
    }
}
