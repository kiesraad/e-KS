use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    AppStore, ElectoralDistrict,
    candidate_lists::CandidateList,
    submit::{PotentialProblems, Problematic},
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
            items.push(PotentialProblems::DuplicateDistricts {
                duplicates: self.duplicate_districts.clone(),
            });
        }
        if self.list.electoral_districts.is_empty() {
            items.push(PotentialProblems::NoDistricts)
        }
        items
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

    pub fn candidate_count(&self) -> usize {
        self.list.candidates.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        candidate_lists::CandidateListId, persons::PersonId, test_utils::sample_candidate_list,
    };

    use super::*;

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
        assert!(items.contains(&PotentialProblems::DuplicateDistricts {
            duplicates: vec![ElectoralDistrict::PsAmsterdam]
        }));
    }
}
