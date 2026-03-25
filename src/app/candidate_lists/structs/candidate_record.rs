use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    CsrfToken, CsrfTokens, OptionStringExt,
    common::{BsnOrNoneConfirmed, DutchAddressForm, FullNameForm},
    constants::DEFAULT_DATE_FORMAT,
    core::AnyLocale,
    form::{FieldErrors, FormData, WithCsrfToken},
    persons::{Person, PersonalDataFieldsForm, Representative},
};

const NO_BSN: &str = "persoon heeft geen BSN";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(from = "CandidateRecordCsv", into = "CandidateRecordCsv")]
pub struct CandidateRecord {
    name: FullNameForm,
    personal_data: PersonalDataFieldsForm,
    address: DutchAddressForm,
    representative: RepresentativeRecord,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "Representative")]
#[serde(default)]
pub struct RepresentativeRecord {
    #[validate(flatten)]
    #[serde(flatten)]
    pub name: FullNameForm,
    #[validate(flatten)]
    #[serde(flatten)]
    pub address: DutchAddressForm,
}

impl From<Representative> for RepresentativeRecord {
    fn from(person: Representative) -> Self {
        RepresentativeRecord {
            name: FullNameForm::from(person.name),
            address: DutchAddressForm::from(person.address),
        }
    }
}

impl RepresentativeRecord {
    fn is_empty(&self) -> bool {
        [
            &self.name.first_name,
            &self.name.last_name,
            &self.name.last_name_prefix,
            &self.name.initials,
            &self.address.locality,
            &self.address.postal_code,
            &self.address.house_number,
            &self.address.house_number_addition,
            &self.address.street_name,
        ]
        .into_iter()
        .all(|value| value.trim().is_empty())
    }
}

