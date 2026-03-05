use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    OptionStringExt,
    common::{DutchAddress, HouseNumber, HouseNumberAddition, Locality, PostalCode, StreetName},
};

/// Form-friendly representation of a Dutch address.
///
/// Uses `String` fields so it can be bound to form inputs and validated.
#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "DutchAddress")]
#[serde(default)]
pub struct DutchAddressForm {
    #[validate(parse = "Locality", optional)]
    pub locality: String,
    #[validate(parse = "PostalCode", optional)]
    pub postal_code: String,
    #[validate(parse = "HouseNumber", optional)]
    pub house_number: String,
    #[validate(parse = "HouseNumberAddition", optional)]
    pub house_number_addition: String,
    #[validate(parse = "StreetName", optional)]
    pub street_name: String,
}

impl From<DutchAddress> for DutchAddressForm {
    fn from(address: DutchAddress) -> Self {
        DutchAddressForm {
            locality: address.locality.to_string_or_default(),
            postal_code: address.postal_code.to_string_or_default(),
            house_number: address.house_number.to_string_or_default(),
            house_number_addition: address.house_number_addition.to_string_or_default(),
            street_name: address.street_name.to_string_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_from_address_uses_empty_strings_for_missing_parts() {
        let address = DutchAddress::default();
        let form = DutchAddressForm::from(address);

        assert_eq!("", form.street_name);
        assert_eq!("", form.house_number);
        assert_eq!("", form.house_number_addition);
        assert_eq!("", form.postal_code);
        assert_eq!("", form.locality);
    }
}
