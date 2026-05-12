use crate::{
    AppError, AppStore, authorised_agents::AuthorisedAgent, list_submitters::ListSubmitter,
    submit::Problematic,
};

#[derive(Clone, Debug)]
pub struct PoliticalGroupSteps {
    pub authorised_agents: Vec<AuthorisedAgent>,
    pub list_submitter: ListSubmitter,
    pub substitute_submitters: Vec<ListSubmitter>,

    pub basic_state: &'static str,
    pub authorised_agents_state: &'static str,
    pub submitters_state: &'static str,
}

impl PoliticalGroupSteps {
    pub fn new(store: &AppStore) -> Result<Self, AppError> {
        let political_group = store.get_political_group();
        let authorised_agents = store.get_authorised_agents();
        let list_submitter = store.get_list_submitter();
        let substitute_submitters = store.get_substitute_submitters();

        Ok(Self {
            basic_state: if political_group.is_all_good() {
                "ok"
            } else if political_group.is_basic_info_empty() {
                "empty"
            } else {
                "warning"
            },
            authorised_agents_state: if !authorised_agents.is_empty()
                && authorised_agents.iter().all(AuthorisedAgent::is_complete)
            {
                "ok"
            } else if authorised_agents.is_empty() && political_group.is_basic_info_empty() {
                "empty"
            } else {
                "warning"
            },
            submitters_state: if list_submitter.is_all_good()
                && substitute_submitters.iter().all(ListSubmitter::is_all_good)
            {
                "ok"
            } else if list_submitter.is_empty() && political_group.is_basic_info_empty() {
                "empty"
            } else {
                "warning"
            },
            authorised_agents,
            list_submitter,
            substitute_submitters,
        })
    }
}
