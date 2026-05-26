use crate::{
    TokenValue,
    common::{LegalName, MinimalNameForm},
    name_authorisations::NameAuthorisation,
};
use serde::Deserialize;
use validate::Validate;

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "NameAuthorisation")]
#[serde(default)]
pub struct NameAuthorisationForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: MinimalNameForm,
    #[validate(parse = "LegalName")]
    pub legal_name: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<NameAuthorisation> for NameAuthorisationForm {
    fn from(value: NameAuthorisation) -> Self {
        NameAuthorisationForm {
            name: MinimalNameForm::from(value.name),
            legal_name: value.legal_name.to_string(),
            csrf_token: Default::default(),
        }
    }
}
