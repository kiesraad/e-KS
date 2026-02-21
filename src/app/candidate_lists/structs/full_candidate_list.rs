use serde::Serialize;

use crate::{
    AppError, Store,
    candidate_lists::{CandidateList, CandidateListId},
    candidates::Candidate,
    persons::PersonId,
};

#[derive(Debug, Clone, Serialize)]
pub struct FullCandidateList {
    pub list: CandidateList,
    pub candidates: Vec<Candidate>,
}

impl FullCandidateList {
    pub fn get(store: &Store, list_id: CandidateListId) -> Result<FullCandidateList, AppError> {
        let list = store.get_candidate_list(list_id)?;

        CandidateList::build_full_candidate_list(store, list)
    }
}

impl FullCandidateList {
    pub fn get_index(&self, person_id: PersonId) -> Option<usize> {
        self.candidates
            .iter()
            .position(|c| c.person.id == person_id)
    }

    pub fn contains(&self, person_id: PersonId) -> bool {
        self.candidates.iter().any(|c| c.person.id == person_id)
    }

    pub fn get_ids(&self) -> Vec<PersonId> {
        self.candidates.iter().map(|c| c.person.id).collect()
    }

    pub fn id(&self) -> CandidateListId {
        self.list.id
    }
}
