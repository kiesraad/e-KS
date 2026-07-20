use crate::{
    AppError, PgStore,
    candidate_lists::{CandidateList, CandidateListId},
    candidates::CandidateWithProblems,
};

#[derive(Debug, Clone)]
pub struct FullCandidateList {
    pub list: CandidateList,
    pub candidates: Vec<CandidateWithProblems>,
}

impl FullCandidateList {
    pub fn get(store: &PgStore, list_id: CandidateListId) -> Result<FullCandidateList, AppError> {
        let list = store.get_candidate_list(list_id)?;

        CandidateList::build_full_candidate_list(store, list)
    }
}

#[cfg(test)]
impl FullCandidateList {
    pub fn contains(&self, person_id: crate::persons::PersonId) -> bool {
        self.candidates
            .iter()
            .any(|c| c.data.person.id == person_id)
    }

    pub fn id(&self) -> CandidateListId {
        self.list.id
    }
}
