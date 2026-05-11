use crate::{
    AppStore, ElectoralDistrict,
    authorised_agents::AuthorisedAgent,
    candidate_lists::{CandidateList, CandidateListSummary},
    list_submitters::ListSubmitter,
    persons::Person,
};

/// Aggregation struct for everything that can be missing or incomplete for a list submission
#[derive(Debug)]
pub struct IncompleteItems {
    pub general_items: GeneralItems,
    pub candidate_items: Vec<PersonItems<Person>>,
    pub list_items: Vec<ListItems>,
}

impl IncompleteItems {
    pub fn find_all(store: &AppStore) -> Self {
        let candidate_lists = CandidateListSummary::list(store);

        Self {
            general_items: Self::find_general_items(store),
            candidate_items: vec![], // TODO: complete in issue #605
            list_items: candidate_lists
                .iter()
                .filter_map(|candidate_list| {
                    if candidate_list.is_complete() {
                        None
                    } else {
                        Some(ListItems {
                            list: candidate_list.list.clone(),
                            items: candidate_list.incomplete_items(),
                        })
                    }
                })
                .collect(),
        }
    }

    fn find_general_items(store: &AppStore) -> GeneralItems {
        let mut political_group_items = store.get_political_group().incomplete_items();

        let authorised_agents = store.get_authorised_agents();
        if authorised_agents.is_empty() {
            political_group_items.push(IncompleteItem::NoAuthorizedAgent);
        }
        let authorized_agent_items = authorised_agents
            .into_iter()
            .map(|aa| PersonItems::new(aa))
            .collect();

        let list_submitter_items = store.get_list_submitter().incomplete_items();

        GeneralItems {
            general: political_group_items,
            authorized_agents: authorized_agent_items,
            list_submitter: list_submitter_items,
            substitute_submitters: vec![], // TODO
        }
    }

    pub fn models_downloadable(&self) -> bool {
        let candidate_iter = self.candidate_items.iter().flat_map(|ci| &ci.items);
        let list_iter = self.list_items.iter().flat_map(|ci| &ci.items);
        let general_iter = self.general_items.flatten();

        !candidate_iter
            .chain(list_iter)
            .chain(general_iter)
            .any(|ii| ii.severity() == Severity::Error)
    }
}

#[derive(Debug)]
pub struct GeneralItems {
    general: Vec<IncompleteItem>,
    authorized_agents: Vec<PersonItems<AuthorisedAgent>>,
    list_submitter: Vec<IncompleteItem>,
    substitute_submitters: Vec<PersonItems<ListSubmitter>>,
}

impl GeneralItems {
    pub fn flatten(&self) -> Vec<&IncompleteItem> {
        let mut result = Vec::new();

        result.extend(&self.general);
        result.extend(self.authorized_agents.iter().flat_map(|aa| &aa.items));
        result.extend(self.substitute_submitters.iter().flat_map(|ss| &ss.items));
        result.extend(&self.list_submitter);

        result
    }
}
#[derive(Debug)]
pub struct PersonItems<T> {
    pub person: T,
    pub items: Vec<IncompleteItem>,
}

impl<T: Completable> PersonItems<T> {
    fn new(person: T) -> Self {
        let items = person.incomplete_items();
        PersonItems { person, items }
    }
}

#[derive(Debug)]
pub struct ListItems {
    pub list: CandidateList,
    pub items: Vec<IncompleteItem>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum IncompleteItem {
    // candidate list
    NoCandidates,
    TooManyCandidates { actual: usize, max: usize },
    DuplicateDistricts { duplicates: Vec<ElectoralDistrict> },
    NoDistricts,

    // political group
    NoLegalName,
    NoDisplayName,
    NoPreviousElectionResults,
    NoAuthorizedAgent,

    // name related
    NoInitials(Severity),
    NoLastName(Severity),

    // address related
    NoStreetName(Severity),
    NoHouseNumber(Severity),
    NoPostalCode(Severity),
    NoLocality(Severity),
    NoCountry(Severity),
}

impl IncompleteItem {
    pub fn severity(&self) -> Severity {
        match &self {
            // candidate list
            IncompleteItem::NoCandidates => Severity::Error,
            IncompleteItem::TooManyCandidates { .. } => Severity::Warn,
            IncompleteItem::DuplicateDistricts { .. } => Severity::Error,
            IncompleteItem::NoDistricts => Severity::Error,

            // political group
            IncompleteItem::NoLegalName => Severity::Warn,
            IncompleteItem::NoDisplayName => Severity::Error,
            IncompleteItem::NoPreviousElectionResults => Severity::Info,
            IncompleteItem::NoAuthorizedAgent => Severity::Warn,

            // name related
            IncompleteItem::NoInitials(severity) => *severity,
            IncompleteItem::NoLastName(severity) => *severity,

            // address related
            IncompleteItem::NoStreetName(severity) => *severity,
            IncompleteItem::NoHouseNumber(severity) => *severity,
            IncompleteItem::NoPostalCode(severity) => *severity,
            IncompleteItem::NoLocality(severity) => *severity,
            IncompleteItem::NoCountry(severity) => *severity,
        }
    }
}

#[derive(PartialEq, Clone, Debug, Copy)]
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

    fn empty_general() -> GeneralItems {
        GeneralItems {
            general: Vec::new(),
            authorized_agents: Vec::new(),
            list_submitter: Vec::new(),
            substitute_submitters: Vec::new(),
        }
    }

    #[test]
    fn is_printable() {
        assert!(
            IncompleteItems {
                general_items: empty_general(),
                candidate_items: Vec::new(),
                list_items: Vec::new(),
            }
            .models_downloadable()
        );

        assert!(
            IncompleteItems {
                general_items: empty_general(),
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
                general_items: empty_general(),
                candidate_items: vec![PersonItems {
                    person: sample_person(PersonId::new()),
                    items: vec![IncompleteItem::NoCandidates]
                }],
                list_items: Vec::new(),
            }
            .models_downloadable()
        );
    }
}
