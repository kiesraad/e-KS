use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    TokenValue,
    form::{validate_length, validate_post_code},
    persons::Person,
};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "Person")]
pub struct AddressForm {
    #[validate(with = "validate_length(2, 255)", optional)]
    pub locality: String,
    #[validate(with = "validate_post_code()", optional)]
    pub postal_code: String,
    #[validate(with = "validate_length(1, 16)", optional)]
    pub house_number: String,
    #[validate(with = "validate_length(1, 16)", optional)]
    pub house_number_addition: String,
    #[validate(with = "validate_length(2, 255)", optional)]
    pub street_name: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<Person> for AddressForm {
    fn from(person: Person) -> Self {
        AddressForm {
            locality: person.locality.unwrap_or_default(),
            postal_code: person.postal_code.unwrap_or_default(),
            house_number: person.house_number.unwrap_or_default(),
            house_number_addition: person.house_number_addition.unwrap_or_default(),
            street_name: person.street_name.unwrap_or_default(),
            csrf_token: Default::default(),
        }
    }
}
