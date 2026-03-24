use serde::{Deserialize, Serialize};

use crate::{
    OptionStringExt,
    common::{BsnOrNoneConfirmed, DutchAddress, FullName},
    constants::DEFAULT_DATE_FORMAT,
    core::AnyLocale,
    persons::Person,
};

const NO_BSN: &str = "kandidaat heeft geen BSN";

#[derive(Debug, Serialize, Deserialize)]
pub struct CandidateRecord {
    voorletters: String,
    roepnaam: String,
    voorvoegsel: String,
    achternaam: String,

    woonplaats: String,
    landcode: String,
    bsn: String,
    geboortedatum: String,
    geslacht: String,

    gemachtigde_voorletters: String,
    gemachtigde_roepnaam: String,
    gemachtigde_voorvoegsel: String,
    gemachtigde_achternaam: String,

    // these fields are either the correspondence or the authorised person address
    postcode: String,
    huisnummer: String,
    toevoeging: String,
    straatnaam: String,
    plaats: String,
}

impl From<Person> for CandidateRecord {
    fn from(person: Person) -> Self {
        let candidate_name = person.name;
        let candidate_personal_data = person.personal_data;

        let authorised_name = match &candidate_personal_data.country {
            Some(country) if !country.is_nl() => person.representative.name,
            _ => FullName::default(),
        };

        let address = match &candidate_personal_data.country {
            Some(country) if country.is_nl() => person.address,
            Some(_) => person.representative.address,
            None => DutchAddress {
                street_name: None,
                house_number: None,
                house_number_addition: None,
                locality: None,
                postal_code: None,
            },
        };

        Self {
            voorletters: candidate_name.initials.to_string(),
            roepnaam: candidate_name.first_name.to_string_or_default(),
            voorvoegsel: candidate_name.last_name_prefix.to_string_or_default(),
            achternaam: candidate_name.last_name.to_string(),

            woonplaats: candidate_personal_data
                .place_of_residence
                .to_string_or_default(),
            landcode: candidate_personal_data.country.to_string_or_default(),
            bsn: match candidate_personal_data.bsn {
                Some(BsnOrNoneConfirmed::NoneConfirmed) => NO_BSN.to_string(),
                Some(BsnOrNoneConfirmed::Bsn(bsn)) => bsn.to_exposed_string(),
                None => "".to_string(),
            },
            geboortedatum: candidate_personal_data
                .date_of_birth
                .map(|d| d.format(DEFAULT_DATE_FORMAT))
                .to_string_or_default(),
            geslacht: match candidate_personal_data.gender {
                Some(gender) => gender.abbreviation(AnyLocale::Nl).to_string(),
                None => "".to_string(),
            },

            gemachtigde_voorletters: authorised_name.initials.to_string(),
            gemachtigde_roepnaam: authorised_name.first_name.to_string_or_default(),
            gemachtigde_voorvoegsel: authorised_name.last_name_prefix.to_string_or_default(),
            gemachtigde_achternaam: authorised_name.last_name.to_string(),

            postcode: address.postal_code.to_string_or_default(),
            huisnummer: address.house_number.to_string_or_default(),
            toevoeging: address.house_number_addition.to_string_or_default(),
            straatnaam: address.street_name.to_string_or_default(),
            plaats: address.locality.to_string_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::NaiveDate;

    use crate::{
        common::{
            CountryCode, DateOfBirth, FirstName, FullName, Gender, HouseNumber,
            HouseNumberAddition, Initials, LastName, LastNamePrefix, Locality, PlaceOfResidence,
            PostalCode, StreetName, UtcDateTime,
        },
        persons::{PersonId, PersonalData, Representative},
    };

    use super::*;

    /// alternative to sample person.
    /// 1. It's a person with everything filled in
    /// 2. Defined here because it has semantic significance for the tests
    fn test_person() -> Person {
        Person {
            id: PersonId::new(),
            name: FullName {
                first_name: FirstName::from_str("Jan").ok(),
                last_name: LastName::from_str("Berg").unwrap(),
                last_name_prefix: LastNamePrefix::from_str("van de").ok(),
                initials: Initials::from_str("J.").unwrap(),
            },
            personal_data: PersonalData {
                gender: Some(Gender::Male),
                bsn: BsnOrNoneConfirmed::from_str("999994335").ok(),
                date_of_birth: Some(DateOfBirth::from(
                    NaiveDate::from_ymd_opt(2000, 10, 20).unwrap(),
                )),
                place_of_residence: PlaceOfResidence::from_str("Amsterdam").ok(),
                country: CountryCode::from_str("NL").ok(),
            },
            address: DutchAddress {
                street_name: StreetName::from_str("Mooie Straat").ok(),
                house_number: HouseNumber::from_str("12").ok(),
                house_number_addition: HouseNumberAddition::from_str("a").ok(),
                locality: Locality::from_str("Rotterdam").ok(),
                postal_code: PostalCode::from_str("1234AB").ok(),
            },
            representative: Representative {
                name: FullName {
                    first_name: FirstName::from_str("Pietje").ok(),
                    last_name: LastName::from_str("Puk").unwrap(),
                    last_name_prefix: None,
                    initials: Initials::from_str("P.").unwrap(),
                },
                address: DutchAddress {
                    street_name: StreetName::from_str("Mooiere Straat").ok(),
                    house_number: HouseNumber::from_str("34").ok(),
                    house_number_addition: HouseNumberAddition::from_str("b").ok(),
                    locality: Locality::from_str("Den Haag").ok(),
                    postal_code: PostalCode::from_str("5678CD").ok(),
                },
            },
            updated_at: UtcDateTime::now(),
        }
    }

    #[test]
    fn candidate_into_record() {
        let person = test_person();

        let record: CandidateRecord = person.clone().into();

        assert_eq!(record.voorletters, "J.");
        assert_eq!(record.roepnaam, "Jan");
        assert_eq!(record.voorvoegsel, "van de");
        assert_eq!(record.achternaam, "Berg");
        assert_eq!(record.woonplaats, "Amsterdam");
        assert_eq!(record.landcode, "NL");
        assert_eq!(record.bsn, "999994335");
        assert_eq!(record.geboortedatum, "20-10-2000");
        assert_eq!(record.geslacht, "m");

        // despite having a representative filled in, because country == NL, it should be empty
        assert_eq!(record.gemachtigde_voorletters, "");
        assert_eq!(record.gemachtigde_roepnaam, "");
        assert_eq!(record.gemachtigde_voorvoegsel, "");
        assert_eq!(record.gemachtigde_achternaam, "");

        // the correspondent fields instead of authorised person fields
        assert_eq!(record.postcode, "1234AB");
        assert_eq!(record.huisnummer, "12");
        assert_eq!(record.toevoeging, "a");
        assert_eq!(record.straatnaam, "Mooie Straat");
        assert_eq!(record.plaats, "Rotterdam");
    }

    #[test]
    fn candidate_into_record_no_bsn() {
        let mut no_bsn = test_person();
        no_bsn.personal_data.bsn = None;
        let mut no_bsn_confirmed = test_person();
        no_bsn_confirmed.personal_data.bsn = Some(BsnOrNoneConfirmed::NoneConfirmed);
        let mut bsn = test_person();
        bsn.personal_data.bsn = BsnOrNoneConfirmed::from_str("999994335").ok();

        let no_bsn_record: CandidateRecord = no_bsn.into();
        let no_bsn_confirmed_record: CandidateRecord = no_bsn_confirmed.into();
        let bsn_record: CandidateRecord = bsn.into();

        assert_eq!(no_bsn_record.bsn, "");
        assert_eq!(no_bsn_confirmed_record.bsn, "kandidaat heeft geen BSN");
        assert_eq!(bsn_record.bsn, "999994335");
    }

    #[test]
    fn candidate_into_record_genders() {
        let mut male = test_person();
        male.personal_data.gender = Some(Gender::Male);
        let mut female = test_person();
        female.personal_data.gender = Some(Gender::Female);
        let mut x = test_person();
        x.personal_data.gender = None;

        let male_record: CandidateRecord = male.into();
        let female_record: CandidateRecord = female.into();
        let x_record: CandidateRecord = x.into();

        assert_eq!(male_record.geslacht, "m");
        assert_eq!(female_record.geslacht, "v");
        assert_eq!(x_record.geslacht, "");
    }

    #[test]
    fn candidate_into_record_with_authorised_person() {
        let mut person = test_person();
        person.personal_data.country = CountryCode::from_str("BE").ok();

        let record: CandidateRecord = person.into();

        // because country != NL, authorised person info should be present
        assert_eq!(record.gemachtigde_voorletters, "P.");
        assert_eq!(record.gemachtigde_roepnaam, "Pietje");
        assert_eq!(record.gemachtigde_voorvoegsel, "");
        assert_eq!(record.gemachtigde_achternaam, "Puk");

        // the correspondent fields instead of authorised person fields
        assert_eq!(record.postcode, "5678CD");
        assert_eq!(record.huisnummer, "34");
        assert_eq!(record.toevoeging, "b");
        assert_eq!(record.straatnaam, "Mooiere Straat");
        assert_eq!(record.plaats, "Den Haag");
    }
}
