use axum_extra::routing::TypedPath as _;

use crate::{
    AppStore, QueryParamState,
    candidate_lists::{CandidateList, CandidateListSummary},
    common::{PotentialProblems, Problematic, Severity},
    list_submitters::ListSubmitter,
    name_authorisations::NameAuthorisation,
    persons::Person,
    political_groups::PoliticalGroup,
};

impl PotentialProblems {
    pub fn candidate_list_fix_path(&self, list: &CandidateList) -> String {
        match self {
            PotentialProblems::NoCandidates => list.view_path().to_string(),
            PotentialProblems::TooManyCandidates { actual, max } => {
                let overflow = actual.saturating_sub(*max);
                list.view_path()
                    .with_query_params(QueryParamState::highlight_last(overflow))
                    .to_string()
            }
            PotentialProblems::FewCandidatesWithFirstName { .. }
            | PotentialProblems::FewCandidatesWithoutFirstName { .. }
            | PotentialProblems::FewCandidatesWithGender { .. }
            | PotentialProblems::FewCandidatesWithoutGender { .. } => list.view_path().to_string(),
            _ => list.update_path().to_string(),
        }
    }

    pub fn person_fix_path(&self, person: &Person) -> String {
        match self {
            PotentialProblems::IncompleteAddress { .. } => person.update_address_path().to_string(),
            PotentialProblems::NoRepresentative | PotentialProblems::RepresentativeProblem(_) => {
                person.update_representative_path().to_string()
            }
            _ => person.update_path().to_string(),
        }
    }

    pub fn general_fix_path(&self) -> String {
        match self {
            PotentialProblems::NoAuthorisedAgent | PotentialProblems::NoLegalName => {
                NameAuthorisation::list_path().to_string()
            }
            PotentialProblems::NoListSubmitter => ListSubmitter::update_path().to_string(),
            PotentialProblems::NoSubstituteSubmitter => ListSubmitter::view_path().to_string(),
            _ => PoliticalGroup::update_path().to_string(),
        }
    }
}

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
            candidates: {
                let mut seen = std::collections::HashSet::new();
                candidate_lists
                    .iter()
                    .flat_map(|list| list.list.candidates.iter())
                    .filter(|id| seen.insert(*id))
                    .filter_map(|id| store.get_person(*id).ok())
                    .filter_map(|person| {
                        let mut problems = person.get_problems(());
                        problems.extend(person.get_problems(store));
                        (!problems.is_empty()).then(|| PersonProblems { person, problems })
                    })
                    .collect()
            },
            lists: candidate_lists
                .iter()
                .filter_map(|candidate_list| {
                    let mut problems = candidate_list.get_problems(());
                    problems.extend(candidate_list.get_deviation_problems(store));
                    (!problems.is_empty()).then(|| ListProblems {
                        list: candidate_list.list.clone(),
                        problems,
                    })
                })
                .collect(),
        }
    }

    fn find_general_problems(store: &AppStore) -> GeneralProblems {
        let mut general = store.get_political_group().get_problems(());

        let name_authorisations = store.get_name_authorisations();
        if name_authorisations.is_empty() {
            // TODO: consider list designation type
            general.push(PotentialProblems::NoLegalName);
        }
        let name_authorisations = name_authorisations
            .into_iter()
            .map(PersonProblems::new)
            .filter(|pp| !pp.problems.is_empty())
            .collect();

        let list_submitter = store.get_list_submitter();
        if list_submitter.is_empty() {
            general.push(PotentialProblems::NoListSubmitter);
        }

        let list_submitter_problems = list_submitter.get_problems(());
        let list_submitter = if !list_submitter_problems.is_empty() {
            Some(PersonProblems {
                person: list_submitter,
                problems: list_submitter_problems,
            })
        } else {
            None
        };

        let substitute_submitters = store.get_substitute_submitters();
        if substitute_submitters.is_empty() {
            general.push(PotentialProblems::NoSubstituteSubmitter);
        }
        let substitute_submitters = substitute_submitters
            .into_iter()
            .map(PersonProblems::new)
            .filter(|pp| !pp.problems.is_empty())
            .collect();

        GeneralProblems {
            general,
            name_authorisations,
            list_submitter,
            substitute_submitters,
        }
    }

    pub fn models_downloadable(&self) -> bool {
        let candidate_iter = self.candidates.iter().flat_map(|ci| &ci.problems);
        // Lists without candidates cannot produce exports, so their errors don't block downloads
        let list_iter = self
            .lists
            .iter()
            .filter(|li| !li.problems.contains(&PotentialProblems::NoCandidates))
            .flat_map(|ci| &ci.problems);
        let general_iter = self.general.flatten();

        !candidate_iter
            .chain(list_iter)
            .chain(general_iter)
            .any(|ii| ii.severity() == Severity::Error)
    }
}

#[derive(Debug)]
pub struct GeneralProblems {
    pub general: Vec<PotentialProblems>,
    pub name_authorisations: Vec<PersonProblems<NameAuthorisation>>,
    pub list_submitter: Option<PersonProblems<ListSubmitter>>,
    pub substitute_submitters: Vec<PersonProblems<ListSubmitter>>,
}

impl GeneralProblems {
    pub fn flatten(&self) -> Vec<&PotentialProblems> {
        let mut result = Vec::new();

        result.extend(&self.general);
        result.extend(self.name_authorisations.iter().flat_map(|na| &na.problems));
        result.extend(
            self.substitute_submitters
                .iter()
                .flat_map(|ss| &ss.problems),
        );
        if let Some(submitter_problems) = &self.list_submitter {
            result.extend(&submitter_problems.problems);
        }

        result
    }
}

#[derive(Debug)]
pub struct PersonProblems<T> {
    pub person: T,
    pub problems: Vec<PotentialProblems>,
}

impl<T: Problematic<()>> PersonProblems<T> {
    fn new(person: T) -> Self {
        let problems = person.get_problems(());
        PersonProblems { person, problems }
    }
}

#[derive(Debug)]
pub struct ListProblems {
    pub list: CandidateList,
    pub problems: Vec<PotentialProblems>,
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
            name_authorisations: Vec::new(),
            list_submitter: None,
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
