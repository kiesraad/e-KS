use serde::{Deserialize, Serialize};

use crate::{
    ElectionConfig, OptionAsStrExt,
    common::{
        BrpVerification, BsnOrNoneConfirmed, CountryCode, DateOfBirth, Gender, InfoProblems,
        PlaceOfResidence, PotentialProblems, Problematic, Problems,
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

impl BrpVerification for PersonalData {
    async fn verify(&self) -> Result<bool, String> {
        let client = reqwest::Client::new();
        let resp = client
            .post("http://localhost:5010/haalcentraal/api/brp/personen")
            .header("Content-Type", "application/json")
            .json(
                r#"{
                "type": "RaadpleegMetBurgerservicenummer",
                "burgerservicenummer": ["100600505"],
                "fields": [
                    "burgerservicenummer",
     			"datumInschrijvingInGemeente",
     			"geboorte.datum",
     			"gemeenteVanInschrijving",
     			"geslacht",
     			"naam.adellijkeTitelPredicaat",
     			"naam.geslachtsnaam",
     			"naam.voornamen",
     			"naam.voorvoegsel",
     			"naam.aanduidingNaamgebruik",
     			"nationaliteiten.datumIngangGeldigheid",
     			"nationaliteiten.nationaliteit",
     			"overlijden.datum",
     			"partners.aangaanHuwelijkPartnerschap.datum",
     			"partners.naam.geslachtsnaam",
     			"partners.naam.voorvoegsel",
     			"partners.ontbindingHuwelijkPartnerschap",
     			"uitsluitingKiesrecht",
     			"verblijfplaats.verblijfadres.huisletter",
     			"verblijfplaats.verblijfadres.huisnummer",
     			"verblijfplaats.verblijfadres.huisnummertoevoeging",
     			"verblijfplaats.verblijfadres.officieleStraatnaam",
     			"verblijfplaats.verblijfadres.postcode",
     			"verblijfplaats.verblijfadres.woonplaats"
                ]
            }"#,
            )
            .send()
            .await
            .map_err(|_| String::from("Request failed"))?;

        // let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|_| String::from("Malformed response body"))?;

        dbg!(&text);

        let x = vec![1, 2];
        Ok(true)
    }
}

impl Problematic<ElectionConfig> for PersonalData {
    fn get_problems(&self, election: ElectionConfig) -> Problems {
        let mut potential_problems = Vec::new();
        let mut info_problems = Vec::new();

        if self.bsn.is_none() {
            potential_problems.push(PotentialProblems::NoBsn);
        }

        match &self.place_of_residence {
            None => potential_problems.push(PotentialProblems::NoPlaceOfResidence),
            Some(p) if p.as_str().is_empty() => {
                potential_problems.push(PotentialProblems::NoPlaceOfResidence)
            }
            Some(PlaceOfResidence::Unknown(_)) if self.lives_in_nl() => {
                potential_problems.push(PotentialProblems::UnknownPlaceOfResidence)
            }
            _ => {}
        }

        if self.country.is_empty_or_none() {
            potential_problems.push(PotentialProblems::NoCountryOfResidence);
        }

        match &self.date_of_birth {
            None => potential_problems.push(PotentialProblems::NoDateOfBirth),
            Some(dob) => {
                if dob.is_very_old() {
                    info_problems.push(InfoProblems::VeryOldDateOfBirth);
                }
                if dob.is_too_young(&election) {
                    potential_problems.push(PotentialProblems::TooYoungDateOfBirth);
                }
            }
        }

        Problems {
            potential_problems,
            info_problems,
        }
    }
}

impl PersonalData {
    pub fn lives_in_nl(&self) -> bool {
        self.country.as_ref().is_none_or(|country| country.is_nl())
    }

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
    use crate::PgStore;
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
        let problems = complete_personal_data().get_problems(ElectionConfig::EK27);
        assert!(problems.potential_problems.is_empty());
        assert!(problems.info_problems.is_empty());
    }

    #[test]
    fn missing_bsn_produces_warning() {
        let mut data = complete_personal_data();
        data.bsn = None;
        assert!(
            data.get_problems(ElectionConfig::EK27)
                .potential_problems
                .contains(&PotentialProblems::NoBsn)
        );
    }

    #[test]
    fn missing_date_of_birth_produces_error() {
        let mut data = complete_personal_data();
        data.date_of_birth = None;
        assert!(
            data.get_problems(ElectionConfig::EK27)
                .potential_problems
                .contains(&PotentialProblems::NoDateOfBirth)
        );
    }

    #[test]
    fn very_old_date_of_birth_produces_warning() {
        let mut data = complete_personal_data();
        data.date_of_birth = Some(NaiveDate::from_ymd_opt(1900, 1, 1).unwrap().into());
        assert!(
            data.get_problems(ElectionConfig::EK27)
                .info_problems
                .contains(&InfoProblems::VeryOldDateOfBirth)
        );
    }

    #[test]
    fn too_young_date_of_birth_produces_warning() {
        let store = PgStore::new_for_test();
        let eligible_dob = store.election.eligible_date_of_birth();
        let mut data = complete_personal_data();
        data.date_of_birth = Some((eligible_dob + Duration::days(1)).into());
        assert!(
            data.get_problems(ElectionConfig::EK27)
                .potential_problems
                .contains(&PotentialProblems::TooYoungDateOfBirth)
        );
    }

    #[test]
    fn recent_date_of_birth_does_not_warn() {
        let mut data = complete_personal_data();
        data.date_of_birth = Some("01-01-2000".parse().unwrap());
        assert!(
            !data
                .get_problems(ElectionConfig::EK27)
                .info_problems
                .contains(&InfoProblems::VeryOldDateOfBirth)
        );
    }

    #[test]
    fn missing_place_of_residence_produces_error() {
        let mut data = complete_personal_data();
        data.place_of_residence = None;
        assert!(
            data.get_problems(ElectionConfig::EK27)
                .potential_problems
                .contains(&PotentialProblems::NoPlaceOfResidence)
        );
    }

    #[test]
    fn unknown_place_of_residence_in_nl_produces_warning() {
        let mut data = complete_personal_data();
        data.place_of_residence = Some(PlaceOfResidence::Unknown("Faketown".to_string()));
        data.country = Some("NL".parse().unwrap());
        assert!(
            data.get_problems(ElectionConfig::EK27)
                .potential_problems
                .contains(&PotentialProblems::UnknownPlaceOfResidence)
        );
    }

    #[test]
    fn unknown_place_of_residence_outside_nl_does_not_warn() {
        let mut data = complete_personal_data();
        data.place_of_residence = Some(PlaceOfResidence::Unknown("Faketown".to_string()));
        data.country = Some("DE".parse().unwrap());
        assert!(
            !data
                .get_problems(ElectionConfig::EK27)
                .potential_problems
                .contains(&PotentialProblems::UnknownPlaceOfResidence)
        );
    }

    #[test]
    fn missing_country_of_residence_produces_error() {
        let mut data = complete_personal_data();
        data.country = None;
        assert!(
            data.get_problems(ElectionConfig::EK27)
                .potential_problems
                .contains(&PotentialProblems::NoCountryOfResidence)
        );
    }

    #[tokio::test]
    async fn brp() -> Result<(), String> {
        let data = PersonalData {
            // First person from the brp personen mock with a PlaceOfResidence
            gender: None,
            bsn: Some("100600505".parse().unwrap()),
            date_of_birth: Some("06-04-1975".parse().unwrap()),
            place_of_residence: Some("'s-Gravenhage".parse().unwrap()),
            country: Some("NL".parse().unwrap()),
        };
        let result = data.verify().await?;
        println!("{result}");
        panic!();
    }
}
