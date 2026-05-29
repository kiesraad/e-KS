use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    AppStore, ElectoralDistrict, OptionAsStrExt,
    candidate_lists::CandidateList,
    common::{PotentialProblems, Problematic},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CandidateListSummary {
    pub list: CandidateList,
    pub max_count: usize,
    pub duplicate_districts: Vec<ElectoralDistrict>,
}

impl Problematic for CandidateListSummary {
    fn get_problems(&self) -> Vec<PotentialProblems> {
        let mut items = vec![];

        if self.candidate_count() == 0 {
            items.push(PotentialProblems::NoCandidates);
        } else if self.candidate_count() > self.max_count {
            items.push(PotentialProblems::TooManyCandidates {
                actual: self.candidate_count(),
                max: self.max_count,
            });
        }

        if !self.duplicate_districts.is_empty() {
            items.push(PotentialProblems::DuplicateDistricts);
        }

        if self.list.electoral_districts.is_empty() {
            items.push(PotentialProblems::NoDistricts)
        }

        items
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Deviant {
    FewWith(usize),
    FewWithout(usize),
}

const MAX_DEVIATION_PERCENTAGE: usize = 20;

/// Check if the number of things with a certain property are deviant from the rest of the things
/// based on the `MAX_DEVIATION_PERCENTAGE`
fn compute_deviation(number_with: usize, total: usize) -> Option<Deviant> {
    let number_without = total - number_with;
    let deviating = number_with.min(number_without);

    if deviating == 0 || deviating * 100 > MAX_DEVIATION_PERCENTAGE * total {
        // all are the same (with or without), or mixed enough that we don't consider them deviant
        None
    } else if number_with < number_without {
        Some(Deviant::FewWith(number_with))
    } else {
        Some(Deviant::FewWithout(number_without))
    }
}

impl CandidateListSummary {
    pub fn list(store: &AppStore) -> Vec<CandidateListSummary> {
        let lists = store.get_candidate_lists();

        let mut district_count = BTreeMap::<ElectoralDistrict, usize>::new();
        for list in &lists {
            for district in &list.electoral_districts {
                district_count
                    .entry(*district)
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
        }

        lists
            .into_iter()
            .map(|list| {
                let max_count = store.get_political_group().get_max_candidates();
                let duplicate_districts = list
                    .electoral_districts
                    .iter()
                    .filter(|district| *district_count.entry(**district).or_default() > 1)
                    .cloned()
                    .collect();

                CandidateListSummary {
                    list,
                    max_count,
                    duplicate_districts,
                }
            })
            .collect()
    }

    pub fn get_deviation_problems(&self, store: &AppStore) -> Vec<PotentialProblems> {
        let total = self.candidate_count();
        if total <= 1 {
            return Vec::new();
        }

        let mut with_first_name = 0;
        let mut with_gender = 0;
        for candidate in &self.list.candidates {
            let Ok(person) = store.get_person(*candidate) else {
                continue;
            };

            if !person.name.first_name.is_empty_or_none() {
                with_first_name += 1;
            }

            if person.personal_data.gender.is_some() {
                with_gender += 1;
            }
        }

        [
            compute_deviation(with_first_name, total).map(|d| match d {
                Deviant::FewWith(count) => {
                    PotentialProblems::FewCandidatesWithFirstName { count, total }
                }
                Deviant::FewWithout(count) => {
                    PotentialProblems::FewCandidatesWithoutFirstName { count, total }
                }
            }),
            compute_deviation(with_gender, total).map(|d| match d {
                Deviant::FewWith(count) => {
                    PotentialProblems::FewCandidatesWithGender { count, total }
                }
                Deviant::FewWithout(count) => {
                    PotentialProblems::FewCandidatesWithoutGender { count, total }
                }
            }),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn candidate_count(&self) -> usize {
        self.list.candidates.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        AppError, AppStore,
        candidate_lists::CandidateListId,
        persons::PersonId,
        test_utils::{sample_candidate_list, sample_person, sample_person_with},
    };

    use super::*;

    async fn make_store_with_candidates(
        persons: Vec<crate::persons::Person>,
    ) -> Result<(AppStore, Vec<PersonId>), AppError> {
        let store = AppStore::new_for_test();
        let ids = persons.iter().map(|p| p.id).collect();
        for person in persons {
            person.create(&store).await?;
        }
        Ok((store, ids))
    }

    fn summary_with_candidates(ids: Vec<PersonId>) -> CandidateListSummary {
        let mut list = sample_candidate_list(CandidateListId::new());
        list.candidates = ids;
        CandidateListSummary {
            list,
            max_count: 200,
            duplicate_districts: vec![],
        }
    }

    #[test]
    fn deviation_computation() {
        assert_eq!(compute_deviation(0, 100), None);
        assert_eq!(compute_deviation(1, 100), Some(Deviant::FewWith(1)));
        assert_eq!(
            compute_deviation(MAX_DEVIATION_PERCENTAGE, 100),
            Some(Deviant::FewWith(MAX_DEVIATION_PERCENTAGE))
        );
        assert_eq!(compute_deviation(MAX_DEVIATION_PERCENTAGE + 1, 100), None);
        assert_eq!(
            compute_deviation(100 - MAX_DEVIATION_PERCENTAGE - 1, 100),
            None
        );
        assert_eq!(
            compute_deviation(100 - MAX_DEVIATION_PERCENTAGE, 100),
            Some(Deviant::FewWithout(MAX_DEVIATION_PERCENTAGE))
        );
        assert_eq!(compute_deviation(99, 100), Some(Deviant::FewWithout(1)));
        assert_eq!(compute_deviation(100, 100), None);
    }

    #[tokio::test]
    async fn no_deviation_in_first_name_usage_produces_no_warning() -> Result<(), AppError> {
        let persons = (0..5).map(|_| sample_person(PersonId::new())).collect();
        let (store, ids) = make_store_with_candidates(persons).await?;
        let problems = summary_with_candidates(ids).get_deviation_problems(&store);
        assert!(!problems.iter().any(|p| matches!(
            p,
            PotentialProblems::FewCandidatesWithFirstName { .. }
                | PotentialProblems::FewCandidatesWithoutFirstName { .. }
        )));
        Ok(())
    }

    #[tokio::test]
    async fn small_minority_with_first_name_produces_warning() -> Result<(), AppError> {
        let mut persons: Vec<_> = (0..9)
            .map(|_| sample_person_with(PersonId::new(), None, "Jansen", None, "H."))
            .collect();
        persons.push(sample_person(PersonId::new())); // 1 with first name
        let (store, ids) = make_store_with_candidates(persons).await?;
        let problems = summary_with_candidates(ids).get_deviation_problems(&store);
        assert!(
            problems.contains(&PotentialProblems::FewCandidatesWithFirstName {
                count: 1,
                total: 10
            })
        );
        Ok(())
    }

    #[tokio::test]
    async fn small_minority_without_first_name_produces_warning() -> Result<(), AppError> {
        let mut persons: Vec<_> = (0..9).map(|_| sample_person(PersonId::new())).collect();
        persons.push(sample_person_with(
            PersonId::new(),
            None,
            "Jansen",
            None,
            "H.",
        )); // 1 without
        let (store, ids) = make_store_with_candidates(persons).await?;
        let problems = summary_with_candidates(ids).get_deviation_problems(&store);
        assert!(
            problems.contains(&PotentialProblems::FewCandidatesWithoutFirstName {
                count: 1,
                total: 10
            })
        );
        Ok(())
    }

    #[tokio::test]
    async fn large_minority_in_first_name_usage_produces_no_warning() -> Result<(), AppError> {
        let mut persons: Vec<_> = (0..7)
            .map(|_| sample_person_with(PersonId::new(), None, "Jansen", None, "H."))
            .collect();
        persons.extend((0..3).map(|_| sample_person(PersonId::new()))); // 3/10 = 30%
        let (store, ids) = make_store_with_candidates(persons).await?;
        let problems = summary_with_candidates(ids).get_deviation_problems(&store);
        assert!(!problems.iter().any(|p| matches!(
            p,
            PotentialProblems::FewCandidatesWithFirstName { .. }
                | PotentialProblems::FewCandidatesWithoutFirstName { .. }
        )));
        Ok(())
    }

    #[tokio::test]
    async fn small_minority_with_gender_produces_warning() -> Result<(), AppError> {
        let mut persons: Vec<_> = (0..9)
            .map(|_| {
                let mut p = sample_person(PersonId::new());
                p.personal_data.gender = None;
                p
            })
            .collect();
        persons.push(sample_person(PersonId::new())); // 1 with gender
        let (store, ids) = make_store_with_candidates(persons).await?;
        let problems = summary_with_candidates(ids).get_deviation_problems(&store);
        assert!(
            problems.contains(&PotentialProblems::FewCandidatesWithGender {
                count: 1,
                total: 10
            })
        );
        Ok(())
    }

    #[tokio::test]
    async fn small_minority_without_gender_produces_warning() -> Result<(), AppError> {
        let mut persons: Vec<_> = (0..9).map(|_| sample_person(PersonId::new())).collect();
        let mut person_without_gender = sample_person(PersonId::new());
        person_without_gender.personal_data.gender = None;
        persons.push(person_without_gender);
        let (store, ids) = make_store_with_candidates(persons).await?;
        let problems = summary_with_candidates(ids).get_deviation_problems(&store);
        assert!(
            problems.contains(&PotentialProblems::FewCandidatesWithoutGender {
                count: 1,
                total: 10
            })
        );
        Ok(())
    }

    #[test]
    fn no_incomplete_items() {
        let mut list = sample_candidate_list(CandidateListId::new());
        list.candidates.push(PersonId::new());
        let list_summary = CandidateListSummary {
            list,
            max_count: 200,
            duplicate_districts: Vec::new(),
        };

        assert!(list_summary.get_problems().is_empty());
    }

    #[test]
    fn empty_list_incomplete_items() {
        let mut list = sample_candidate_list(CandidateListId::new());
        list.electoral_districts = Vec::new();
        let list_summary = CandidateListSummary {
            list,
            max_count: 200,
            duplicate_districts: Vec::new(),
        };

        let items = list_summary.get_problems();

        assert_eq!(items.len(), 2);
        assert!(items.contains(&PotentialProblems::NoCandidates));
        assert!(items.contains(&PotentialProblems::NoDistricts));
    }

    #[test]
    fn list_incomplete_items_too_many() {
        let mut list = sample_candidate_list(CandidateListId::new());
        list.candidates.push(PersonId::new());
        let list_summary = CandidateListSummary {
            list,
            max_count: 0,
            duplicate_districts: vec![ElectoralDistrict::PsAmsterdam],
        };

        let items = list_summary.get_problems();

        assert_eq!(items.len(), 2);
        assert!(items.contains(&PotentialProblems::TooManyCandidates { actual: 1, max: 0 }));
        assert!(items.contains(&PotentialProblems::DuplicateDistricts));
    }
}
