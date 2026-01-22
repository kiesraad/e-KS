use crate::{
    TokenValue,
    form::*,
    political_group::structs::{
        AuthorisedAgentId, ListSubmitterId, PoliticalGroup, PoliticalGroupId,
    },
};
use chrono::Utc;
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(
    target = "PoliticalGroup",
    build = "PoliticalGroupForm::build_political_group"
)]
pub struct PoliticalGroupForm {
    #[validate(with = "validate_length(2, 255)")]
    legal_name: String,
    #[validate(with = "validate_length(2, 255)")]
    display_name: String,
    #[validate(parse = "AuthorisedAgentId", optional)]
    authorised_agent_id: String,
    #[validate(parse = "ListSubmitterId", optional)]
    list_submitter_id: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl WithCsrfToken for PoliticalGroupForm {
    fn with_csrf_token(self, csrf_token: CsrfToken) -> Self {
        PoliticalGroupForm {
            csrf_token: csrf_token.value,
            ..self
        }
    }
}

impl PoliticalGroupForm {
    fn build_political_group(
        validated: PoliticalGroupFormValidated,
        current: Option<PoliticalGroup>,
    ) -> PoliticalGroup {
        if let Some(current) = current {
            current
        } else {
            PoliticalGroup {
                id: PoliticalGroupId::new(),

                legal_name: validated.legal_name,
                display_name: validated.display_name,
                authorised_agent_id: validated.authorised_agent_id,
                list_submitter_id: validated.list_submitter_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }
    }
}
