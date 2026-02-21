use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{AppError, ElectoralDistrict, Store, candidate_lists::CandidateList};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CandidateListSummary {
    pub list: CandidateList,
    pub person_count: usize,
    pub duplicate_districts: Vec<ElectoralDistrict>,
}

impl CandidateListSummary {
    pub fn list(store: &Store) -> Result<Vec<CandidateListSummary>, AppError> {
        let lists = store.get_candidate_lists()?;

        let mut district_count = BTreeMap::<ElectoralDistrict, usize>::new();
        for list in &lists {
            for district in &list.electoral_districts {
                district_count
                    .entry(*district)
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
        }

        let summaries = lists
            .into_iter()
            .map(|list| {
                let person_count = list.candidates.len();
                let duplicate_districts = list
                    .electoral_districts
                    .iter()
                    .filter(|district| *district_count.entry(**district).or_default() > 1)
                    .cloned()
                    .collect();

                CandidateListSummary {
                    list,
                    person_count,
                    duplicate_districts,
                }
            })
            .collect();

        Ok(summaries)
    }
}
