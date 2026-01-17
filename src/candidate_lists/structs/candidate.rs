use serde::Serialize;

use crate::{candidate_lists::CandidateListId, persons::Person};

#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub list_id: CandidateListId,
    pub position: i32,
    pub person: Person,
}
