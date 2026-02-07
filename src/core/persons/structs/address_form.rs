use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{DutchAddressForm, TokenValue, persons::Person};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "Person")]
pub struct AddressForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub address: DutchAddressForm,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<Person> for AddressForm {
    fn from(person: Person) -> Self {
        AddressForm {
            address: DutchAddressForm::from(person.address),
            csrf_token: Default::default(),
        }
    }
}
