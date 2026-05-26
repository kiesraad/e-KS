use crate::{
    OptionStringExt, TokenValue,
    authorised_agents::AuthorisedAgent,
    common::{LegalName, MinimalNameForm},
};
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "AuthorisedAgent")]
#[serde(default)]
pub struct AuthorisedAgentForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: MinimalNameForm,
    #[validate(parse = "LegalName", optional)]
    pub legal_name: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<AuthorisedAgent> for AuthorisedAgentForm {
    fn from(value: AuthorisedAgent) -> Self {
        AuthorisedAgentForm {
            name: MinimalNameForm::from(value.name),
            legal_name: value.legal_name.to_string_or_default(),
            csrf_token: Default::default(),
        }
    }
}
