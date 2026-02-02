use serde::Deserialize;
use validate::Validate;

use crate::{
    TokenValue,
    form::{validate_length, validate_teletex_chars},
    political_groups::PoliticalGroup,
};

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "PoliticalGroup")]
#[serde(default)]
pub struct PoliticalGroupForm {
    #[validate(parse = "bool", optional)]
    pub long_list_allowed: String,
    #[validate(
        with = "validate_length(2, 255)",
        with = "validate_teletex_chars()",
        optional
    )]
    pub legal_name: String,
    #[validate(
        with = "validate_length(2, 255)",
        with = "validate_teletex_chars()",
        optional
    )]
    pub display_name: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<PoliticalGroup> for PoliticalGroupForm {
    fn from(value: PoliticalGroup) -> Self {
        PoliticalGroupForm {
            long_list_allowed: value
                .long_list_allowed
                .map(|b| b.to_string())
                .unwrap_or_default(),
            legal_name: value.legal_name.unwrap_or_default(),
            display_name: value.display_name.unwrap_or_default(),
            csrf_token: Default::default(),
        }
    }
}
