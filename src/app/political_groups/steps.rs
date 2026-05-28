use crate::{
    AppError, AppStore,
    app::list_designation::ListDesignation,
    common::{Problematic, Severity},
    list_submitters::ListSubmitter,
    name_authorisations::NameAuthorisation,
    political_groups::PoliticalGroup,
};

#[derive(Clone, Debug)]
pub struct PoliticalGroupSteps {
    pub name_authorisations: Vec<NameAuthorisation>,
    pub list_submitter: ListSubmitter,
    pub substitute_submitters: Vec<ListSubmitter>,
    pub is_blank: bool,

    pub list_designation_state: &'static str,
    pub basic_state: &'static str,
    pub name_authorisations_state: &'static str,
    pub submitters_state: &'static str,
}

impl PoliticalGroupSteps {
    pub fn new(store: &AppStore) -> Result<Self, AppError> {
        let political_group = store.get_political_group();
        let name_authorisations = store.get_name_authorisations();
        let list_submitter = store.get_list_submitter();
        let substitute_submitters = store.get_substitute_submitters();

        let basic_info_empty = political_group.is_basic_info_empty();
        let is_blank = political_group.list_designation == Some(ListDesignation::Blank);

        Ok(Self {
            is_blank,
            list_designation_state: Self::list_designation_state(&political_group),
            basic_state: Self::basic_state(basic_info_empty, &political_group),
            name_authorisations_state: Self::name_authorisations_state(
                basic_info_empty,
                &name_authorisations,
            ),
            submitters_state: Self::submitters_state(
                name_authorisations.is_empty(),
                &list_submitter,
                &substitute_submitters,
            ),
            name_authorisations,
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

    fn name_authorisations_state(
        fine_if_empty: bool,
        name_authorisations: &[NameAuthorisation],
    ) -> &'static str {
        if name_authorisations.is_empty() {
            return if fine_if_empty { "empty" } else { "warning" };
        }

        match name_authorisations
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
