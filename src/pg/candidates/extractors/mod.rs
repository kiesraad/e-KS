use serde::Deserialize;

use crate::{candidate_lists::CandidateListId, persons::PersonId};

mod candidate;

#[derive(Deserialize)]
struct CandidateListAndPersonPathParams {
    #[serde(alias = "list_id")]
    list_id: CandidateListId,
    #[serde(alias = "person_id")]
    person_id: PersonId,
}
