use serde::Deserialize;
use validate::Validate;

use crate::{
    TokenValue,
    form::{CsrfToken, WithCsrfToken},
    political_groups::{ListSubmitterId, PoliticalGroup},
};

#[derive(Debug, Validate, Default, Deserialize)]
#[validate(
    target = "PoliticalGroup",
    build = "PreferredSubmitterForm::build_political_group"
)]
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

impl PreferredSubmitterForm {
    fn build_political_group(
        validated: PreferredSubmitterFormValidated,
        current: Option<PoliticalGroup>,
    ) -> PoliticalGroup {
        if let Some(current_person) = current {
            PoliticalGroup {
                list_submitter_id: validated.list_submitter_id,
                ..current_person
            }
        } else {
            panic!("Can't update the default list submitter of a non-existent political group!");
        }
    }
}

impl WithCsrfToken for PreferredSubmitterForm {
    fn with_csrf_token(self, csrf_token: CsrfToken) -> Self {
        Self {
            csrf_token: csrf_token.value,
            ..self
        }
    }
}
