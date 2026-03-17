use serde::{Deserialize, Serialize};

use crate::{
    OptionStringExt,
    common::{BsnOrNoneConfirmed, DutchAddress},
    core::AnyLocale, persons::Person,
};

const NO_BSN: &str = "geen";

#[derive(Debug, Serialize, Deserialize)]
pub struct CandidateRecord {
    kandidaat_voorletters: String,
    kandidaat_roepnaam: String,
    kandidaat_voorvoegsel: String,
    kandidaat_achternaam: String,

    kandidaat_woonplaats: String,
    kandidaat_landcode: String,
    kandidaat_bsn: String,
    kandidaat_geboortedatum: String,
    kandidaat_geslacht: String,

    gemachtigde_voorletters: String,
    gemachtigde_roepnaam: String,
    gemachtigde_voorvoegsel: String,
    gemachtigde_achternaam: String,

    correspondentie_of_gemachtigde_postcode: String,
    correspondentie_of_gemachtigde_huisnummer: String,
    correspondentie_of_gemachtigde_toevoeging: String,
    correspondentie_of_gemachtigde_straatnaam: String,
    correspondentie_of_gemachtigde_woonplaats: String,
}

impl From<Person> for CandidateRecord {
    fn from(person: Person) -> Self {
        let candidate_name = person.name;
        let candidate_personal_data = person.personal_data;

        let authorised_name = person.representative.name;

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
            kandidaat_voorletters: candidate_name.initials.to_string(),
            kandidaat_roepnaam: candidate_name.first_name.to_string_or_default(),
            kandidaat_voorvoegsel: candidate_name.last_name_prefix.to_string_or_default(),
            kandidaat_achternaam: candidate_name.last_name.to_string(),

            kandidaat_woonplaats: candidate_personal_data
                .place_of_residence
                .to_string_or_default(),
            kandidaat_landcode: candidate_personal_data.country.to_string_or_default(),
            kandidaat_bsn: match candidate_personal_data.bsn {
                Some(BsnOrNoneConfirmed::NoneConfirmed) => NO_BSN.to_string(),
                Some(BsnOrNoneConfirmed::Bsn(bsn)) => bsn.to_exposed_string(),
                None => "".to_string(),
            },
            kandidaat_geboortedatum: candidate_personal_data.date_of_birth.to_string_or_default(),
            kandidaat_geslacht: match candidate_personal_data.gender {
                Some(gender) => gender.abbreviation(AnyLocale::Nl).to_string(),
                None => "".to_string(),
            },

            gemachtigde_voorletters: authorised_name.initials.to_string(),
            gemachtigde_roepnaam: authorised_name.first_name.to_string_or_default(),
            gemachtigde_voorvoegsel: authorised_name.last_name_prefix.to_string_or_default(),
            gemachtigde_achternaam: authorised_name.last_name.to_string(),

            correspondentie_of_gemachtigde_postcode: address.postal_code.to_string_or_default(),
            correspondentie_of_gemachtigde_huisnummer: address.house_number.to_string_or_default(),
            correspondentie_of_gemachtigde_toevoeging: address
                .house_number_addition
                .to_string_or_default(),
            correspondentie_of_gemachtigde_straatnaam: address.street_name.to_string_or_default(),
            correspondentie_of_gemachtigde_woonplaats: address.locality.to_string_or_default(),
        }
    }
}
