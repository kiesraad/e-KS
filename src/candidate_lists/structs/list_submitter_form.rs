use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{TokenValue, candidate_lists::CandidateList, political_groups::ListSubmitterId};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "CandidateList")]
#[serde(default)]
pub struct ListSubmitterForm {
    #[validate(parse = "ListSubmitterId", optional)]
    pub list_submitter_id: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<CandidateList> for ListSubmitterForm {
    fn from(value: CandidateList) -> Self {
        ListSubmitterForm {
            list_submitter_id: value
                .list_submitter_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            csrf_token: Default::default(),
        }
    }
}