impl WithCsrfToken for CandidateRecord {
    fn with_csrf_token(self, _csrf_token: CsrfToken) -> Self {
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
struct CandidateRecordCsv {
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

    correspondentie_postcode: String,
    correspondentie_huisnummer: String,
    correspondentie_toevoeging: String,
    correspondentie_straatnaam: String,
    correspondentie_plaats: String,

    gemachtigde_postcode: String,
    gemachtigde_huisnummer: String,
    gemachtigde_toevoeging: String,
    gemachtigde_straatnaam: String,
    gemachtigde_plaats: String,
}

impl From<CandidateRecordCsv> for CandidateRecord {
    fn from(csv: CandidateRecordCsv) -> Self {
        CandidateRecord {
            name: FullNameForm {
                first_name: csv.roepnaam,
                last_name: csv.achternaam,
                last_name_prefix: csv.voorvoegsel,
                initials: csv.voorletters,
            },
            personal_data: PersonalDataFieldsForm {
                gender: gender_from_csv(&csv.geslacht),
                date_of_birth: csv.geboortedatum,
                bsn: bsn_from_csv(&csv.bsn),
                place_of_residence: csv.woonplaats,
                country: csv.landcode,
            },
            address: DutchAddressForm {
                locality: csv.correspondentie_plaats,
                postal_code: csv.correspondentie_postcode,
                house_number: csv.correspondentie_huisnummer,
                house_number_addition: csv.correspondentie_toevoeging,
                street_name: csv.correspondentie_straatnaam,
            },
            representative: RepresentativeRecord {
                name: FullNameForm {
                    first_name: csv.gemachtigde_roepnaam,
                    last_name: csv.gemachtigde_achternaam,
                    last_name_prefix: csv.gemachtigde_voorvoegsel,
                    initials: csv.gemachtigde_voorletters,
                },
                address: DutchAddressForm {
                    locality: csv.gemachtigde_plaats,
                    postal_code: csv.gemachtigde_postcode,
                    house_number: csv.gemachtigde_huisnummer,
                    house_number_addition: csv.gemachtigde_toevoeging,
                    street_name: csv.gemachtigde_straatnaam,
                },
            },
        }
    }
}

impl From<CandidateRecord> for CandidateRecordCsv {
    fn from(record: CandidateRecord) -> Self {
        CandidateRecordCsv {
            voorletters: record.name.initials,
            roepnaam: record.name.first_name,
            voorvoegsel: record.name.last_name_prefix,
            achternaam: record.name.last_name,

            woonplaats: record.personal_data.place_of_residence,
            landcode: record.personal_data.country,
            bsn: bsn_to_csv(&record.personal_data.bsn),
            geboortedatum: record.personal_data.date_of_birth,
            geslacht: gender_to_csv(&record.personal_data.gender),

            gemachtigde_voorletters: record.representative.name.initials,
            gemachtigde_roepnaam: record.representative.name.first_name,
            gemachtigde_voorvoegsel: record.representative.name.last_name_prefix,
            gemachtigde_achternaam: record.representative.name.last_name,

            correspondentie_postcode: record.address.postal_code,
            correspondentie_huisnummer: record.address.house_number,
            correspondentie_toevoeging: record.address.house_number_addition,
            correspondentie_straatnaam: record.address.street_name,
            correspondentie_plaats: record.address.locality,

            gemachtigde_postcode: record.representative.address.postal_code,
            gemachtigde_huisnummer: record.representative.address.house_number,
            gemachtigde_toevoeging: record.representative.address.house_number_addition,
            gemachtigde_straatnaam: record.representative.address.street_name,
            gemachtigde_plaats: record.representative.address.locality,
        }
    }
}

impl From<Person> for CandidateRecord {
    fn from(person: Person) -> Self {
        let candidate_name = person.name;
        let candidate_personal_data = person.personal_data;

        let representative = person.representative;
        let address = person.address;

        Self::from(CandidateRecordCsv {
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

            gemachtigde_voorletters: representative.name.initials.to_string(),
            gemachtigde_roepnaam: representative.name.first_name.to_string_or_default(),
            gemachtigde_voorvoegsel: representative.name.last_name_prefix.to_string_or_default(),
            gemachtigde_achternaam: representative.name.last_name.to_string(),

            correspondentie_postcode: address.postal_code.to_string_or_default(),
            correspondentie_huisnummer: address.house_number.to_string_or_default(),
            correspondentie_toevoeging: address.house_number_addition.to_string_or_default(),
            correspondentie_straatnaam: address.street_name.to_string_or_default(),
            correspondentie_plaats: address.locality.to_string_or_default(),

            gemachtigde_postcode: representative.address.postal_code.to_string_or_default(),
            gemachtigde_huisnummer: representative.address.house_number.to_string_or_default(),
            gemachtigde_toevoeging: representative
                .address
                .house_number_addition
                .to_string_or_default(),
            gemachtigde_straatnaam: representative.address.street_name.to_string_or_default(),
            gemachtigde_plaats: representative.address.locality.to_string_or_default(),
        })
    }
}

impl CandidateRecord {
    pub fn validate_create(self, csrf_tokens: &CsrfTokens) -> Result<Person, FormData<Self>> {
        let mut errors = Vec::new();

        let name = collect_validation_result(
            "name",
            self.name.clone().validate_create(csrf_tokens),
            &mut errors,
        );
        let personal_data = collect_validation_result(
            "personal_data",
            self.personal_data.clone().validate_create(csrf_tokens),
            &mut errors,
        );
        let address = collect_validation_result(
            "address",
            self.address.clone().validate_create(csrf_tokens),
            &mut errors,
        );
        let representative = if self.representative.is_empty() {
            Some(Representative::default())
        } else {
            collect_validation_result(
                "representative",
                self.representative.clone().validate_create(csrf_tokens),
                &mut errors,
            )
        };

        if !errors.is_empty() {
            return Err(FormData::new_with_errors(self, csrf_tokens, errors));
        }

        Ok(Person {
            name: name.expect("validated field"),
            personal_data: personal_data.expect("validated field"),
            address: address.expect("validated field"),
            representative: representative.expect("validated field"),
            ..Default::default()
        })
    }
}

fn collect_validation_result<T, F>(
    prefix: &str,
    result: Result<T, FormData<F>>,
    errors: &mut FieldErrors,
) -> Option<T>
where
    F: WithCsrfToken,
{
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            errors.extend(
                error
                    .errors()
                    .into_iter()
                    .map(|(field_name, error)| (format!("{prefix}.{field_name}"), error)),
            );
            None
        }
    }
}

fn bsn_from_csv(value: &str) -> String {
    let value = value.trim();
    if value == NO_BSN {
        "none-confirmed".to_string()
    } else {
        value.to_string()
    }
}

fn bsn_to_csv(value: &str) -> String {
    if value.trim() == "none-confirmed" {
        NO_BSN.to_string()
    } else {
        value.to_string()
    }
}

fn gender_from_csv(value: &str) -> String {
    match value.trim() {
        "m" => "male".to_string(),
        "v" => "female".to_string(),
        value => value.to_string(),
    }
}

