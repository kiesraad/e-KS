use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    AppStore, ElectoralDistrict,
    candidate_lists::CandidateList,
    submit::{Completable, IncompleteItem},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CandidateListSummary {
    pub list: CandidateList,
    pub person_count: usize,
    pub max_count: usize,
    pub duplicate_districts: Vec<ElectoralDistrict>,
}

impl Completable for CandidateListSummary {
    fn incomplete_items(&self) -> Vec<IncompleteItem> {
        let mut items = vec![];
        if self.person_count == 0 {
            items.push(IncompleteItem::NoCandidates);
        } else if self.person_count > self.max_count {
            items.push(IncompleteItem::TooManyCandidates {
                actual: self.person_count,
                max: self.max_count,
            });
        }
        if !self.duplicate_districts.is_empty() {
            items.push(IncompleteItem::DuplicateDistricts {
                duplicates: self.duplicate_districts.clone(),
            });
        }
        if self.list.electoral_districts.is_empty() {
            items.push(IncompleteItem::NoDistricts)
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
                let person_count = list.candidates.len();
                let max_count = store.get_political_group().get_max_candidates();
                let duplicate_districts = list
                    .electoral_districts
                    .iter()
                    .filter(|district| *district_count.entry(**district).or_default() > 1)
                    .cloned()
                    .collect();

                CandidateListSummary {
                    list,
                    person_count,
                    max_count,
                    duplicate_districts,
                }
            })
            .collect()
    }
}
