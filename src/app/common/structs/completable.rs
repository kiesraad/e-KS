use crate::{ElectoralDistrict, candidate_lists::CandidateList};

pub trait Completable {
    /// returns all incomplete items of its own and of all child objects
    fn incomplete_items(&self) -> Vec<IncompleteItem<'_>>;

    fn is_complete(&self) -> bool {
        self.incomplete_items().is_empty()
    }
}

#[derive(PartialEq)]
pub enum IncompleteItem<'a> {
    // candidate list
    NoCandidates { candidate_list: &'a CandidateList },
    TooManyCandidates { candidate_list: &'a CandidateList, actual: usize, max: usize },
    DuplicateDistricts {candidate_list: &'a CandidateList, duplicates: &'a Vec<ElectoralDistrict>},
    // political group
    LongListAllowedIsNone,
    NoLegalName,
    NoDisplayName,
}
