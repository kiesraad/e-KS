use crate::{
    candidate_lists::CandidateList, common::InfoProblems, list_designation::ListDesignation,
    list_submitters::ListSubmitter, persons::Person, political_groups::PoliticalGroup,
};

// TODO salvage the paths, throw away the rest
// TODO probably have both fix path impls be part of problematic.rs
// TODO check which of these redirect to an overlay and need the back path to be set
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
            InfoProblems::NoListDesignation => ListDesignation::update_path().to_string(),
            _ => PoliticalGroup::update_path().to_string(),
        }
    }
}
