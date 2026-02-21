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
    fn from(person: Representative) -> Self {
        RepresentativeForm {
            name: FullNameForm::from(person.name),
            address: DutchAddressForm::from(person.address),
            csrf_token: Default::default(),
        }
    }
}
