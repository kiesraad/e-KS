use serde::Serialize;

use crate::{candidate_lists::CandidateListId, common::{Problems, WithProblems}, persons::Person};

#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub list_id: CandidateListId,
    pub position: usize,
    pub person: Person,
}

pub type CandidateWithProblems = WithProblems<Candidate>;
