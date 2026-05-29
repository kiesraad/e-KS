use axum_extra::routing::TypedPath as _;

use crate::{
    AppStore, QueryParamState,
    candidate_lists::{CandidateList, CandidateListSummary},
    common::{PotentialProblems, Problematic, Severity},
    list_designation::ListDesignation,
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
            PotentialProblems::TooFewAuthorizedNames { .. }
            | PotentialProblems::TooManyAuthorizedNames { .. }
            | PotentialProblems::NoDesignationType => NameAuthorisation::list_path().to_string(),
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

        let mut seen_duplicate_district = false;
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
                        let problems = person.get_problems();
                        (!problems.is_empty()).then(|| PersonProblems { person, problems })
                    })
                    .collect()
            },
            lists: candidate_lists
                .iter()
                .filter_map(|candidate_list| {
                    let mut problems = candidate_list.get_problems();
                    problems.extend(candidate_list.get_deviation_problems(store));
                    (!problems.is_empty()).then(|| ListProblems {
                        list: candidate_list.list.clone(),
                        problems,
                    })
                })
                // make sure only one DuplicateDistrict Problem remains
                .map(|list_problem| {
                    let problems = list_problem.problems.iter().cloned().fold(
                        Vec::new(),
                        |mut problems, problem| {
                            if problem != PotentialProblems::DuplicateDistricts
                                || !seen_duplicate_district
                            {
                                if problem == PotentialProblems::DuplicateDistricts {
                                    seen_duplicate_district = true;
                                }
                                problems.push(problem);
                            }
                            problems
                        },
                    );
                    ListProblems {
                        list: list_problem.list,
                        problems,
                    }
                })
                .collect(),
        }
    }

    fn find_general_problems(store: &AppStore) -> GeneralProblems {
        let political_group = store.get_political_group();
        let mut general = political_group.get_problems();

        let name_authorisations = store.get_name_authorisations();
        let name_authorisations = match political_group.list_designation {
            None => {
                general.push(PotentialProblems::NoDesignationType);
                // assume standalone list as default
                general.extend(Self::find_name_authorisation_size_problems(
                    ListDesignation::Standalone,
                    name_authorisations.len(),
                ));
                Self::find_name_authorisation_problems(name_authorisations)
            }
            Some(ListDesignation::Blank) => Vec::new(),
            Some(ListDesignation::Standalone) | Some(ListDesignation::Combined) => {
                general.extend(Self::find_name_authorisation_size_problems(
                    political_group.list_designation.unwrap(),
                    name_authorisations.len(),
                ));
                Self::find_name_authorisation_problems(name_authorisations)
            }
        };

        let list_submitter = store.get_list_submitter();
        if list_submitter.is_empty() {
            general.push(PotentialProblems::NoListSubmitter);
        }

        let list_submitter_problems = list_submitter.get_problems();
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

    fn find_name_authorisation_problems(
        name_authorisations: Vec<NameAuthorisation>,
    ) -> Vec<PersonProblems<NameAuthorisation>> {
        name_authorisations
            .into_iter()
            .map(PersonProblems::new)
            .filter(|pp| !pp.problems.is_empty())
            .collect()
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

    fn find_name_authorisation_size_problems(
        list_designation: ListDesignation,
        authorised_names_count: usize,
    ) -> Vec<PotentialProblems> {
        match list_designation {
            ListDesignation::Standalone if authorised_names_count > 1 => {
                vec![PotentialProblems::TooManyAuthorizedNames {
                    actual: authorised_names_count,
                    max: 1,
                }]
            }
            ListDesignation::Standalone if authorised_names_count < 1 => {
                vec![PotentialProblems::TooFewAuthorizedNames {
                    actual: authorised_names_count,
                    min: 1,
                }]
            }
            ListDesignation::Combined if authorised_names_count < 2 => {
                vec![PotentialProblems::TooFewAuthorizedNames {
                    actual: authorised_names_count,
                    min: 2,
                }]
            }
            _ => Vec::new(),
        }
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

#[cfg(test)]
mod tests {
    use crate::{
        AppError, ElectoralDistrict,
        candidate_lists::CandidateListId,
        list_submitters::ListSubmitterId,
        name_authorisations::NameAuthorisationId,
        persons::PersonId,
        test_utils::{
            sample_candidate_list, sample_list_submitter, sample_name_authorisation, sample_person,
        },
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

    async fn add_submitters(store: &AppStore) -> Result<(), AppError> {
        sample_list_submitter(ListSubmitterId::new())
            .update(store)
            .await?;
        sample_list_submitter(ListSubmitterId::new())
            .create_substitute(store)
            .await?;
        Ok(())
    }

    async fn add_name_authorisations(store: &AppStore, count: usize) -> Result<(), AppError> {
        for _ in 0..count {
            sample_name_authorisation(NameAuthorisationId::new())
                .create(store)
                .await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn standalone_no_name_authorisations() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        // make sure no other general errors occur
        add_submitters(&store).await?;

        // make political group standalone
        let mut group = store.get_political_group();
        group.list_designation = Some(ListDesignation::Standalone);
        group.update(&store).await?;

        let problems = Problems::find_general_problems(&store);

        assert_eq!(problems.general.len(), 1);
        assert_eq!(
            problems.general[0],
            PotentialProblems::TooFewAuthorizedNames { actual: 0, min: 1 }
        );

        Ok(())
    }

    #[tokio::test]
    async fn standalone_two_name_authorisations() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        // make sure no other general errors occur
        add_submitters(&store).await?;

        // make political group standalone
        let mut group = store.get_political_group();
        group.list_designation = Some(ListDesignation::Standalone);
        group.update(&store).await?;

        add_name_authorisations(&store, 2).await?;

        let problems = Problems::find_general_problems(&store);

        assert_eq!(problems.general.len(), 1);
        assert_eq!(
            problems.general[0],
            PotentialProblems::TooManyAuthorizedNames { actual: 2, max: 1 }
        );

        Ok(())
    }

    #[tokio::test]
    async fn combined_one_name_authorisations() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        // make sure no other general errors occur
        add_submitters(&store).await?;

        // make political group standalone
        let mut group = store.get_political_group();
        group.list_designation = Some(ListDesignation::Combined);
        group.update(&store).await?;

        add_name_authorisations(&store, 1).await?;

        let problems = Problems::find_general_problems(&store);

        assert_eq!(problems.general.len(), 1);
        assert_eq!(
            problems.general[0],
            PotentialProblems::TooFewAuthorizedNames { actual: 1, min: 2 }
        );

        Ok(())
    }

    #[tokio::test]
    async fn blank_ten_name_authorisations() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        // make sure no other general errors occur
        add_submitters(&store).await?;

        // make political group standalone
        let mut group = store.get_political_group();
        group.list_designation = Some(ListDesignation::Blank);
        group.update(&store).await?;

        add_name_authorisations(&store, 10).await?;

        let problems = Problems::find_general_problems(&store);

        assert!(problems.general.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn max_one_duplicate_district_problem() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        for _ in 0..10 {
            let mut list1 = sample_candidate_list(CandidateListId::new());
            list1.electoral_districts = vec![ElectoralDistrict::UT, ElectoralDistrict::GR];
            list1.create(&store).await?;
        }

        let problems = Problems::find_all(&store);
        assert_eq!(
            problems
                .lists
                .iter()
                .fold(Vec::new(), |mut problems, list_problems| {
                    problems.extend(list_problems.problems.clone());
                    problems
                })
                .iter()
                .filter(|problem| problem == &&PotentialProblems::DuplicateDistricts)
                .count(),
            1
        );

        Ok(())
    }
}
