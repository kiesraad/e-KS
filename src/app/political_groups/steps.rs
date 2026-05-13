use crate::{
    AppError, AppStore,
    authorised_agents::AuthorisedAgent,
    list_submitters::ListSubmitter,
    submit::{Problematic, Severity},
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
            basic_state: if political_group.is_basic_info_empty() {
                "empty"
            } else {
                political_group.highest_severity_class()
            },
            authorised_agents_state: if authorised_agents.is_empty() {
                if political_group.is_basic_info_empty() {
                    "empty"
                } else {
                    "warning"
                }
            } else {
                match authorised_agents
                    .iter()
                    .filter_map(Problematic::highest_severity)
                    .max()
                {
                    None => "ok",
                    Some(severity) => severity.class(),
                }
            },
            submitters_state: if list_submitter.is_empty() {
                if political_group.is_basic_info_empty() {
                    "empty"
                } else {
                    "error"
                }
            } else {
                let mut issues = Vec::new();
                if let Some(issue) = list_submitter.highest_severity() {
                    issues.push(issue);
                }
                if substitute_submitters.is_empty() {
                    issues.push(Severity::Info);
                }

                match substitute_submitters
                    .iter()
                    .filter_map(Problematic::highest_severity)
                    .chain(issues)
                    .max()
                {
                    None => "ok",
                    Some(severity) => severity.class(),
                }
            },
            authorised_agents,
            list_submitter,
            substitute_submitters,
        })
    }
}
