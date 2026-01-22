use crate::{
    TokenValue,
    form::*,
    political_group::structs::{ListSubmitter, ListSubmitterId},
};
use chrono::Utc;
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(
    target = "ListSubmitter",
    build = "ListSubmitterForm::build_list_submitter"
)]
#[serde(default)]
pub struct ListSubmitterForm {
    #[validate(with = "validate_length(2, 255)", with = "validate_teletex_chars()")]
    last_name: String,
    #[validate(
        with = "validate_length(1, 255)",
        with = "validate_teletex_chars()",
        optional
    )]
    last_name_prefix: String,
    #[validate(with = "validate_initials()")]
    initials: String,
    #[validate(with = "validate_length(2, 255)")]
    locality: String,
    #[validate(with = "validate_length(2, 16)")]
    postal_code: String,
    #[validate(with = "validate_length(1, 16)")]
    house_number: String,
    #[validate(with = "validate_length(1, 16)", optional)]
    house_number_addition: String,
    #[validate(with = "validate_length(2, 255)")]
    street_name: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl WithCsrfToken for ListSubmitterForm {
    fn with_csrf_token(self, csrf_token: CsrfToken) -> Self {
        ListSubmitterForm {
            csrf_token: csrf_token.value,
            ..self
        }
    }
}

impl ListSubmitterForm {
    fn build_list_submitter(
        validated: ListSubmitterFormValidated,
        current: Option<ListSubmitter>,
    ) -> ListSubmitter {
        if let Some(current) = current {
            ListSubmitter {
                last_name: validated.last_name,
                last_name_prefix: validated.last_name_prefix,
                initials: validated.initials,
                locality: validated.locality,
                postal_code: validated.postal_code,
                house_number: validated.house_number,
                house_number_addition: validated.house_number_addition,
                street_name: validated.street_name,
                ..current
            }
        } else {
            ListSubmitter {
                id: ListSubmitterId::new(),
                last_name: validated.last_name,
                last_name_prefix: validated.last_name_prefix,
                initials: validated.initials,
                locality: validated.locality,
                postal_code: validated.postal_code,
                house_number: validated.house_number,
                house_number_addition: validated.house_number_addition,
                street_name: validated.street_name,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }
    }
}
