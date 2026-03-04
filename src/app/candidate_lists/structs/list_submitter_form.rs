use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{TokenValue, candidate_lists::CandidateList, list_submitters::ListSubmitterId};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "CandidateList")]
#[serde(default)]
pub struct ListSubmitterForm {
    #[validate(parse = "ListSubmitterId", optional)]
    pub list_submitter_id: String,
    pub substitute_list_submitter_ids: Vec<ListSubmitterId>,
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
            substitute_list_submitter_ids: value.substitute_list_submitter_ids,
            csrf_token: Default::default(),
        }
    }
}
