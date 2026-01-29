use crate::{
    TokenValue,
    political_groups::{AuthorisedAgentId, PoliticalGroup},
};
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "PoliticalGroup")]
#[serde(default)]
pub struct PoliticalGroupForm {
    #[validate(parse = "bool", optional)]
    pub long_list_allowed: String,
    #[validate(parse = "bool", optional)]
    pub legal_name_confirmed: String,
    #[validate(parse = "bool", optional)]
    pub display_name_confirmed: String,
    #[validate(parse = "AuthorisedAgentId", optional)]
    pub authorised_agent_id: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<PoliticalGroup> for PoliticalGroupForm {
    fn from(value: PoliticalGroup) -> Self {
        PoliticalGroupForm {
            long_list_allowed: value
                .long_list_allowed
                .map(|value| value.to_string())
                .unwrap_or_default(),
            legal_name_confirmed: value
                .legal_name_confirmed
                .map(|value| value.to_string())
                .unwrap_or_default(),
            display_name_confirmed: value
                .display_name_confirmed
                .map(|value| value.to_string())
                .unwrap_or_default(),
            authorised_agent_id: value
                .authorised_agent_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            csrf_token: Default::default(),
        }
    }
}
