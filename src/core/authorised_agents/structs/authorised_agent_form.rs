use crate::{FullNameForm, TokenValue, authorised_agents::AuthorisedAgent};
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "AuthorisedAgent")]
pub struct AuthorisedAgentForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: FullNameForm,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<AuthorisedAgent> for AuthorisedAgentForm {
    fn from(value: AuthorisedAgent) -> Self {
        AuthorisedAgentForm {
            name: FullNameForm::from(value.name),
            csrf_token: Default::default(),
        }
    }
}
