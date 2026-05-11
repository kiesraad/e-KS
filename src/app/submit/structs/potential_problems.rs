use crate::{
    AppStore, ElectoralDistrict,
    authorised_agents::AuthorisedAgent,
    candidate_lists::{CandidateList, CandidateListSummary},
    list_submitters::ListSubmitter,
    persons::Person,
};

/// Aggregation struct for everything that can be missing or incomplete for a list submission
#[derive(Debug)]
pub struct Problems {
    pub general: GeneralProblems,
    pub candidates: Vec<PersonProblems<Person>>,
    pub lists: Vec<ListProblems>,
}

impl Problems {
    pub fn find_all(store: &AppStore) -> Self {
        let candidate_lists = CandidateListSummary::list(store);

        Self {
            general: Self::find_general_problems(store),
            candidates: vec![], // TODO: complete in issue #605
            lists: candidate_lists
                .iter()
                .filter_map(|candidate_list| {
                    if candidate_list.is_all_good() {
                        None
                    } else {
                        Some(ListProblems {
                            list: candidate_list.list.clone(),
                            problems: candidate_list.get_problems(),
                        })
                    }
                })
                .collect(),
        }
    }

    fn find_general_problems(store: &AppStore) -> GeneralProblems {
        let mut general = store.get_political_group().get_problems();

        let authorised_agents = store.get_authorised_agents();
        if authorised_agents.is_empty() {
            general.push(PotentialProblems::NoAuthorizedAgent);
        }
        let authorised_agents = authorised_agents
            .into_iter()
            .map(PersonProblems::new)
            .collect();

        let list_submitter = store.get_list_submitter().get_problems();

        GeneralProblems {
            general,
            authorised_agents,
            list_submitter,
            substitute_submitters: vec![], // TODO
        }
    }

    pub fn models_downloadable(&self) -> bool {
        let candidate_iter = self.candidates.iter().flat_map(|ci| &ci.problems);
        let list_iter = self.lists.iter().flat_map(|ci| &ci.problems);
        let general_iter = self.general.flatten();

        !candidate_iter
            .chain(list_iter)
            .chain(general_iter)
            .any(|ii| ii.severity() == Severity::Error)
    }
}

#[derive(Debug)]
pub struct GeneralProblems {
    general: Vec<PotentialProblems>,
    authorised_agents: Vec<PersonProblems<AuthorisedAgent>>,
    list_submitter: Vec<PotentialProblems>,
    substitute_submitters: Vec<PersonProblems<ListSubmitter>>,
}

impl GeneralProblems {
    pub fn flatten(&self) -> Vec<&PotentialProblems> {
        let mut result = Vec::new();

        result.extend(&self.general);
        result.extend(self.authorised_agents.iter().flat_map(|aa| &aa.problems));
        result.extend(
            self.substitute_submitters
                .iter()
                .flat_map(|ss| &ss.problems),
        );
        result.extend(&self.list_submitter);

        result
    }
}
#[derive(Debug)]
pub struct PersonProblems<T> {
    pub person: T,
    pub problems: Vec<PotentialProblems>,
}

impl<T: Problematic> PersonProblems<T> {
    fn new(person: T) -> Self {
        let problems = person.get_problems();
        PersonProblems { person, problems }
    }
}

#[derive(Debug)]
pub struct ListProblems {
    pub list: CandidateList,
    pub problems: Vec<PotentialProblems>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum PotentialProblems {
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

impl PotentialProblems {
    pub fn severity(&self) -> Severity {
        match &self {
            // candidate list
            PotentialProblems::NoCandidates => Severity::Error,
            PotentialProblems::TooManyCandidates { .. } => Severity::Warn,
            PotentialProblems::DuplicateDistricts { .. } => Severity::Error,
            PotentialProblems::NoDistricts => Severity::Error,

            // political group
            PotentialProblems::NoLegalName => Severity::Warn,
            PotentialProblems::NoDisplayName => Severity::Error,
            PotentialProblems::NoPreviousElectionResults => Severity::Info,
            PotentialProblems::NoAuthorizedAgent => Severity::Warn,

            // name related
            PotentialProblems::NoInitials(severity) => *severity,
            PotentialProblems::NoLastName(severity) => *severity,

            // address related
            PotentialProblems::NoStreetName(severity) => *severity,
            PotentialProblems::NoHouseNumber(severity) => *severity,
            PotentialProblems::NoPostalCode(severity) => *severity,
            PotentialProblems::NoLocality(severity) => *severity,
            PotentialProblems::NoCountry(severity) => *severity,
        }
    }
}

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum Severity {
    Info,
    Warn,
    Error,
}

pub trait Problematic {
    /// returns all incomplete items of its own and of all children
    fn get_problems(&self) -> Vec<PotentialProblems>;

    fn is_all_good(&self) -> bool {
        self.get_problems().is_empty()
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

    fn empty_general() -> GeneralProblems {
        GeneralProblems {
            general: Vec::new(),
            authorised_agents: Vec::new(),
            list_submitter: Vec::new(),
            substitute_submitters: Vec::new(),
        }
    }

    #[test]
    fn is_printable() {
        assert!(
            Problems {
                general: empty_general(),
                candidates: Vec::new(),
                lists: Vec::new(),
            }
            .models_downloadable()
        );

        assert!(
            Problems {
                general: empty_general(),
                candidates: vec![],
                lists: vec![ListProblems {
                    list: sample_candidate_list(CandidateListId::new()),
                    problems: vec![PotentialProblems::TooManyCandidates {
                        actual: 12,
                        max: 12
                    }],
                }],
            }
            .models_downloadable()
        );

        assert!(
            !Problems {
                general: empty_general(),
                candidates: vec![PersonProblems {
                    person: sample_person(PersonId::new()),
                    problems: vec![PotentialProblems::NoCandidates]
                }],
                lists: Vec::new(),
            }
            .models_downloadable()
        );
    }
}
