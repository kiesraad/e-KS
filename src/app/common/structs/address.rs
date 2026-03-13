use serde::{Deserialize, Serialize};

use crate::OptionAsStrExt;

use super::{
    CountryCode, HouseNumber, HouseNumberAddition, InternationalPostalCode, Locality, PostalCode,
    StateOrProvince, StreetName,
};

/// A Dutch postal address with optional components.
///
/// An address is considered complete when the street name, house number,
/// postal code, and locality are present.
#[derive(Default, Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
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
}

/// A postal address outside the Netherlands.
///
/// An address is considered complete when the street name, house number,
/// postal code, locality, and country are present.
#[derive(Default, Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct InternationalAddress {
    /// Street name (e.g. "Downing Street").
    pub street_name: Option<StreetName>,
    /// House number (e.g. "10").
    pub house_number: Option<HouseNumber>,
    /// House number addition.
    pub house_number_addition: Option<HouseNumberAddition>,
    /// City or town.
    pub locality: Option<Locality>,
    /// State, province, or region.
    pub state_or_province: Option<StateOrProvince>,
    /// International postal code with relaxed validation.
    pub postal_code: Option<InternationalPostalCode>,
    /// ISO 3166-1 alpha-2 country code.
    pub country: Option<CountryCode>,
}

impl InternationalAddress {
    /// Returns `true` when all required address parts are present.
    pub fn is_complete(&self) -> bool {
        self.street_name.is_some()
            && self.house_number.is_some()
            && self.postal_code.is_some()
            && self.locality.is_some()
            && self.country.is_some()
    }

    /// Returns `true` when all address parts are empty or `None`.
    pub fn is_empty(&self) -> bool {
        self.street_name.is_empty_or_none()
            && self.house_number.is_empty_or_none()
            && self.house_number_addition.is_empty_or_none()
            && self.postal_code.is_empty_or_none()
            && self.locality.is_empty_or_none()
            && self.state_or_province.is_empty_or_none()
            && self.country.is_empty_or_none()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Address {
    Dutch(DutchAddress),
    International(InternationalAddress),
}

impl Default for Address {
    fn default() -> Self {
        Address::Dutch(DutchAddress::default())
    }
}

impl Address {
    pub fn as_dutch(&self) -> Option<&DutchAddress> {
        match self {
            Address::Dutch(address) => Some(address),
            _ => None,
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            Address::Dutch(address) => address.is_complete(),
            Address::International(address) => address.is_complete(),
        }
    }

    pub fn postal_code(&self) -> Option<String> {
        match self {
            Address::Dutch(address) => address.postal_code.as_ref().map(ToString::to_string),
            Address::International(address) => {
                address.postal_code.as_ref().map(ToString::to_string)
            }
        }
    }

    pub fn locality(&self) -> Option<String> {
        match self {
            Address::Dutch(address) => address.locality.as_ref().map(ToString::to_string),
            Address::International(address) => address.locality.as_ref().map(ToString::to_string),
        }
    }

    pub fn country(&self) -> Option<String> {
        match self {
            Address::Dutch(_) => None,
            Address::International(address) => address.country.as_ref().map(ToString::to_string),
        }
    }

    pub fn state_or_province(&self) -> Option<String> {
        match self {
            Address::Dutch(_) => None,
            Address::International(address) => {
                address.state_or_province.as_ref().map(ToString::to_string)
            }
        }
    }

    /// The street name, house number, and house number addition, e.g. "Main Street 10A".
    ///
    /// Returns `None` if the street name or house number are `None`.
    pub fn address_line_1(&self) -> Option<String> {
        let (street_name, house_number, house_number_addition) = match self {
            Address::Dutch(address) => (
                address.street_name.as_ref(),
                address.house_number.as_ref(),
                address.house_number_addition.as_ref(),
            ),
            Address::International(address) => (
                address.street_name.as_ref(),
                address.house_number.as_ref(),
                address.house_number_addition.as_ref(),
            ),
        };

        match (street_name, house_number, house_number_addition) {
            (Some(street_name), Some(house_number), Some(house_number_addition)) => Some(format!(
                "{street_name} {house_number}{house_number_addition}"
            )),
            (Some(street_name), Some(house_number), None) => {
                Some(format!("{street_name} {house_number}"))
            }
            _ => None,
        }
    }

    /// The locality, postal code, state/province, and country.
    ///
    /// Returns `None` if the locality or postal code are `None`.
    pub fn address_line_2(&self) -> Option<String> {
        match (
            self.locality(),
            self.postal_code(),
            self.state_or_province(),
            self.country(),
        ) {
            (Some(locality), Some(postal_code), Some(state_or_province), Some(country)) => Some(
                format!("{postal_code} {locality}, {state_or_province} ({country})"),
            ),
            (Some(locality), Some(postal_code), Some(state_or_province), None) => {
                Some(format!("{postal_code} {locality}, {state_or_province}"))
            }
            (Some(locality), Some(postal_code), None, Some(country)) => {
                Some(format!("{postal_code} {locality} ({country})"))
            }
            (Some(locality), Some(postal_code), None, None) => {
                Some(format!("{postal_code} {locality}"))
            }
            _ => None,
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
            Address::Dutch(address).address_line_1()
        );
    }

    #[test]
    fn address_line_1_without_addition() {
        let mut address = sample_address();
        address.house_number_addition = None;

        assert_eq!(
            Some("Hoofdstraat 123".to_string()),
            Address::Dutch(address).address_line_1()
        );
    }

    #[test]
    fn address_line_1_requires_street_and_house_number() {
        let mut address = sample_address();
        address.street_name = None;

        assert_eq!(None, Address::Dutch(address).address_line_1());

        address = sample_address();
        address.house_number = None;

        assert_eq!(None, Address::Dutch(address).address_line_1());
    }

    fn sample_international_address() -> InternationalAddress {
        InternationalAddress {
            street_name: Some(StreetName::from_str("Downing Street").unwrap()),
            house_number: Some(HouseNumber::from_str("10").unwrap()),
            house_number_addition: None,
            locality: Some(Locality::from_str("London").unwrap()),
            state_or_province: Some(StateOrProvince::from_str("Greater London").unwrap()),
            postal_code: Some(InternationalPostalCode::from_str("SW1A 2AA").unwrap()),
            country: Some(CountryCode::from_str("GB").unwrap()),
        }
    }

    #[test]
    fn international_address_is_complete_when_required_parts_present() {
        assert!(sample_international_address().is_complete());
    }

    #[test]
    fn international_address_requires_country_for_completeness() {
        let mut address = sample_international_address();
        address.country = None;

        assert!(!address.is_complete());
    }

    #[test]
    fn international_address_is_empty_when_all_parts_absent_or_empty() {
        let address = InternationalAddress {
            street_name: Some(StreetName::default()),
            house_number: Some(HouseNumber::default()),
            house_number_addition: Some(HouseNumberAddition::default()),
            locality: Some(Locality::default()),
            state_or_province: Some(StateOrProvince::default()),
            postal_code: Some(InternationalPostalCode::default()),
            country: Some(CountryCode::default()),
        };

        assert!(address.is_empty());
    }

    #[test]
    fn international_address_line_2_includes_state_or_province_when_present() {
        assert_eq!(
            Some("SW1A 2AA London, Greater London (GB)".to_string()),
            Address::International(sample_international_address()).address_line_2()
        );
    }
}
