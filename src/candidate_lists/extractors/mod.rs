use serde::Deserialize;

use crate::{candidate_lists::CandidateListId, persons::PersonId};

mod candidate;
mod candidate_list;
mod full_candidate_list;

#[derive(Deserialize)]
struct CandidateListPathParams {
    #[serde(alias = "list_id")]
    list_id: CandidateListId,
}

#[derive(Deserialize)]
struct CandidateListAndPersonPathParams {
    #[serde(alias = "list_id")]
    list_id: CandidateListId,
    #[serde(alias = "person_id")]
    person_id: PersonId,
}
