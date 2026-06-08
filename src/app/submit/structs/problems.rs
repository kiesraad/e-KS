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
    submit::SubmitPath,
};

impl PotentialProblems {
    pub fn candidate_list_fix_path(&self, list: &CandidateList) -> String {
        let submit = SubmitPath {}.to_string();
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
            PotentialProblems::DuplicateDistricts => CandidateList::list_path().to_string(),
            _ => list.update_path_from(submit).to_string(),
        }
    }

    pub fn person_fix_path(&self, person: &Person) -> String {
        let submit = SubmitPath {}.to_string();
        match self {
            PotentialProblems::IncompleteAddress { .. } => person
                .update_address_path()
                .with_query_params(QueryParamState::redirect_to(submit))
                .to_string(),
            PotentialProblems::NoRepresentative | PotentialProblems::RepresentativeProblem(_) => {
                person
                    .update_representative_path()
                    .with_query_params(QueryParamState::redirect_to(submit))
                    .to_string()
            }
            _ => person
                .update_path()
                .with_query_params(QueryParamState::redirect_to(submit))
                .to_string(),
        }
    }

    pub fn general_fix_path(&self) -> String {
        let submit = SubmitPath {}.to_string();
        match self {
            PotentialProblems::NoAuthorisedAgent | PotentialProblems::NoLegalName => {
                NameAuthorisation::list_path().to_string()
            }
            PotentialProblems::NoListSubmitter => ListSubmitter::update_path()
                .with_query_params(QueryParamState::redirect_to(submit))
                .to_string(),
            PotentialProblems::NoSubstituteSubmitter => ListSubmitter::view_path().to_string(),
            PotentialProblems::NoCandidateList => CandidateList::list_path().to_string(),
            PotentialProblems::NoDesignationType => ListDesignation::update_path().to_string(),
            PotentialProblems::TooFewAuthorizedNames { .. }
            | PotentialProblems::TooManyAuthorizedNames { .. } => {
                NameAuthorisation::list_path().to_string()
            }
            _ => PoliticalGroup::update_path().to_string(),
        }
    }
}

/// Aggregation struct for everything that can be missing or incomplete for a list submission
#[derive(Debug)]
pub struct Problems {
    pub general: GeneralProblems,
    pub candidates: Vec<PersonProblems>,
    pub lists: Vec<ListProblems>,
}

impl Problems {
    pub fn find_all(store: &AppStore) -> Self {
        let candidate_lists = CandidateListSummary::list(store);
        Self {
            general: Self::find_general_problems(store),
            candidates: Self::find_candidate_problems(store, &candidate_lists),
            lists: Self::find_list_problems(store, &candidate_lists),
        }
    }

    fn find_general_problems(store: &AppStore) -> GeneralProblems {
        let political_group = store.get_political_group();
        let mut general = political_group.get_problems(());

        if store.get_candidate_list_count() == 0 {
            general.push(PotentialProblems::NoCandidateList);
        }

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
            Some(list_designation) => {
                general.extend(Self::find_name_authorisation_size_problems(
                    list_designation,
                    name_authorisations.len(),
                ));
                Self::find_name_authorisation_problems(name_authorisations)
            }
        };

        let list_submitter = store.get_list_submitter();
        if list_submitter.is_empty() {
            general.push(PotentialProblems::NoListSubmitter);
        }

        let list_submitter_problems = list_submitter.get_problems(());
        let list_submitter = if !list_submitter_problems.is_empty() {
            Some(EntityProblems {
                entity: list_submitter,
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
            .map(EntityProblems::new)
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
    ) -> Vec<EntityProblems<NameAuthorisation>> {
        name_authorisations
            .into_iter()
            .map(EntityProblems::new)
            .filter(|pp| !pp.problems.is_empty())
            .collect()
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

    fn find_candidate_problems(
        store: &AppStore,
        candidate_lists: &[CandidateListSummary],
    ) -> Vec<PersonProblems> {
        let mut seen = std::collections::HashSet::new();
        candidate_lists
            .iter()
            .flat_map(|list| list.list.candidates.iter())
            .filter(|id| seen.insert(*id))
            .filter_map(|id| store.get_person(*id).ok())
            .filter_map(|person| {
                // TODO: Remove the below line once `Problematic` gets an overhaul.
                // get_problems is only for the 'candidate_too_young' check. Other checks are done in the get_problems of person.
                let mut problems = person.personal_data.get_problems(store);
                problems.extend(person.get_problems(()));
                (!problems.is_empty()).then_some(PersonProblems {
                    entity: person,
                    problems,
                })
            })
            .collect()
    }

    fn find_list_problems(
        store: &AppStore,
        candidate_lists: &[CandidateListSummary],
    ) -> Vec<ListProblems> {
        let mut seen_duplicate_district = false;
        candidate_lists
            .iter()
            .filter_map(|candidate_list| {
                let problems = candidate_list.get_problems(store);
                (!problems.is_empty()).then(|| ListProblems {
                    entity: candidate_list.list.clone(),
                    problems,
                })
            })
            // make sure only one DuplicateDistrict Problem remains
            .map(|list_problem| {
                let ListProblems { entity, problems } = list_problem;
                let problems = problems
                    .into_iter()
                    .filter(|problem| {
                        if *problem == PotentialProblems::DuplicateDistricts {
                            if seen_duplicate_district {
                                return false;
                            }
                            seen_duplicate_district = true;
                        }
                        true
                    })
                    .collect();
                ListProblems { entity, problems }
            })
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
}

#[derive(Debug)]
pub struct GeneralProblems {
    pub general: Vec<PotentialProblems>,
    pub name_authorisations: Vec<EntityProblems<NameAuthorisation>>,
    pub list_submitter: Option<EntityProblems<ListSubmitter>>,
    pub substitute_submitters: Vec<EntityProblems<ListSubmitter>>,
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
pub struct EntityProblems<T> {
    pub entity: T,
    pub problems: Vec<PotentialProblems>,
}

impl<T: Problematic<()>> EntityProblems<T> {
    fn new(entity: T) -> Self {
        let problems = entity.get_problems(());
        EntityProblems { entity, problems }
    }
}
pub type ListProblems = EntityProblems<CandidateList>;
pub type PersonProblems = EntityProblems<Person>;

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

    async fn add_candidate_list(store: &AppStore) -> Result<(), AppError> {
        sample_candidate_list(CandidateListId::new())
            .create(store)
            .await?;
        Ok(())
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
                    entity: sample_candidate_list(CandidateListId::new()),
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
                    entity: sample_person(PersonId::new()),
                    problems: vec![PotentialProblems::NoCandidates]
                }],
                lists: Vec::new(),
            }
            .models_downloadable()
        );
    }

    #[tokio::test]
    async fn no_candidate_list_added() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        // make sure no other general errors occur
        add_submitters(&store).await?;
        add_name_authorisations(&store, 1).await?;

        let problems = Problems::find_general_problems(&store);

        assert_eq!(problems.general.len(), 1);
        assert_eq!(problems.general[0], PotentialProblems::NoCandidateList);

        Ok(())
    }

    #[tokio::test]
    async fn standalone_no_name_authorisations() -> Result<(), AppError> {
        let store = AppStore::new_for_test();
        // make sure no other general errors occur
        add_submitters(&store).await?;
        add_candidate_list(&store).await?;

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
        add_candidate_list(&store).await?;

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
        add_candidate_list(&store).await?;

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
        add_candidate_list(&store).await?;

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
                .flat_map(|list_problems| &list_problems.problems)
                .filter(|problem| **problem == PotentialProblems::DuplicateDistricts)
                .count(),
            1
        );

        Ok(())
    }
}
