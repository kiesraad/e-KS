use serde::Serialize;

use crate::structs::{candidate_lists::CandidateListId, common::WithProblems, persons::Person};

#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub list_id: CandidateListId,
    pub position: usize,
    pub person: Person,
}

pub type CandidateWithProblems = WithProblems<Candidate>;
