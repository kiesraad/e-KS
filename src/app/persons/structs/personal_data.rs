use serde::{Deserialize, Serialize};

use crate::common::{BsnOrNoneConfirmed, CountryCode, DateOfBirth, Gender, PlaceOfResidence};

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct PersonalData {
    pub gender: Option<Gender>,

    pub bsn: Option<BsnOrNoneConfirmed>,
    pub date_of_birth: Option<DateOfBirth>,

    pub place_of_residence: Option<PlaceOfResidence>,
    pub country: Option<CountryCode>,
}

impl PersonalData {
    pub fn locality(&self) -> Option<String> {
        match (&self.place_of_residence, &self.country) {
            (Some(place), Some(country)) if !country.is_nl() => {
                Some(format!("{} ({})", place, country))
            }
            (Some(place), _) => Some(place.to_string()),
            _ => None,
        }
    }
}
