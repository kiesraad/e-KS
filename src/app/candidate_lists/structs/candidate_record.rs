use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{
    OptionStringExt,
    common::{BsnOrNoneConfirmed, DutchAddress, DutchAddressForm, FullNameForm},
    constants::DEFAULT_DATE_FORMAT,
    core::AnyLocale,
    persons::{Person, PersonalDataFieldsForm, Representative, RepresentativeFieldsForm},
};

const NO_BSN: &str = "kandidaat heeft geen BSN";
pub(crate) const CSV_HEADERS: [&str; 23] = [
    "voorletters",
    "roepnaam",
    "voorvoegsel",
    "achternaam",
    "woonplaats",
    "landcode",
    "bsn",
    "geboortedatum",
    "geslacht",
    "correspondentie_postcode",
    "correspondentie_huisnummer",
    "correspondentie_toevoeging",
    "correspondentie_straatnaam",
    "correspondentie_plaats",
    "gemachtigde_voorletters",
    "gemachtigde_roepnaam",
    "gemachtigde_voorvoegsel",
    "gemachtigde_achternaam",
    "gemachtigde_postcode",
    "gemachtigde_huisnummer",
    "gemachtigde_toevoeging",
    "gemachtigde_straatnaam",
    "gemachtigde_plaats",
];

#[derive(Debug, Serialize, Deserialize, Clone, Default, Validate)]
#[validate(target = "Person")]
#[serde(default)]
pub struct CandidateRecord {
    #[serde(flatten)]
    #[validate(flatten)]
    name: FullNameForm,
    #[serde(flatten)]
    #[validate(flatten)]
    personal_data: PersonalDataFieldsForm,
    #[serde(flatten)]
    #[validate(flatten)]
    address: DutchAddressForm,
    #[serde(flatten)]
    #[validate(flatten)]
    representative: Option<RepresentativeFieldsForm>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub(crate) struct CandidateRecordCsv {
    voorletters: String,
    roepnaam: String,
    voorvoegsel: String,
    achternaam: String,

    woonplaats: String,
    landcode: String,
    bsn: String,
    geboortedatum: String,
    geslacht: String,

    correspondentie_postcode: String,
    correspondentie_huisnummer: String,
    correspondentie_toevoeging: String,
    correspondentie_straatnaam: String,
    correspondentie_plaats: String,

    gemachtigde_voorletters: String,
    gemachtigde_roepnaam: String,
    gemachtigde_voorvoegsel: String,
    gemachtigde_achternaam: String,

    gemachtigde_postcode: String,
    gemachtigde_huisnummer: String,
    gemachtigde_toevoeging: String,
    gemachtigde_straatnaam: String,
    gemachtigde_plaats: String,
}

impl From<CandidateRecordCsv> for CandidateRecord {
    fn from(csv: CandidateRecordCsv) -> Self {
        let representative = RepresentativeFieldsForm {
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
        };

        CandidateRecord {
            name: FullNameForm {
                first_name: csv.roepnaam,
                last_name: csv.achternaam,
                last_name_prefix: csv.voorvoegsel,
                initials: csv.voorletters,
            },
            personal_data: PersonalDataFieldsForm {
                gender: csv.geslacht.parse().unwrap_or_default(),
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
            representative: if representative.is_empty() {
                None
            } else {
                Some(representative)
            },
        }
    }
}

impl From<Person> for CandidateRecordCsv {
    fn from(person: Person) -> Self {
        let Person {
            name: candidate_name,
            personal_data: candidate_personal_data,
            address: person_address,
            representative: person_representative,
            ..
        } = person;

        let representative = match candidate_personal_data.country.as_ref() {
            Some(country) if !country.is_nl() => person_representative.unwrap_or_default(),
            _ => Representative::default(),
        };

        let address = match candidate_personal_data.country.as_ref() {
            Some(country) if country.is_nl() => person_address,
            _ => DutchAddress::default(),
        };

        CandidateRecordCsv {
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
                None => String::new(),
            },
            geboortedatum: candidate_personal_data
                .date_of_birth
                .map(|d| d.format(DEFAULT_DATE_FORMAT))
                .to_string_or_default(),
            geslacht: match candidate_personal_data.gender {
                Some(gender) => gender.abbreviation(AnyLocale::Nl).to_string(),
                None => String::new(),
            },

            correspondentie_postcode: address.postal_code.to_string_or_default(),
            correspondentie_huisnummer: address.house_number.to_string_or_default(),
            correspondentie_toevoeging: address.house_number_addition.to_string_or_default(),
            correspondentie_straatnaam: address.street_name.to_string_or_default(),
            correspondentie_plaats: address.locality.to_string_or_default(),

            gemachtigde_voorletters: representative.name.initials.to_string(),
            gemachtigde_roepnaam: representative.name.first_name.to_string_or_default(),
            gemachtigde_voorvoegsel: representative.name.last_name_prefix.to_string_or_default(),
            gemachtigde_achternaam: representative.name.last_name.to_string(),
            gemachtigde_postcode: representative.address.postal_code.to_string_or_default(),
            gemachtigde_huisnummer: representative.address.house_number.to_string_or_default(),
            gemachtigde_toevoeging: representative
                .address
                .house_number_addition
                .to_string_or_default(),
            gemachtigde_straatnaam: representative.address.street_name.to_string_or_default(),
            gemachtigde_plaats: representative.address.locality.to_string_or_default(),
        }
    }
}

impl From<Person> for CandidateRecord {
    fn from(person: Person) -> Self {
        Self::from(CandidateRecordCsv::from(person))
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

#[cfg(test)]
mod tests {
    use crate::{
        common::{DutchAddress, Gender},
        form::generate_csrf_token,
    };

    use super::*;

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
            correspondentie_postcode: "1234AB".to_string(),
            correspondentie_huisnummer: "12".to_string(),
            correspondentie_toevoeging: "a".to_string(),
            correspondentie_straatnaam: "Mooie Straat".to_string(),
            correspondentie_plaats: "Rotterdam".to_string(),
            ..Default::default()
        });

