use crate::{AppStore, ElectoralDistrict, candidate_lists::CandidateList, list_submitters, persons::Person};

/// Aggregation struct for everything that can be missing or incomplete for a list submission
pub struct IncompleteItems {
    pub general_items: GeneralItems,
    pub candidate_items: Vec<CandidateItems>,
    pub list_items: Vec<ListItems>,
}

pub struct GeneralItems(Vec<IncompleteItem>);

pub struct CandidateItems {
    pub person: Person,
    pub items: Vec<IncompleteItem>,
}

pub struct ListItems {
    pub list: CandidateList,
    pub items: Vec<IncompleteItem>,
}

pub enum IncompleteItem {
    // candidate list
    NoCandidates,
    TooManyCandidates { actual: usize, max: usize },
    DuplicateDistricts { duplicates: Vec<ElectoralDistrict> },
    // political group
    LongListAllowedIsNone,
    NoLegalName,
    NoDisplayName,
}

impl IncompleteItems {
    pub fn find_all(store: &AppStore) -> Self {
        let political_group = store.get_political_group();
        let authorised_agent = store.get_authorised_agents()[0]; // TODO is this correct? Is there always only one authorised agent?
        /* NOTE: we're ahead of the curve here. currently, multiple list submitters are possible 
        (separate submitters per list) but in PR #585  this will change to only one submitter */
        let list_submitter = store.get_list_submitters()[0];
        let substitute_submitters = store.get_substitute_submitters();

        Self {
            general_items: GeneralItems([].concat()),
            candidate_items: todo!(),
            list_items: todo!(),
        }
    }
}
pub trait Completable {
    /// returns all incomplete items of its own and of all child objects
    fn incomplete_items(&self) -> Vec<IncompleteItem>;
}
