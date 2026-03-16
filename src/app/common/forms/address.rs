use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    OptionStringExt,
    common::{
        CountryCode, DutchAddress, HouseNumber, HouseNumberAddition, InternationalAddress,
        InternationalPostalCode, Locality, PostalCode, StateOrProvince, StreetName,
    },
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

/// Form-friendly representation of an international address.
#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "InternationalAddress")]
#[serde(default)]
pub struct InternationalAddressForm {
    #[validate(parse = "CountryCode", optional)]
    pub country: String,
    #[validate(parse = "Locality", optional)]
    pub locality: String,
    #[validate(parse = "StateOrProvince", optional)]
    pub state_or_province: String,
    #[validate(parse = "InternationalPostalCode", optional)]
    pub postal_code: String,
    #[validate(parse = "HouseNumber", optional)]
    pub house_number: String,
    #[validate(parse = "HouseNumberAddition", optional)]
    pub house_number_addition: String,
    #[validate(parse = "StreetName", optional)]
    pub street_name: String,
}

impl From<InternationalAddress> for InternationalAddressForm {
    fn from(address: InternationalAddress) -> Self {
        InternationalAddressForm {
            country: address.country.to_string_or_default(),
            locality: address.locality.to_string_or_default(),
            state_or_province: address.state_or_province.to_string_or_default(),
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

    #[test]
    fn international_form_from_address_uses_empty_strings_for_missing_parts() {
        let address = InternationalAddress::default();
        let form = InternationalAddressForm::from(address);

        assert_eq!("", form.country);
        assert_eq!("", form.street_name);
        assert_eq!("", form.house_number);
        assert_eq!("", form.house_number_addition);
        assert_eq!("", form.postal_code);
        assert_eq!("", form.locality);
        assert_eq!("", form.state_or_province);
    }
}
