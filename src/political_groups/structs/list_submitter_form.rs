use crate::{
    TokenValue,
    form::{CsrfToken, WithCsrfToken, validate_initials, validate_length, validate_teletex_chars},
    political_groups::ListSubmitter,
};
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
    pub last_name: String,
    #[validate(
        with = "validate_length(1, 255)",
        with = "validate_teletex_chars()",
        optional
    )]
    pub last_name_prefix: String,
    #[validate(with = "validate_initials()")]
    pub initials: String,
    #[validate(with = "validate_length(2, 255)")]
    pub locality: String,
    #[validate(with = "validate_length(2, 16)")]
    pub postal_code: String,
    #[validate(with = "validate_length(1, 16)")]
    pub house_number: String,
    #[validate(with = "validate_length(1, 16)", optional)]
    pub house_number_addition: String,
    #[validate(with = "validate_length(2, 255)")]
    pub street_name: String,
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
                last_name: validated.last_name,
                last_name_prefix: validated.last_name_prefix,
                initials: validated.initials,
                locality: validated.locality,
                postal_code: validated.postal_code,
                house_number: validated.house_number,
                house_number_addition: validated.house_number_addition,
                street_name: validated.street_name,
                ..Default::default()
            }
        }
    }
}

impl From<ListSubmitter> for ListSubmitterForm {
    fn from(value: ListSubmitter) -> Self {
        ListSubmitterForm {
            last_name: value.last_name,
            last_name_prefix: value.last_name_prefix.unwrap_or_default(),
            initials: value.initials,
            locality: value.locality,
            postal_code: value.postal_code,
            house_number: value.house_number,
            house_number_addition: value.house_number_addition.unwrap_or_default(),
            street_name: value.street_name,
            csrf_token: Default::default(),
        }
    }
}
