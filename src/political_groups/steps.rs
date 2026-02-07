use crate::{
    authorised_agents::AuthorisedAgent, list_submitters::ListSubmitter,
    political_groups::PoliticalGroup, substitute_list_submitters::SubstituteSubmitter,
};

#[derive(Clone, Copy, Debug)]
pub struct PoliticalGroupSteps {
    pub basic_complete: bool,
    pub basic_empty: bool,
    pub authorised_agents_complete: bool,
    pub authorised_agents_empty: bool,
    pub submitters_complete: bool,
    pub submitters_empty: bool,
}

impl PoliticalGroupSteps {
    pub fn new(
        political_group: &PoliticalGroup,
        authorised_agents: &[AuthorisedAgent],
        list_submitters: &[ListSubmitter],
        substitute_submitters: &[SubstituteSubmitter],
    ) -> Self {
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

        Self {
            basic_complete,
            basic_empty,
            authorised_agents_complete,
            authorised_agents_empty,
            submitters_complete,
            submitters_empty,
        }
    }
}
