use crate::{
    AppStore,
    candidate_lists::{CandidateList, CandidateListSummary},
    common::{InfoProblems, Problematic},
    list_designation::ListDesignation,
    list_submitters::ListSubmitter,
    persons::Person,
    political_groups::PoliticalGroup,
};

impl InfoProblems {
    pub fn candidate_list_fix_path(&self, list: &CandidateList) -> String {
        match self {
            InfoProblems::FewCandidatesWithFirstName { .. }
            | InfoProblems::FewCandidatesWithoutFirstName { .. }
            | InfoProblems::FewCandidatesWithGender { .. }
            | InfoProblems::FewCandidatesWithoutGender { .. } => list.view_path().to_string(),
            _ => list.update_path().to_string(),
        }
    }

    pub fn person_fix_path(&self, person: &Person) -> String {
        match self {
            InfoProblems::IncompleteAddress { .. } => person.update_address_path().to_string(),
            _ => person.update_path().to_string(),
        }
    }

    pub fn general_fix_path(&self) -> String {
        match self {
            InfoProblems::NoSubstituteSubmitter => ListSubmitter::view_path().to_string(),
            InfoProblems::NoDesignationType => ListDesignation::update_path().to_string(),
            _ => PoliticalGroup::update_path().to_string(),
        }
    }

    pub fn find_all(store: &AppStore) -> Vec<InfoProblems> {
        let mut problems = Vec::new();
        problems.extend(Self::find_general_problems(store));
        problems.extend(Self::find_list_problems(store));

        problems
    }

    fn find_general_problems(store: &AppStore) -> Vec<InfoProblems> {
        let mut problems = Vec::new();
        if store.get_political_group().list_designation.is_none() {
            problems.push(InfoProblems::NoDesignationType);
        }
        if store.get_substitute_submitters().is_empty() {
            problems.push(InfoProblems::NoSubstituteSubmitter);
        }
        problems.extend(
            store
                .get_name_authorisations()
                .iter()
                .flat_map(|na| na.get_info_problems(())),
        );

        problems
    }

    fn find_list_problems(store: &AppStore) -> Vec<InfoProblems> {
        let mut problems = Vec::new();
        problems.extend(
            CandidateListSummary::list(store)
                .iter()
                .flat_map(|candidate_list| {
                    let mut problems = candidate_list.get_info_problems(());
                    problems.extend(candidate_list.get_deviation_problems(store));
                    problems
                }),
        );
        problems
    }
}
