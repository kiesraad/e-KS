use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::OptionStringExt;

use super::{HouseNumber, HouseNumberAddition, Locality, PostalCode, StreetName};

/// A Dutch postal address with optional components.
///
/// An address is considered complete when the street name, house number,
/// postal code, and locality are present.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DutchAddress {
    /// Street name (e.g. "Hoofdstraat").
    pub street_name: Option<StreetName>,
    /// House number (e.g. "123").
    pub house_number: Option<HouseNumber>,
    /// House number addition (e.g. "A" or "A1A").
    pub house_number_addition: Option<HouseNumberAddition>,
    /// City or town.
    pub locality: Option<Locality>,
    /// Dutch postal code.
    pub postal_code: Option<PostalCode>,
}

impl DutchAddress {
    /// Returns `true` when all required address parts are present.
    pub fn is_complete(&self) -> bool {
        self.street_name.is_some()
            && self.house_number.is_some()
            && self.postal_code.is_some()
            && self.locality.is_some()
    }

    /// Returns `true` when all address parts are empty or `None`.
    pub fn is_empty(&self) -> bool {
        self.street_name.is_empty_or_none()
            && self.house_number.is_empty_or_none()
            && self.house_number_addition.is_empty_or_none()
            && self.postal_code.is_empty_or_none()
            && self.locality.is_empty_or_none()
    }

    /// The street name, house number, and house number addition, e.g. "Hoofdstraat 123a".
    ///
    /// Returns `None` if the street name or house number are `None`.
    pub fn address_line_1(&self) -> Option<String> {
        match (
            self.street_name.as_ref(),
            self.house_number.as_ref(),
            self.house_number_addition.as_ref(),
        ) {
            (Some(street_name), Some(house_number), Some(house_number_addition)) => Some(format!(
                "{} {}{}",
                street_name, house_number, house_number_addition
            )),
            (Some(street_name), Some(house_number), None) => {
                Some(format!("{} {}", street_name, house_number))
            }
            _ => None,
        }
    }
}

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
    use std::str::FromStr;

    fn sample_address() -> DutchAddress {
        DutchAddress {
            street_name: Some(StreetName::from_str("Hoofdstraat").unwrap()),
            house_number: Some(HouseNumber::from_str("123").unwrap()),
            house_number_addition: Some(HouseNumberAddition::from_str("a").unwrap()),
            locality: Some(Locality::from_str("Amsterdam").unwrap()),
            postal_code: Some(PostalCode::from_str("1234 AB").unwrap()),
        }
    }

    #[test]
    fn is_complete_when_required_parts_present() {
        let mut address = sample_address();
        address.house_number_addition = None;

        assert!(address.is_complete());
    }

    #[test]
    fn is_not_complete_when_any_required_part_missing() {
        let mut address = sample_address();
        address.locality = None;

        assert!(!address.is_complete());
    }

    #[test]
    fn is_empty_when_all_parts_absent_or_empty() {
        let address = DutchAddress {
            street_name: Some(StreetName::default()),
            house_number: Some(HouseNumber::default()),
            house_number_addition: Some(HouseNumberAddition::default()),
            locality: Some(Locality::default()),
            postal_code: Some(PostalCode::default()),
        };

        assert!(address.is_empty());
    }

    #[test]
    fn is_not_empty_when_any_part_has_value() {
        let address = DutchAddress {
            street_name: Some(StreetName::from_str("Kerkstraat").unwrap()),
            ..DutchAddress::default()
        };

        assert!(!address.is_empty());
    }

    #[test]
    fn address_line_1_with_addition() {
        let address = sample_address();

        assert_eq!(
            Some("Hoofdstraat 123a".to_string()),
            address.address_line_1()
        );
    }

    #[test]
    fn address_line_1_without_addition() {
        let mut address = sample_address();
        address.house_number_addition = None;

        assert_eq!(
            Some("Hoofdstraat 123".to_string()),
            address.address_line_1()
        );
    }

    #[test]
    fn address_line_1_requires_street_and_house_number() {
        let mut address = sample_address();
        address.street_name = None;

        assert_eq!(None, address.address_line_1());

        address = sample_address();
        address.house_number = None;

        assert_eq!(None, address.address_line_1());
    }

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
