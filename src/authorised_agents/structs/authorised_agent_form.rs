use crate::{
    TokenValue,
    authorised_agents::AuthorisedAgent,
    form::{
        validate_initials, validate_last_name_prefix, validate_length,
        validate_no_last_name_prefix, validate_teletex_chars,
    },
};
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "AuthorisedAgent")]
#[serde(default)]
pub struct AuthorisedAgentForm {
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
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<AuthorisedAgent> for AuthorisedAgentForm {
    fn from(value: AuthorisedAgent) -> Self {
        AuthorisedAgentForm {
            last_name: value.last_name,
            last_name_prefix: value.last_name_prefix.unwrap_or_default(),
            initials: value.initials,
            csrf_token: Default::default(),
        }
    }
}
