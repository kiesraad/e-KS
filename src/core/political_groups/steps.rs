use crate::{
    AppError, AppStore, authorised_agents::AuthorisedAgent, list_submitters::ListSubmitter,
    substitute_list_submitters::SubstituteSubmitter,
};

#[derive(Clone, Debug)]
pub struct PoliticalGroupSteps {
    pub authorised_agents: Vec<AuthorisedAgent>,
    pub list_submitters: Vec<ListSubmitter>,
    pub substitute_submitters: Vec<SubstituteSubmitter>,

    pub basic_complete: bool,
    pub basic_empty: bool,
    pub authorised_agents_complete: bool,
    pub authorised_agents_empty: bool,
    pub submitters_complete: bool,
    pub submitters_empty: bool,
}

impl PoliticalGroupSteps {
    pub fn new(store: AppStore) -> Result<Self, AppError> {
        let political_group = store.get_political_group()?;
        let authorised_agents = store.get_authorised_agents()?;
        let list_submitters = store.get_list_submitters()?;
        let substitute_submitters = store.get_substitute_submitters()?;

        let basic_complete = political_group.is_basic_info_complete();
        let basic_empty = political_group.is_basic_info_empty();

        let authorised_agents_empty = authorised_agents.is_empty();
        let authorised_agents_complete =
            !authorised_agents_empty && authorised_agents.iter().all(AuthorisedAgent::is_complete);

        let list_submitters_empty = list_submitters.is_empty();
        let substitute_submitters_empty = substitute_submitters.is_empty();
        let submitters_empty = list_submitters_empty && substitute_submitters_empty;

        let list_submitters_complete =
            !list_submitters_empty && list_submitters.iter().all(ListSubmitter::is_complete);
        let substitute_submitters_complete = !substitute_submitters_empty
            && substitute_submitters
                .iter()
                .all(SubstituteSubmitter::is_complete);

        let submitters_complete = list_submitters_complete && substitute_submitters_complete;

        Ok(Self {
            authorised_agents,
            list_submitters,
            substitute_submitters,
            basic_complete,
            basic_empty,
            authorised_agents_complete,
            authorised_agents_empty,
            submitters_complete,
            submitters_empty,
        })
    }
}
