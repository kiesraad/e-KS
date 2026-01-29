use crate::{
    TokenValue,
    form::{validate_initials, validate_length, validate_teletex_chars},
    political_groups::ListSubmitter,
};
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "ListSubmitter")]
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