fn gender_to_csv(value: &str) -> String {
    match value.trim() {
        "male" => "m".to_string(),
        "female" => "v".to_string(),
        value => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::NaiveDate;

    use crate::{
        CsrfTokens,
        common::{
            CountryCode, Date, DutchAddress, FirstName, FullName, Gender, HouseNumber,
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
                date_of_birth: Some(Date::from(NaiveDate::from_ymd_opt(2000, 10, 20).unwrap())),
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
        let csv: CandidateRecordCsv = record.into();

        assert_eq!(csv.voorletters, "J.");
        assert_eq!(csv.roepnaam, "Jan");
        assert_eq!(csv.voorvoegsel, "van de");
        assert_eq!(csv.achternaam, "Berg");
        assert_eq!(csv.woonplaats, "Amsterdam");
        assert_eq!(csv.landcode, "NL");
        assert_eq!(csv.bsn, "999994335");
        assert_eq!(csv.geboortedatum, "20-10-2000");
        assert_eq!(csv.geslacht, "m");

        assert_eq!(csv.gemachtigde_voorletters, "P.");
        assert_eq!(csv.gemachtigde_roepnaam, "Pietje");
        assert_eq!(csv.gemachtigde_voorvoegsel, "");
        assert_eq!(csv.gemachtigde_achternaam, "Puk");

        assert_eq!(csv.correspondentie_postcode, "1234AB");
        assert_eq!(csv.correspondentie_huisnummer, "12");
        assert_eq!(csv.correspondentie_toevoeging, "a");
        assert_eq!(csv.correspondentie_straatnaam, "Mooie Straat");
        assert_eq!(csv.correspondentie_plaats, "Rotterdam");
        assert_eq!(csv.gemachtigde_postcode, "5678CD");
        assert_eq!(csv.gemachtigde_huisnummer, "34");
        assert_eq!(csv.gemachtigde_toevoeging, "b");
        assert_eq!(csv.gemachtigde_straatnaam, "Mooiere Straat");
        assert_eq!(csv.gemachtigde_plaats, "Den Haag");
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
        let no_bsn_csv: CandidateRecordCsv = no_bsn_record.into();
        let no_bsn_confirmed_csv: CandidateRecordCsv = no_bsn_confirmed_record.into();
        let bsn_csv: CandidateRecordCsv = bsn_record.into();

        assert_eq!(no_bsn_csv.bsn, "");
        assert_eq!(no_bsn_confirmed_csv.bsn, "kandidaat heeft geen BSN");
        assert_eq!(bsn_csv.bsn, "999994335");
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
        let male_csv: CandidateRecordCsv = male_record.into();
        let female_csv: CandidateRecordCsv = female_record.into();
        let x_csv: CandidateRecordCsv = x_record.into();

        assert_eq!(male_csv.geslacht, "m");
        assert_eq!(female_csv.geslacht, "v");
        assert_eq!(x_csv.geslacht, "");
    }

    #[test]
    fn candidate_into_record_with_authorised_person() {
        let mut person = test_person();
        person.personal_data.country = CountryCode::from_str("BE").ok();

        let record: CandidateRecord = person.into();
        let csv: CandidateRecordCsv = record.into();

        assert_eq!(csv.gemachtigde_voorletters, "P.");
        assert_eq!(csv.gemachtigde_roepnaam, "Pietje");
        assert_eq!(csv.gemachtigde_voorvoegsel, "");
        assert_eq!(csv.gemachtigde_achternaam, "Puk");

        assert_eq!(csv.correspondentie_postcode, "1234AB");
        assert_eq!(csv.correspondentie_huisnummer, "12");
        assert_eq!(csv.correspondentie_toevoeging, "a");
        assert_eq!(csv.correspondentie_straatnaam, "Mooie Straat");
        assert_eq!(csv.correspondentie_plaats, "Rotterdam");
        assert_eq!(csv.gemachtigde_postcode, "5678CD");
        assert_eq!(csv.gemachtigde_huisnummer, "34");
        assert_eq!(csv.gemachtigde_toevoeging, "b");
        assert_eq!(csv.gemachtigde_straatnaam, "Mooiere Straat");
        assert_eq!(csv.gemachtigde_plaats, "Den Haag");
    }

    #[test]
    fn validate_create_maps_correspondence_address_to_person_address() {
        let record = CandidateRecord::from(CandidateRecordCsv {
            voorletters: "J.".to_string(),
            roepnaam: "Jan".to_string(),
            voorvoegsel: "van de".to_string(),
            achternaam: "Berg".to_string(),
            woonplaats: "Amsterdam".to_string(),
            landcode: "NL".to_string(),
            bsn: NO_BSN.to_string(),
            geboortedatum: "20-10-2000".to_string(),
            geslacht: "m".to_string(),
            gemachtigde_voorletters: String::new(),
            gemachtigde_roepnaam: String::new(),
            gemachtigde_voorvoegsel: String::new(),
            gemachtigde_achternaam: String::new(),
            correspondentie_postcode: "1234AB".to_string(),
            correspondentie_huisnummer: "12".to_string(),
            correspondentie_toevoeging: "a".to_string(),
            correspondentie_straatnaam: "Mooie Straat".to_string(),
            correspondentie_plaats: "Rotterdam".to_string(),
            gemachtigde_postcode: String::new(),
            gemachtigde_huisnummer: String::new(),
            gemachtigde_toevoeging: String::new(),
            gemachtigde_straatnaam: String::new(),
            gemachtigde_plaats: String::new(),
        });

        let person = record.validate_create(&CsrfTokens::default()).unwrap();

        assert_eq!(person.name.initials.to_string(), "J.");
        assert_eq!(person.name.first_name.unwrap().to_string(), "Jan");
        assert_eq!(person.name.last_name_prefix.unwrap().to_string(), "van de");
        assert_eq!(person.name.last_name.to_string(), "Berg");
        assert_eq!(person.personal_data.gender, Some(Gender::Male));
        assert_eq!(
            person.personal_data.bsn,
            Some(BsnOrNoneConfirmed::NoneConfirmed)
        );
        assert_eq!(
            person
                .personal_data
                .date_of_birth
                .map(|d| d.format(DEFAULT_DATE_FORMAT).to_string()),
            Some("20-10-2000".to_string())
        );
        assert_eq!(
            person
                .personal_data
                .place_of_residence
                .as_ref()
                .map(|v| v.to_string()),
            Some("Amsterdam".to_string())
        );
        assert_eq!(
            person.personal_data.country.as_ref().map(|v| v.to_string()),
            Some("NL".to_string())
        );
        assert_eq!(
            person.address.postal_code.as_ref().map(|v| v.to_string()),
            Some("1234AB".to_string())
        );
        assert_eq!(
            person.address.house_number.as_ref().map(|v| v.to_string()),
            Some("12".to_string())
        );
        assert_eq!(
            person
                .address
                .house_number_addition
                .as_ref()
                .map(|v| v.to_string()),
            Some("a".to_string())
        );
        assert_eq!(
            person.address.street_name.as_ref().map(|v| v.to_string()),
            Some("Mooie Straat".to_string())
        );
        assert_eq!(
            person.address.locality.as_ref().map(|v| v.to_string()),
            Some("Rotterdam".to_string())
        );
        assert_eq!(person.representative, Representative::default());
    }

    #[test]
    fn validate_create_maps_representative_name_and_address_when_present() {
        let record = CandidateRecord::from(CandidateRecordCsv {
            voorletters: "J.".to_string(),
            roepnaam: "Jan".to_string(),
            voorvoegsel: "van de".to_string(),
            achternaam: "Berg".to_string(),
            woonplaats: "Antwerp".to_string(),
            landcode: "BE".to_string(),
            bsn: String::new(),
            geboortedatum: "20-10-2000".to_string(),
            geslacht: "v".to_string(),
            gemachtigde_voorletters: "P.".to_string(),
            gemachtigde_roepnaam: "Pietje".to_string(),
            gemachtigde_voorvoegsel: String::new(),
            gemachtigde_achternaam: "Puk".to_string(),
            correspondentie_postcode: String::new(),
            correspondentie_huisnummer: String::new(),
            correspondentie_toevoeging: String::new(),
            correspondentie_straatnaam: String::new(),
            correspondentie_plaats: String::new(),
            gemachtigde_postcode: "5678CD".to_string(),
            gemachtigde_huisnummer: "34".to_string(),
            gemachtigde_toevoeging: "b".to_string(),
            gemachtigde_straatnaam: "Mooiere Straat".to_string(),
            gemachtigde_plaats: "Den Haag".to_string(),
        });

        let person = record.validate_create(&CsrfTokens::default()).unwrap();

        assert_eq!(person.personal_data.gender, Some(Gender::Female));
        assert_eq!(
            person.personal_data.country.as_ref().map(|v| v.to_string()),
            Some("BE".to_string())
        );
        assert_eq!(person.address, DutchAddress::default());
        assert_eq!(person.representative.name.initials.to_string(), "P.");
        assert_eq!(
            person
                .representative
                .name
                .first_name
                .as_ref()
                .map(|v| v.to_string()),
            Some("Pietje".to_string())
        );
        assert_eq!(person.representative.name.last_name.to_string(), "Puk");
        assert_eq!(
            person
                .representative
                .address
                .postal_code
                .as_ref()
                .map(|v| v.to_string()),
            Some("5678CD".to_string())
        );
        assert_eq!(
            person
                .representative
                .address
                .house_number
                .as_ref()
                .map(|v| v.to_string()),
            Some("34".to_string())
        );
        assert_eq!(
            person
                .representative
                .address
                .house_number_addition
                .as_ref()
                .map(|v| v.to_string()),
            Some("b".to_string())
        );
        assert_eq!(
            person
                .representative
                .address
                .street_name
                .as_ref()
                .map(|v| v.to_string()),
            Some("Mooiere Straat".to_string())
        );
        assert_eq!(
            person
                .representative
                .address
                .locality
                .as_ref()
                .map(|v| v.to_string()),
            Some("Den Haag".to_string())
        );
    }
}
