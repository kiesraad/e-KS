use crate::{
    AppError, AppStore,
    authorised_agents::AuthorisedAgent,
    common::{Problematic, Severity},
    list_submitters::ListSubmitter,
    political_groups::PoliticalGroup,
};

#[derive(Clone, Debug)]
pub struct PoliticalGroupSteps {
    pub authorised_agents: Vec<AuthorisedAgent>,
    pub list_submitter: ListSubmitter,
    pub substitute_submitters: Vec<ListSubmitter>,

    pub list_designation_state: &'static str,
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

        let basic_info_empty = political_group.is_basic_info_empty();

        Ok(Self {
            list_designation_state: Self::list_designation_state(&political_group),
            basic_state: Self::basic_state(basic_info_empty, &political_group),
            authorised_agents_state: Self::authorised_agents_state(
                basic_info_empty,
                &authorised_agents,
            ),
            submitters_state: Self::submitters_state(
                authorised_agents.is_empty(),
                &list_submitter,
                &substitute_submitters,
            ),
            authorised_agents,
            list_submitter,
            substitute_submitters,
        })
    }

    fn list_designation_state(political_group: &PoliticalGroup) -> &'static str {
        if political_group.list_designation.is_none() {
            "empty"
        } else {
            "ok"
        }
    }

    fn basic_state(basic_info_empty: bool, political_group: &PoliticalGroup) -> &'static str {
        if basic_info_empty {
            "empty"
        } else {
            political_group.highest_severity_class()
        }
    }

    fn authorised_agents_state(
        fine_if_empty: bool,
        authorised_agents: &[AuthorisedAgent],
    ) -> &'static str {
        if authorised_agents.is_empty() {
            return if fine_if_empty { "empty" } else { "warning" };
        }

        match authorised_agents
            .iter()
            .filter_map(Problematic::highest_severity)
            .max()
        {
            None => "ok",
            Some(severity) => severity.class(),
        }
    }

    fn submitters_state(
        fine_if_empty: bool,
        list_submitter: &ListSubmitter,
        substitute_submitters: &[ListSubmitter],
    ) -> &'static str {
        if list_submitter.is_empty() {
            return if fine_if_empty { "empty" } else { "error" };
        }

        match substitute_submitters
            .iter()
            .filter_map(Problematic::highest_severity)
            .chain(list_submitter.highest_severity())
            .chain(substitute_submitters.is_empty().then_some(Severity::Info))
            .max()
        {
            None => "ok",
            Some(severity) => severity.class(),
        }
    }
}
