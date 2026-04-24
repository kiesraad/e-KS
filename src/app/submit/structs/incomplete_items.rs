use crate::{AppStore, ElectoralDistrict, candidate_lists::CandidateList, persons::Person};

/// Aggregation struct for everything that can be missing or incomplete for a list submission
pub struct IncompleteItems {
    pub general_items: GeneralItems,
    pub candidate_items: Vec<CandidateItems>,
    pub list_items: Vec<ListItems>,
}

impl IncompleteItems {
    pub fn find_all(store: &AppStore) -> Self {
        let political_group_items = store.get_political_group().incomplete_items();

        Self {
            general_items: GeneralItems([political_group_items].concat()), // TODO: complete in issue #607
            candidate_items: vec![], // TODO: complete in issue #605
            list_items: vec![],      // TODO: complete in issue #608
        }
    }

    pub fn models_downloadable(&self) -> bool {
        ![
            self.general_items.0.clone(),
            self.candidate_items
                .iter()
                .flat_map(|ci| ci.items.clone())
                .collect(),
            self.list_items
                .iter()
                .flat_map(|li| li.items.clone())
                .collect(),
        ]
        .concat()
        .iter()
        .any(|i| i.severity() == Severity::Error)
    }
}

pub struct GeneralItems(pub Vec<IncompleteItem>);

pub struct CandidateItems {
    pub person: Person,
    pub items: Vec<IncompleteItem>,
}

pub struct ListItems {
    pub list: CandidateList,
    pub items: Vec<IncompleteItem>,
}

#[derive(Clone, PartialEq)]
pub enum IncompleteItem {
    // candidate list
    NoCandidates,
    TooManyCandidates { actual: usize, max: usize },
    DuplicateDistricts { duplicates: Vec<ElectoralDistrict> },
    // political group
    NoLegalName,
    NoDisplayName,
}

impl IncompleteItem {
    pub fn severity(&self) -> Severity {
        match &self {
            // candidate list
            IncompleteItem::NoCandidates => Severity::Error,
            IncompleteItem::TooManyCandidates { .. } => Severity::Warn,
            IncompleteItem::DuplicateDistricts { .. } => Severity::Error,
            // political group
            IncompleteItem::NoLegalName => Severity::Warn,
            IncompleteItem::NoDisplayName => Severity::Error,
        }
    }
}

#[derive(PartialEq)]
pub enum Severity {
    Info,
    Warn,
    Error,
}

pub trait Completable {
    /// returns all incomplete items of its own and of all children
    fn incomplete_items(&self) -> Vec<IncompleteItem>;

    fn is_complete(&self) -> bool {
        self.incomplete_items().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{sample_candidate_list, sample_person},
    };

    use super::*;

    #[test]
    fn is_printable() {
        assert!(
            IncompleteItems {
                general_items: GeneralItems(vec![]),
                candidate_items: vec![],
                list_items: vec![],
            }
            .models_downloadable()
        );

        assert!(
            IncompleteItems {
                general_items: GeneralItems(vec![]),
                candidate_items: vec![],
                list_items: vec![ListItems {
                    list: sample_candidate_list(CandidateListId::new()),
                    items: vec![IncompleteItem::TooManyCandidates {
                        actual: 12,
                        max: 12
                    }],
                }],
            }
            .models_downloadable()
        );

        assert!(
            !IncompleteItems {
                general_items: GeneralItems(vec![]),
                candidate_items: vec![CandidateItems {
                    person: sample_person(PersonId::new()),
                    items: vec![IncompleteItem::NoCandidates]
                }],
                list_items: vec![],
            }
            .models_downloadable()
        );
    }
}
