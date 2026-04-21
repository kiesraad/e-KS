use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    AppError, AppStore, ElectoralDistrict,
    candidate_lists::CandidateList,
    common::{Completable, IncompleteItem},
    political_groups::PoliticalGroup,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CandidateListSummary {
    pub list: CandidateList,
    pub person_count: usize,
    pub max_candidates: usize,
    // TODO extend with a reference to the list that contains the duplicate such that in the submit validation, both are easily navigable.
    pub duplicate_districts: Vec<ElectoralDistrict>,
    pub political_group: PoliticalGroup,
}

impl Completable for CandidateListSummary {
    fn incomplete_items(&self) -> Vec<IncompleteItem<'_>> {
        let mut items = self.political_group.incomplete_items();

        if self.person_count == 0 {
            items.push(IncompleteItem::NoCandidates {
                candidate_list: &self.list,
            });
        };

        if items.contains(&IncompleteItem::LongListAllowedIsNone) // max candidate check is meaningless if longListAllowed is not provided
            && self.person_count > self.max_candidates
        {
            items.push(IncompleteItem::TooManyCandidates {
                candidate_list: &self.list,
                actual: self.person_count,
                max: self.max_candidates,
            })
        }
        if !self.duplicate_districts.is_empty() {
            items.push(IncompleteItem::DuplicateDistricts {
                candidate_list: &self.list,
                duplicates: &self.duplicate_districts,
            })
        }

        items
    }
}

impl CandidateListSummary {
    pub fn list(
        store: &AppStore,
        max_candidates: usize,
    ) -> Result<Vec<CandidateListSummary>, AppError> {
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

        let summaries = lists
            .into_iter()
            .map(|list| {
                let person_count = list.candidates.len();
                let political_group = store.get_political_group();
                let duplicate_districts = list
                    .electoral_districts
                    .iter()
                    .filter(|district| *district_count.entry(**district).or_default() > 1)
                    .cloned()
                    .collect();

                CandidateListSummary {
                    list,
                    max_candidates,
                    person_count,
                    duplicate_districts,
                    political_group,
                }
            })
            .collect();

        Ok(summaries)
    }
}
