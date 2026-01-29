use serde::Deserialize;
use validate::Validate;

use crate::{
    TokenValue,
    political_groups::{ListSubmitterId, PoliticalGroup},
};

#[derive(Debug, Default, Deserialize, Validate)]
#[validate(target = "PoliticalGroup")]
pub struct PreferredSubmitterForm {
    #[validate(parse = "ListSubmitterId", optional)]
    pub list_submitter_id: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<PoliticalGroup> for PreferredSubmitterForm {
    fn from(political_group: PoliticalGroup) -> Self {
        PreferredSubmitterForm {
            list_submitter_id: political_group
                .list_submitter_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            csrf_token: Default::default(),
        }
    }
}