        let person = record.validate_create(&generate_csrf_token()).unwrap();

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
        assert_eq!(person.representative, None);
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
            gemachtigde_postcode: "5678CD".to_string(),
            gemachtigde_huisnummer: "34".to_string(),
            gemachtigde_toevoeging: "b".to_string(),
            gemachtigde_straatnaam: "Mooiere Straat".to_string(),
            gemachtigde_plaats: "Den Haag".to_string(),
            ..Default::default()
        });

        let person = record.validate_create(&generate_csrf_token()).unwrap();

        assert_eq!(person.personal_data.gender, Some(Gender::Female));
        assert_eq!(
            person.personal_data.country.as_ref().map(|v| v.to_string()),
            Some("BE".to_string())
        );
        assert_eq!(person.address, DutchAddress::default());

        let representative = person.representative.as_ref().unwrap();

        assert_eq!(representative.name.initials.to_string(), "P.");
        assert_eq!(
            representative
                .name
                .first_name
                .as_ref()
                .map(|v| v.to_string()),
            Some("Pietje".to_string())
        );
        assert_eq!(representative.name.last_name.to_string(), "Puk");
        assert_eq!(
            representative
                .address
                .postal_code
                .as_ref()
                .map(|v| v.to_string()),
            Some("5678CD".to_string())
        );
        assert_eq!(
            representative
                .address
                .house_number
                .as_ref()
                .map(|v| v.to_string()),
            Some("34".to_string())
        );
        assert_eq!(
            representative
                .address
                .house_number_addition
                .as_ref()
                .map(|v| v.to_string()),
            Some("b".to_string())
        );
        assert_eq!(
            representative
                .address
                .street_name
                .as_ref()
                .map(|v| v.to_string()),
            Some("Mooiere Straat".to_string())
        );
        assert_eq!(
            representative
                .address
                .locality
                .as_ref()
                .map(|v| v.to_string()),
            Some("Den Haag".to_string())
        );
    }
}
