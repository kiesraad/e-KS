use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{DutchAddressForm, FullNameForm, TokenValue, persons::Person};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "Person")]
pub struct RepresentativeForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub representative: FullNameForm,
    #[validate(flatten)]
    #[serde(flatten)]
    pub address: DutchAddressForm,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<Person> for RepresentativeForm {
    fn from(person: Person) -> Self {
        RepresentativeForm {
            representative: FullNameForm::from(person.representative),
            address: DutchAddressForm::from(person.address),
            csrf_token: Default::default(),
        }
    }
}
