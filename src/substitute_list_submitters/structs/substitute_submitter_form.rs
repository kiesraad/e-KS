use crate::{
    TokenValue,
    form::{
        validate_house_number_addition, validate_housenumber, validate_initials,
        validate_last_name_prefix, validate_length, validate_no_last_name_prefix,
        validate_postal_code, validate_teletex_chars,
    },
    substitute_list_submitters::SubstituteSubmitter,
};
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "SubstituteSubmitter")]
#[serde(default)]
pub struct SubstituteSubmitterForm {
    #[validate(
        with = "validate_length(2, 255)",
        with = "validate_teletex_chars()",
        with = "validate_no_last_name_prefix()"
    )]
    pub last_name: String,
    #[validate(with = "validate_last_name_prefix()", optional)]
    pub last_name_prefix: String,
    #[validate(with = "validate_initials()")]
    pub initials: String,
    #[validate(
        with = "validate_length(2, 255)",
        with = "validate_teletex_chars()",
        optional
    )]
    pub locality: String,
    #[validate(with = "validate_postal_code()", optional)]
    pub postal_code: String,
    #[validate(with = "validate_housenumber()", optional)]
    pub house_number: String,
    #[validate(with = "validate_house_number_addition()", optional)]
    pub house_number_addition: String,
    #[validate(
        with = "validate_length(2, 255)",
        with = "validate_teletex_chars()",
        optional
    )]
    pub street_name: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<SubstituteSubmitter> for SubstituteSubmitterForm {
    fn from(value: SubstituteSubmitter) -> Self {
        SubstituteSubmitterForm {
            last_name: value.last_name,
            last_name_prefix: value.last_name_prefix.unwrap_or_default(),
            initials: value.initials,
            locality: value.locality.unwrap_or_default(),
            postal_code: value.postal_code.unwrap_or_default(),
            house_number: value.house_number.unwrap_or_default(),
            house_number_addition: value.house_number_addition.unwrap_or_default(),
            street_name: value.street_name.unwrap_or_default(),
            csrf_token: Default::default(),
        }
    }
}
