use serde::Deserialize;

use crate::candidate_lists::CandidateListId;

mod candidate_list;
mod full_candidate_list;

#[derive(Deserialize)]
struct CandidateListPathParams {
    #[serde(alias = "list_id")]
    list_id: CandidateListId,
}
