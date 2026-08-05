use crate::structs::{candidate_lists::CandidateList, candidates::CandidateWithProblems};

#[derive(Debug, Clone)]
pub struct FullCandidateList {
    pub list: CandidateList,
    pub candidates: Vec<CandidateWithProblems>,
}

#[cfg(test)]
impl FullCandidateList {
    pub fn contains(&self, person_id: crate::structs::persons::PersonId) -> bool {
        self.candidates
            .iter()
            .any(|c| c.data.person.id == person_id)
    }

    pub fn id(&self) -> crate::structs::candidate_lists::CandidateListId {
        self.list.id
    }
}
