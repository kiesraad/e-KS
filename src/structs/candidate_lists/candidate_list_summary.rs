use serde::{Deserialize, Serialize};

use crate::{
    ElectoralDistrict, OptionAsStrExt,
    structs::{
        candidate_lists::{CandidateList, FullCandidateList},
        common::{InfoProblems, PotentialProblems, Problematic, Problems, WithProblems},
    },
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CandidateListSummary {
    pub list: CandidateList,
    pub max_count: usize,
    pub duplicate_districts: Vec<ElectoralDistrict>,
}

pub type CandidateListWithProblems = WithProblems<CandidateListSummary>;

impl Problematic<FullCandidateList> for CandidateListSummary {
    fn get_problems(&self, full_list: FullCandidateList) -> Problems {
        Problems {
            potential_problems: self.get_potential_problems(),
            info_problems: self.get_info_problems(full_list),
        }
    }
}

impl CandidateListSummary {
    fn get_info_problems(&self, full_list: FullCandidateList) -> Vec<InfoProblems> {
        let mut items = Vec::new();
        let total = self.candidate_count();
        if total <= 1 {
            return items;
        }

        let mut with_first_name = 0;
        let mut with_gender = 0;
        for candidate in full_list.candidates {
            if !candidate.data.person.name.first_name.is_empty_or_none() {
                with_first_name += 1;
            }

            if candidate.data.person.personal_data.gender.is_some() {
                with_gender += 1;
            }
        }

        items.extend(
            [
                compute_deviation(with_first_name, total).map(|d| match d {
                    Deviant::FewWith(count) => {
                        InfoProblems::FewCandidatesWithFirstName { count, total }
                    }
                    Deviant::FewWithout(count) => {
                        InfoProblems::FewCandidatesWithoutFirstName { count, total }
                    }
                }),
                compute_deviation(with_gender, total).map(|d| match d {
                    Deviant::FewWith(count) => {
                        InfoProblems::FewCandidatesWithGender { count, total }
                    }
                    Deviant::FewWithout(count) => {
                        InfoProblems::FewCandidatesWithoutGender { count, total }
                    }
                }),
            ]
            .into_iter()
            .flatten(),
        );

        items
    }

    fn get_potential_problems(&self) -> Vec<PotentialProblems> {
        let mut items = Vec::new();

        if !self.duplicate_districts.is_empty() {
            items.push(PotentialProblems::DuplicateDistricts);
        }

        if self.list.electoral_districts.is_empty() {
            items.push(PotentialProblems::NoDistricts)
        }

        if self.candidate_count() == 0 {
            items.push(PotentialProblems::NoCandidates);
        } else if self.candidate_count() > self.max_count {
            items.push(PotentialProblems::TooManyCandidates {
                count: self.candidate_count() - self.max_count,
            });
        }

        items
    }

    pub fn candidate_count(&self) -> usize {
        self.list.candidates.len()
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

#[cfg(test)]
mod tests {
    use crate::{
        AppError,
        ElectoralDistrict::PsAmsterdam,
        PgStore,
        structs::{candidate_lists::CandidateListId, persons::PersonId},
        test_utils::{sample_candidate_list, sample_person, sample_person_with},
    };

    use super::*;

    async fn make_store_with_candidates(
        persons: Vec<crate::structs::persons::Person>,
    ) -> Result<(PgStore, Vec<PersonId>), AppError> {
        let store = PgStore::new_for_test();
        let ids = persons.iter().map(|p| p.id).collect();
        for person in persons {
            person.create(&store).await?;
        }
        Ok((store, ids))
    }

    async fn create_summary_and_get_problems(
        ids: Vec<PersonId>,
        store: &PgStore,
        districts: Vec<ElectoralDistrict>,
        duplicate_districts: Vec<ElectoralDistrict>,
    ) -> Result<Problems, AppError> {
        let id = CandidateListId::new();
        let mut list = sample_candidate_list(id);
        list.candidates = ids;
        list.electoral_districts = districts;
        list.create(store).await?;
        Ok(CandidateListSummary {
            list,
            max_count: 20,
            duplicate_districts,
        }
        .get_problems(FullCandidateList::get(store, id).unwrap()))
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
        let problems = create_summary_and_get_problems(ids, &store, Vec::new(), Vec::new()).await?;
        assert!(!problems.info_problems.iter().any(|p| matches!(
            p,
            InfoProblems::FewCandidatesWithFirstName { .. }
                | InfoProblems::FewCandidatesWithoutFirstName { .. }
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
        let problems = create_summary_and_get_problems(ids, &store, Vec::new(), Vec::new()).await?;
        assert!(
            problems
                .info_problems
                .contains(&InfoProblems::FewCandidatesWithFirstName {
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
        let problems = create_summary_and_get_problems(ids, &store, Vec::new(), Vec::new()).await?;
        assert!(
            problems
                .info_problems
                .contains(&InfoProblems::FewCandidatesWithoutFirstName {
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
        let problems = create_summary_and_get_problems(ids, &store, Vec::new(), Vec::new()).await?;
        assert!(!problems.info_problems.iter().any(|p| matches!(
            p,
            InfoProblems::FewCandidatesWithFirstName { .. }
                | InfoProblems::FewCandidatesWithoutFirstName { .. }
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
        let problems = create_summary_and_get_problems(ids, &store, Vec::new(), Vec::new()).await?;
        assert!(
            problems
                .info_problems
                .contains(&InfoProblems::FewCandidatesWithGender {
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
        let problems = create_summary_and_get_problems(ids, &store, Vec::new(), Vec::new()).await?;
        assert!(
            problems
                .info_problems
                .contains(&InfoProblems::FewCandidatesWithoutGender {
                    count: 1,
                    total: 10
                })
        );
        Ok(())
    }

    #[tokio::test]
    async fn no_problems() -> Result<(), AppError> {
        let (store, ids) = make_store_with_candidates(vec![sample_person(PersonId::new())]).await?;
        let problems = create_summary_and_get_problems(
            ids,
            &store,
            vec![ElectoralDistrict::PsAmsterdam],
            Vec::new(),
        )
        .await?;

        assert!(problems.info_problems.is_empty());
        assert!(problems.potential_problems.is_empty());
        Ok(())
    }

    #[tokio::test]

    async fn empty_list_problems() -> Result<(), AppError> {
        let (store, ids) = make_store_with_candidates(Vec::new()).await?;
        let problems = create_summary_and_get_problems(ids, &store, Vec::new(), Vec::new()).await?;

        assert_eq!(problems.potential_problems.len(), 2);
        assert!(
            problems
                .potential_problems
                .contains(&PotentialProblems::NoCandidates)
        );
        assert!(
            problems
                .potential_problems
                .contains(&PotentialProblems::NoDistricts)
        );

        assert!(problems.info_problems.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn list_problems_too_many() -> Result<(), AppError> {
        let persons: Vec<_> = (0..25).map(|_| sample_person(PersonId::new())).collect();
        let (store, ids) = make_store_with_candidates(persons).await?;
        let problems =
            create_summary_and_get_problems(ids, &store, vec![PsAmsterdam], vec![PsAmsterdam])
                .await?;

        // list with duplicate district
        let mut list = sample_candidate_list(CandidateListId::new());
        list.electoral_districts = vec![ElectoralDistrict::PsAmsterdam];
        list.create(&store).await?;

        assert_eq!(problems.potential_problems.len(), 2);
        assert!(
            problems
                .potential_problems
                .contains(&PotentialProblems::TooManyCandidates { count: 5 })
        );
        assert!(
            problems
                .potential_problems
                .contains(&PotentialProblems::DuplicateDistricts)
        );

        assert!(problems.info_problems.is_empty());

        Ok(())
    }
}
