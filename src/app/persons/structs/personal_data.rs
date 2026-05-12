use serde::{Deserialize, Serialize};

use crate::{
    OptionAsStrExt,
    common::{BsnOrNoneConfirmed, CountryCode, DateOfBirth, Gender, PlaceOfResidence},
    submit::{PotentialProblems, Problematic},
};

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct PersonalData {
    pub gender: Option<Gender>,

    pub bsn: Option<BsnOrNoneConfirmed>,
    pub date_of_birth: Option<DateOfBirth>,

    pub place_of_residence: Option<PlaceOfResidence>,
    pub country: Option<CountryCode>,
}

impl Problematic for PersonalData {
    fn get_problems(&self) -> Vec<PotentialProblems> {
        let mut items = Vec::new();

        if self.bsn.is_none() {
            items.push(PotentialProblems::NoBsn);
        }

        if self.place_of_residence.is_empty_or_none() {
            items.push(PotentialProblems::NoPlaceOfResidence);
        }

        if self.country.is_empty_or_none() {
            items.push(PotentialProblems::NoCountryOfResidence);
        }

        if self.date_of_birth.is_none() {
            items.push(PotentialProblems::NoDateOfBirth);
        }

        items
    }
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
