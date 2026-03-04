use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{TokenValue, candidate_lists::CandidateList};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "CandidateList")]
#[serde(default)]
pub struct CandidateListCreateForm {
    #[validate(not_empty)]
    pub electoral_districts: Vec<crate::ElectoralDistrict>,
    #[validate(ignore)]
    pub copy_candidates: bool,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}
