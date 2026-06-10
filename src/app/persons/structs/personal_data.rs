use serde::{Deserialize, Serialize};

use crate::{
    AppStore, OptionAsStrExt,
    common::{
        BsnOrNoneConfirmed, CountryCode, DateOfBirth, Gender, PlaceOfResidence, PotentialProblems,
        Problematic,
    },
};

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct PersonalData {
    pub gender: Option<Gender>,

    pub bsn: Option<BsnOrNoneConfirmed>,
    pub date_of_birth: Option<DateOfBirth>,

    pub place_of_residence: Option<PlaceOfResidence>,
    pub country: Option<CountryCode>,
}

impl Problematic<()> for PersonalData {
    // This implementation will be merged with implementation below, once
    // `Problematic` gets the overhaul specified in
    // https://github.com/kiesraad/e-ks/issues/785
    fn get_problems(&self, _: ()) -> Vec<PotentialProblems> {
        let mut items = Vec::new();

        if self.bsn.is_none() {
            items.push(PotentialProblems::NoBsn);
        }

        match &self.place_of_residence {
            None => items.push(PotentialProblems::NoPlaceOfResidence),
            Some(p) if p.as_str().is_empty() => items.push(PotentialProblems::NoPlaceOfResidence),
            Some(PlaceOfResidence::Unknown(_)) => {
                items.push(PotentialProblems::UnknownPlaceOfResidence)
            }
            _ => {}
        }

        if self.country.is_empty_or_none() {
            items.push(PotentialProblems::NoCountryOfResidence);
        }

        match &self.date_of_birth {
            None => items.push(PotentialProblems::NoDateOfBirth),
            Some(dob) => {
                if dob.is_very_old() {
                    items.push(PotentialProblems::VeryOldDateOfBirth);
                }
            }
        }

        items
    }
}

impl Problematic<&AppStore> for PersonalData {
    // This implementation will be merged with implementation above, once
    // `Problematic` gets the overhaul specified in
    // https://github.com/kiesraad/e-ks/issues/785
    fn get_problems(&self, store: &AppStore) -> Vec<PotentialProblems> {
        let election = store.election;
        if let Some(date_of_birth) = self.date_of_birth.clone()
            && date_of_birth.is_too_young(&election)
        {
            vec![PotentialProblems::TooYoungDateOfBirth]
        } else {
            Vec::new()
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppStore;
    use chrono::{Duration, NaiveDate};

    fn complete_personal_data() -> PersonalData {
        PersonalData {
            gender: None,
            bsn: Some(BsnOrNoneConfirmed::NoneConfirmed),
            date_of_birth: Some("01-01-1990".parse().unwrap()),
            place_of_residence: Some("Amsterdam".parse().unwrap()),
            country: Some("NL".parse().unwrap()),
        }
    }

    #[test]
    fn complete_personal_data_has_no_problems() {
        assert!(complete_personal_data().get_problems(()).is_empty());
    }

    #[test]
    fn missing_bsn_produces_warning() {
        let mut data = complete_personal_data();
        data.bsn = None;
        assert!(data.get_problems(()).contains(&PotentialProblems::NoBsn));
    }

    #[test]
    fn missing_date_of_birth_produces_error() {
        let mut data = complete_personal_data();
        data.date_of_birth = None;
        assert!(
            data.get_problems(())
                .contains(&PotentialProblems::NoDateOfBirth)
        );
    }

    #[test]
    fn very_old_date_of_birth_produces_warning() {
        let mut data = complete_personal_data();
        data.date_of_birth = Some(NaiveDate::from_ymd_opt(1900, 1, 1).unwrap().into());
        assert!(
            data.get_problems(())
                .contains(&PotentialProblems::VeryOldDateOfBirth)
        );
    }

    #[test]
    fn too_young_date_of_birth_produces_warning() {
        let store = AppStore::new_for_test();
        let eligible_dob = store.election.eligible_date_of_birth();
        let mut data = complete_personal_data();
        data.date_of_birth = Some((eligible_dob + Duration::days(1)).into());
        assert!(
            data.get_problems(&store)
                .contains(&PotentialProblems::TooYoungDateOfBirth)
        );
    }

    #[test]
    fn recent_date_of_birth_does_not_warn() {
        let mut data = complete_personal_data();
        data.date_of_birth = Some("01-01-2000".parse().unwrap());
        assert!(
            !data
                .get_problems(())
                .contains(&PotentialProblems::VeryOldDateOfBirth)
        );
    }

    #[test]
    fn missing_place_of_residence_produces_error() {
        let mut data = complete_personal_data();
        data.place_of_residence = None;
        assert!(
            data.get_problems(())
                .contains(&PotentialProblems::NoPlaceOfResidence)
        );
    }

    #[test]
    fn missing_country_of_residence_produces_error() {
        let mut data = complete_personal_data();
        data.country = None;
        assert!(
            data.get_problems(())
                .contains(&PotentialProblems::NoCountryOfResidence)
        );
    }
}
