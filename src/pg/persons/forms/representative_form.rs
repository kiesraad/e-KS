use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    common::{DutchAddressForm, MinimalNameForm},
    structs::persons::Representative,
};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "Representative")]
#[serde(default)]
pub struct RepresentativeForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: MinimalNameForm,
    #[validate(flatten)]
    #[serde(flatten)]
    pub address: DutchAddressForm,
}

impl RepresentativeForm {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.address.is_empty()
    }
}

impl From<Representative> for RepresentativeForm {
    fn from(representative: Representative) -> Self {
        Self {
            name: MinimalNameForm::from(representative.name),
            address: DutchAddressForm::from(representative.address),
        }
    }
}
