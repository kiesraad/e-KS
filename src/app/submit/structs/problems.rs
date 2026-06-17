use axum_extra::routing::TypedPath as _;

use crate::{
    AppError, AppStore, Locale, QueryParamState,
    candidate_lists::{CandidateList, CandidateListSummary, FullCandidateList},
    common::{HasSeverity, IndexPath, InfoProblems, PotentialProblems, Problematic, Severity},
    list_designation::ListDesignation,
    list_submitters::ListSubmitter,
    name_authorisations::NameAuthorisation,
    persons::Person,
    political_groups::PoliticalGroup,
    submit::SubmitPath,
};

impl PotentialProblems {
    pub fn candidate_list_fix_path(&self, list: &CandidateList) -> String {
        match self {
            PotentialProblems::NoCandidates => list.view_path().to_string(),
            PotentialProblems::TooManyCandidates { count } => list
                .view_path()
                .with_query_params(QueryParamState::highlight_last(*count))
                .to_string(),
            PotentialProblems::DuplicateDistricts => CandidateList::list_path().to_string(),
            _ => list.view_path().to_string(),
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
            PotentialProblems::NoCandidateList => CandidateList::list_path().to_string(),

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
pub struct AllProblems {
    pub general: GeneralProblems,
    pub candidates: Vec<PersonProblems>,
    pub lists: Vec<ListProblems>,
    pub info_problems: Vec<EntityInfoProblems>,
}

impl AllProblems {
    pub fn find_all(store: &AppStore) -> Result<Self, AppError> {
        let candidate_lists = CandidateListSummary::list(store);
        let (general, general_info) = Self::find_general_problems(store);
        let (candidates, candidates_info) = Self::find_candidate_problems(store, &candidate_lists);
        let (lists, lists_info) = Self::find_list_problems(&candidate_lists, store)?;
        Ok(Self {
            general,
            candidates,
            lists,
            info_problems: [general_info, candidates_info, lists_info]
                .into_iter()
                .flatten()
                .collect(),
        })
    }

    pub fn find_general_problems(store: &AppStore) -> (GeneralProblems, Vec<EntityInfoProblems>) {
        let mut info_problems = Vec::new();
        let mut general = Vec::new();

        let political_group = store.get_political_group();

        let pg_problems = political_group.get_problems(());
        info_problems.extend(
            pg_problems
                .info_problems
                .into_iter()
                .map(EntityInfoProblems::AnyProblem)
                .collect::<Vec<_>>(),
        );
        general.extend(pg_problems.potential_problems);

        if store.get_candidate_list_count() == 0 {
            general.push(PotentialProblems::NoCandidateList);
        }

        let name_authorisations = store.get_name_authorisations();
        let name_authorisations = match political_group.list_designation {
            Some(ListDesignation::Blank) => Vec::new(),
            list_designation => {
                general.extend(Self::find_name_authorisation_size_problems(
                    list_designation.unwrap_or(ListDesignation::Standalone),
                    name_authorisations.len(),
                ));
                let (problems, infos) = Self::find_name_authorisation_problems(name_authorisations);
                info_problems.extend(infos);
                problems
            }
        };

        let list_submitter = store.get_list_submitter();
        if list_submitter.is_empty() {
            general.push(PotentialProblems::NoListSubmitter);
        }

        let all_list_submitter_problems = list_submitter.get_problems(());
        let list_submitter_problems = if !all_list_submitter_problems.potential_problems.is_empty()
        {
            Some(EntityProblems {
                entity: list_submitter.clone(),
                problems: all_list_submitter_problems.potential_problems,
            })
        } else {
            None
        };
        info_problems.extend(
            all_list_submitter_problems
                .info_problems
                .into_iter()
                .map(|problem| EntityInfoProblems::Submitter {
                    submitter: list_submitter.clone(),
                    problem,
                })
                .collect::<Vec<_>>(),
        );

        let submitters = store.get_substitute_submitters();
        if submitters.is_empty() {
            info_problems.push(EntityInfoProblems::AnyProblem(
                InfoProblems::NoSubstituteSubmitter,
            ));
        }
        let mut substitute_submitters = Vec::new();
        for ss in submitters {
            let (ss_problems, infos) = EntityProblems::new(ss.clone());
            if !ss_problems.problems.is_empty() {
                substitute_submitters.push(ss_problems)
            }
            info_problems.extend(
                infos
                    .into_iter()
                    .map(|problem| EntityInfoProblems::SubstituteSubmitter {
                        submitter: ss.clone(),
                        problem,
                    })
                    .collect::<Vec<_>>(),
            );
        }

        (
            GeneralProblems {
                general,
                name_authorisations,
                list_submitter: list_submitter_problems,
                substitute_submitters,
            },
            info_problems,
        )
    }

    fn find_name_authorisation_problems(
        name_authorisations: Vec<NameAuthorisation>,
    ) -> (
        Vec<EntityProblems<NameAuthorisation>>,
        Vec<EntityInfoProblems>,
    ) {
        let mut problems = Vec::new();
        let mut info_problems = Vec::new();
        for name_authorisation in name_authorisations {
            let (na_problems, na_info_problems) = EntityProblems::new(name_authorisation.clone());
            if !na_problems.problems.is_empty() {
                problems.push(na_problems)
            }
            info_problems.extend(
                na_info_problems
                    .into_iter()
                    .map(|problem| EntityInfoProblems::NameAuthorisation {
                        name_authorisation: name_authorisation.clone(),
                        problem,
                    })
                    .collect::<Vec<_>>(),
            );
        }
        (problems, info_problems)
    }

    fn find_name_authorisation_size_problems(
        list_designation: ListDesignation,
        authorised_names_count: usize,
    ) -> Vec<PotentialProblems> {
        match list_designation {
            ListDesignation::Standalone if authorised_names_count > 1 => {
                vec![PotentialProblems::TooManyAuthorizedNames {
                    count: authorised_names_count - 1,
                }]
            }
            ListDesignation::Standalone if authorised_names_count < 1 => {
                vec![PotentialProblems::TooFewAuthorizedNames { count: 1 }]
            }
            ListDesignation::Combined if authorised_names_count < 2 => {
                vec![PotentialProblems::TooFewAuthorizedNames {
                    count: 2 - authorised_names_count,
                }]
            }
            _ => Vec::new(),
        }
    }

    pub fn find_candidate_problems(
        store: &AppStore,
        candidate_lists: &[CandidateListSummary],
    ) -> (Vec<PersonProblems>, Vec<EntityInfoProblems>) {
        let mut info_problems = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let problems = candidate_lists
            .iter()
            .flat_map(|list| list.list.candidates.iter())
            .filter(|id| seen.insert(*id))
            .filter_map(|id| store.get_person(*id).ok())
            .filter_map(|person| {
                let problems = person.get_problems(store.election);
                info_problems.extend(
                    problems
                        .info_problems
                        .into_iter()
                        .map(|problem| EntityInfoProblems::Person {
                            person: Box::new(person.clone()),
                            problem,
                        })
                        .collect::<Vec<_>>(),
                );
                (!problems.potential_problems.is_empty()).then_some(PersonProblems {
                    entity: person,
                    problems: problems.potential_problems,
                })
            })
            .collect();
        (problems, info_problems)
    }

    pub fn find_list_problems(
        candidate_lists: &[CandidateListSummary],
        store: &AppStore,
    ) -> Result<(Vec<ListProblems>, Vec<EntityInfoProblems>), AppError> {
        let mut list_problems = Vec::new();
        let mut info_problems = Vec::new();
        let mut seen_duplicate_district = false;
        for candidate_list in candidate_lists {
            let mut problems =
                candidate_list.get_problems(FullCandidateList::get(store, candidate_list.list.id)?);
            if problems
                .potential_problems
                .contains(&PotentialProblems::DuplicateDistricts)
            {
                if seen_duplicate_district {
                    problems
                        .potential_problems
                        .retain(|problem| problem != &PotentialProblems::DuplicateDistricts)
                }
                seen_duplicate_district = true;
            }
            if !problems.potential_problems.is_empty() {
                list_problems.push(ListProblems {
                    entity: candidate_list.list.clone(),
                    problems: problems.potential_problems,
                })
            }
            info_problems.extend(
                problems
                    .info_problems
                    .into_iter()
                    .map(|problem| EntityInfoProblems::List {
                        list: candidate_list.list.clone(),
                        problem,
                    })
                    .collect::<Vec<_>>(),
            );
        }
        Ok((list_problems, info_problems))
    }

    fn flatten_problems(&self) -> impl Iterator<Item = &PotentialProblems> {
        let candidate_iter = self.candidates.iter().flat_map(|ci| &ci.problems);
        let list_iter = self.lists.iter().flat_map(|ci| &ci.problems);
        let general_iter = self.general.flatten();

        candidate_iter.chain(list_iter).chain(general_iter)
    }

    pub fn models_downloadable(&self) -> bool {
        !self
            .flatten_problems()
            .any(|ii| ii.severity() == Severity::Error)
    }
}

impl HasSeverity for AllProblems {
    fn highest_severity(&self) -> Option<Severity> {
        self.flatten_problems()
            .map(|p| p.severity())
            .max()
            .or_else(|| (!self.info_problems.is_empty()).then_some(Severity::Info))
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
    fn new(entity: T) -> (Self, Vec<InfoProblems>) {
        let problems = entity.get_problems(());
        (
            EntityProblems {
                entity,
                problems: problems.potential_problems,
            },
            problems.info_problems,
        )
    }
}
pub type ListProblems = EntityProblems<CandidateList>;
pub type PersonProblems = EntityProblems<Person>;

#[derive(Debug)]
pub enum EntityInfoProblems {
    AnyProblem(InfoProblems),
    List {
        list: CandidateList,
        problem: InfoProblems,
    },
    Submitter {
        submitter: ListSubmitter,
        problem: InfoProblems,
    },
    SubstituteSubmitter {
        submitter: ListSubmitter,
        problem: InfoProblems,
    },
    Person {
        person: Box<Person>,
        problem: InfoProblems,
    },
    NameAuthorisation {
        name_authorisation: NameAuthorisation,
        problem: InfoProblems,
    },
}

impl EntityInfoProblems {
    pub fn fix_path(&self) -> String {
        let submit = SubmitPath {}.to_string();
        match self {
            EntityInfoProblems::AnyProblem(InfoProblems::NoSubstituteSubmitter) => {
                ListSubmitter::view_path().to_string()
            }
            EntityInfoProblems::AnyProblem(InfoProblems::NoListDesignation) => {
                ListDesignation::update_path().to_string()
            }
            EntityInfoProblems::AnyProblem(InfoProblems::NoPreviousElectionResults) => {
                PoliticalGroup::update_path().to_string()
            }
            EntityInfoProblems::AnyProblem(..) => IndexPath.to_string(),

            EntityInfoProblems::List { list, .. } => list.view_path().to_string(),
            EntityInfoProblems::SubstituteSubmitter { submitter, .. } => submitter
                .substitute_update_path()
                .with_query_params(QueryParamState::redirect_to(submit))
                .to_string(),
            EntityInfoProblems::Submitter { .. } => ListSubmitter::update_path()
                .with_query_params(QueryParamState::redirect_to(submit))
                .to_string(),
            EntityInfoProblems::Person { person, .. } => person
                .update_path()
                .with_query_params(QueryParamState::redirect_to(submit))
                .to_string(),
            EntityInfoProblems::NameAuthorisation {
                name_authorisation, ..
            } => name_authorisation
                .update_path()
                .with_query_params(QueryParamState::redirect_to(submit))
                .to_string(),
        }
    }
    pub fn translate(&self, locale: &Locale) -> String {
        match self {
            EntityInfoProblems::AnyProblem(problem) => problem.translate(locale),
            EntityInfoProblems::List { problem, .. } => problem.translate(locale),
            EntityInfoProblems::Submitter { problem, .. } => problem.translate(locale),
            EntityInfoProblems::SubstituteSubmitter { problem, .. } => problem.translate(locale),
            EntityInfoProblems::Person { problem, .. } => problem.translate(locale),
            EntityInfoProblems::NameAuthorisation { problem, .. } => problem.translate(locale),
        }
    }

    pub fn severity(&self) -> Severity {
        match self {
            EntityInfoProblems::AnyProblem(problem) => problem.severity(),
            EntityInfoProblems::List { problem, .. } => problem.severity(),
            EntityInfoProblems::Submitter { problem, .. } => problem.severity(),
            EntityInfoProblems::SubstituteSubmitter { problem, .. } => problem.severity(),
            EntityInfoProblems::Person { problem, .. } => problem.severity(),
            EntityInfoProblems::NameAuthorisation { problem, .. } => problem.severity(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        AppError, ElectoralDistrict,
        candidate_lists::CandidateListId,
        common::HasSeverity,
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
            AllProblems {
                general: empty_general(),
                candidates: Vec::new(),
                lists: Vec::new(),
                info_problems: Vec::new()
            }
            .models_downloadable()
        );

        assert!(
            AllProblems {
                general: empty_general(),
                candidates: vec![],
                lists: vec![ListProblems {
                    entity: sample_candidate_list(CandidateListId::new()),
                    problems: vec![PotentialProblems::TooManyCandidates { count: 1 }],
                }],
                info_problems: Vec::new()
            }
            .models_downloadable()
        );

        assert!(
            !AllProblems {
                general: empty_general(),
                candidates: vec![PersonProblems {
                    entity: sample_person(PersonId::new()),
                    problems: vec![PotentialProblems::NoCandidates]
                }],
                lists: Vec::new(),
                info_problems: Vec::new()
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

        let (problems, _) = AllProblems::find_general_problems(&store);

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

        let (problems, _) = AllProblems::find_general_problems(&store);

        assert_eq!(problems.general.len(), 1);
        assert_eq!(
            problems.general[0],
            PotentialProblems::TooFewAuthorizedNames { count: 1 }
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

        let (problems, _) = AllProblems::find_general_problems(&store);

        assert_eq!(problems.general.len(), 1);
        assert_eq!(
            problems.general[0],
            PotentialProblems::TooManyAuthorizedNames { count: 1 }
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

        let (problems, _) = AllProblems::find_general_problems(&store);

        assert_eq!(problems.general.len(), 1);
        assert_eq!(
            problems.general[0],
            PotentialProblems::TooFewAuthorizedNames { count: 1 }
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

        let (problems, _) = AllProblems::find_general_problems(&store);

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

        let problems = AllProblems::find_all(&store)?;
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

    #[test]
    fn highest_severity_none() {
        let problems = AllProblems {
            general: empty_general(),
            candidates: Vec::new(),
            lists: Vec::new(),
            info_problems: Vec::new(),
        };
        assert_eq!(problems.highest_severity(), None);
    }

    #[test]
    fn highest_severity_info() {
        let problems = AllProblems {
            general: empty_general(),
            candidates: Vec::new(),
            lists: Vec::new(),
            info_problems: vec![EntityInfoProblems::AnyProblem(
                InfoProblems::NoSubstituteSubmitter,
            )],
        };
        assert_eq!(problems.highest_severity(), Some(Severity::Info));
    }

    #[test]
    fn highest_severity_error() {
        let problems = AllProblems {
            general: empty_general(),
            candidates: vec![PersonProblems {
                entity: sample_person(PersonId::new()),
                problems: vec![PotentialProblems::NoCandidates], // error
            }],
            lists: vec![ListProblems {
                entity: sample_candidate_list(CandidateListId::new()),
                problems: vec![PotentialProblems::TooManyCandidates { count: 1 }], // warning
            }],
            info_problems: vec![EntityInfoProblems::AnyProblem(
                InfoProblems::NoSubstituteSubmitter,
            )],
        };
        assert_eq!(problems.highest_severity(), Some(Severity::Error));
    }

    #[test]
    fn highest_severity_warn() {
        let problems = AllProblems {
            general: empty_general(),
            candidates: Vec::new(),
            lists: vec![ListProblems {
                entity: sample_candidate_list(CandidateListId::new()),
                problems: vec![PotentialProblems::TooManyCandidates { count: 1 }], // warning
            }],
            info_problems: vec![EntityInfoProblems::AnyProblem(
                InfoProblems::NoSubstituteSubmitter,
            )],
        };
        assert_eq!(problems.highest_severity(), Some(Severity::Warn));
    }
}
