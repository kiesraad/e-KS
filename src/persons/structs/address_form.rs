use serde::{Deserialize, Serialize};
use std::str::FromStr;
use validate::Validate as ValidateDerive;

use crate::{CsrfToken, form::*};

use super::Person;

#[derive(Default, Serialize, Deserialize, Clone, Debug, ValidateDerive)]
#[validate(target = "Person", build = "AddressForm::build_address")]
pub struct AddressForm {
    #[validate(with = "validate_length(2, 255)", optional)]
    pub locality: String,
    #[validate(with = "validate_length(2, 16)", optional)]
    pub postal_code: String,
    #[validate(with = "validate_length(1, 16)", optional)]
    pub house_number: String,
    #[validate(with = "validate_length(1, 16)", optional)]
    pub house_number_addition: String,
    #[validate(with = "validate_length(2, 255)", optional)]
    pub street_name: String,
    #[validate(with = "validate_length(2, 255)", optional)]
    pub custom_country: String,
    #[validate(with = "validate_length(2, 255)", optional)]
    pub custom_region: String,
    #[validate(with = "validate_length(2, 255)", optional)]
    pub address_line_1: String,
    #[validate(with = "validate_length(2, 255)", optional)]
    pub address_line_2: String,
    #[validate(parse = "bool")]
    pub is_dutch: String,
    #[validate(csrf)]
    pub csrf_token: String,
}

impl From<Person> for AddressForm {
    fn from(person: Person) -> Self {
        AddressForm {
            locality: person.locality.unwrap_or_default(),
            postal_code: person.postal_code.unwrap_or_default(),
            house_number: person.house_number.unwrap_or_default(),
            house_number_addition: person.house_number_addition.unwrap_or_default(),
            street_name: person.street_name.unwrap_or_default(),
            custom_country: person.custom_country.unwrap_or_default(),
            custom_region: person.custom_region.unwrap_or_default(),
            address_line_1: person.address_line_1.unwrap_or_default(),
            address_line_2: person.address_line_2.unwrap_or_default(),
            is_dutch: person
                .is_dutch
                .as_ref()
                .map(bool::to_string)
                .unwrap_or("true".to_owned()),
            csrf_token: String::new(),
        }
    }
}

impl WithCsrfToken for AddressForm {
    fn with_csrf_token(self, csrf_token: CsrfToken) -> Self {
        AddressForm {
            csrf_token: csrf_token.value,
            ..self
        }
    }
}

impl AddressForm {
    fn build_address(validated: AddressFormValidated, current: Option<&Person>) -> Person {
        if let Some(current_person) = current {
            // TODO: only save the values based on is_dutch
            Person {
                locality: validated.locality,
                postal_code: validated.postal_code,
                house_number: validated.house_number,
                house_number_addition: validated.house_number_addition,
                street_name: validated.street_name,
                is_dutch: Some(validated.is_dutch),
                custom_country: validated.custom_country,
                custom_region: validated.custom_region,
                address_line_1: validated.address_line_1,
                address_line_2: validated.address_line_2,
                ..current_person.clone()
            }
        } else {
            panic!("Can't update the address of a non-existent person!");
        }
    }
}
