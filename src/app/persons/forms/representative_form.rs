use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    TokenValue,
    common::{DutchAddressForm, FullNameForm},
    persons::Representative,
};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "Representative")]
#[serde(default)]
pub struct RepresentativeFieldsForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: FullNameForm,
    #[validate(flatten)]
    #[serde(flatten)]
    pub address: DutchAddressForm,
}

impl RepresentativeFieldsForm {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.address.is_empty()
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "Representative")]
#[serde(default)]
pub struct RepresentativeForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: FullNameForm,
    #[validate(flatten)]
    #[serde(flatten)]
    pub address: DutchAddressForm,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<Representative> for RepresentativeForm {
    fn from(representative: Representative) -> Self {
        Self {
            name: FullNameForm::from(representative.name),
            address: DutchAddressForm::from(representative.address),
            csrf_token: TokenValue::default(),
        }
    }
}
