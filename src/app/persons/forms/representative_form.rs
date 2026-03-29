use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    TokenValue,
    common::{DutchAddressForm, FullNameForm},
    persons::{Person, Representative},
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

impl From<Representative> for RepresentativeFieldsForm {
    fn from(person: Representative) -> Self {
        Self {
            name: FullNameForm::from(person.name),
            address: DutchAddressForm::from(person.address),
        }
    }
}

impl RepresentativeFieldsForm {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty() && self.address.is_empty()
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "Person")]
#[serde(default)]
pub struct RepresentativeForm {
    #[validate(flatten)]
    #[serde(flatten)]
    pub representative: Option<RepresentativeFieldsForm>,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<Person> for RepresentativeForm {
    fn from(person: Person) -> Self {
        Self {
            representative: person.representative.map(|r| r.into()),
            csrf_token: TokenValue::default(),
        }
    }
}

impl RepresentativeForm {
    pub fn representative_fields(&self) -> RepresentativeFieldsForm {
        self.representative.clone().unwrap_or_default()
    }
}
