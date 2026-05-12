use serde::{Deserialize, Serialize};

use crate::{
    OptionAsStrExt,
    common::{BsnOrNoneConfirmed, CountryCode, DateOfBirth, Gender, PlaceOfResidence},
    submit::{PotentialProblems, Problematic},
};

/// The age at which we will start warning that the date of birth might be incorrect
pub const CANDIDATE_WARN_AGE: u32 = 110;

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

        match &self.date_of_birth {
            None => items.push(PotentialProblems::NoDateOfBirth),
            Some(dob) => {
                if chrono::Utc::now()
                    .date_naive()
                    .years_since(**dob)
                    .is_some_and(|y| y >= CANDIDATE_WARN_AGE)
                {
                    items.push(PotentialProblems::VeryOldDateOfBirth);
                }
            }
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
