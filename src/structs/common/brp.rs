use chrono::NaiveDate;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    AppError,
    common::{Bsn, BsnOrNoneConfirmed, DateOfBirth, DutchAddress, FullName},
    persons::{Person, PersonalData},
};

pub struct BrpClient {
    http_client: Client,
    base_url: String,
    api_key: String,
    persons_endpoint: String,
}

impl BrpClient {
    pub fn new(base_url: &str, api_key: &str, persons_endpoint: &str) -> Self {
        Self {
            http_client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            persons_endpoint: persons_endpoint.to_string(),
        }
    }

    pub async fn get_persons(&self, query: &BrpQuery) -> Result<Vec<BrpPerson>, AppError> {
        let url = format!("{}/{}", self.base_url, self.persons_endpoint);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(query)
            .send()
            .await?;

        match response.json::<BrpResponse>().await? {
            BrpResponse::ConsultWithBsn { persons } => Ok(persons),
        }
    }

    pub async fn verify(&self, person: &Person) -> Result<bool, AppError> {
        let query = match person.personal_data.bsn {
            Some(BsnOrNoneConfirmed::Bsn(ref bsn)) => BrpQuery::ConsultWithBsn {
                bsn: vec![bsn.clone()],
                fields: vec![
                    BrpField::Bsn,
                    BrpField::DateOfBirth,
                    BrpField::Gender,
                    BrpField::Initials,
                    BrpField::LastNamePrefix,
                    BrpField::LastName,
                    BrpField::OfficialStreetName,
                    BrpField::HouseNumber,
                    BrpField::HouseNumberAddition,
                    BrpField::PostalCode,
                    BrpField::PlaceOfResidence,
                ],
            },
            Some(BsnOrNoneConfirmed::NoneConfirmed) => {
                unimplemented!("BRP search with address? Or manual verification")
            }
            None => {
                unimplemented!(
                    "Return error, because this person should have a BSN (or none confirmed)?"
                )
            }
        };

        let brp_persons = self.get_persons(&query).await?;
        let brp_person = match brp_persons.as_slice() {
            [] => todo!("Handle person not found"),
            [brp_person] => brp_person,
            [..] => todo!("Handle person not unique"),
        };

        let address_is_valid = match &brp_person.address {
            Some(address) => {
                // Check all, except `known_in_bag`
                person.address.street_name == address.street_name
                    && person.address.house_number == address.house_number
                    && person.address.house_number_addition == address.house_number_addition
                    && person.address.locality == address.locality
                    && person.address.postal_code == address.postal_code
            }
            None => {
                eprintln!(
                    "Not a Dutch Address or no address at all (because the field 'verblijfplaats' was not included)"
                );
                true
            }
        };

        Ok(address_is_valid &&
            // Don't check First name (roepnaam)
            person.name.last_name == brp_person.name.last_name &&
            person.name.last_name_prefix == brp_person.name.last_name_prefix &&
            person.name.initials == brp_person.name.initials &&
            // Check all fields of personal_data except country, check gender only when filled in
            brp_person.personal_data.bsn == person.personal_data.bsn &&
            brp_person.personal_data.date_of_birth  == person.personal_data.date_of_birth &&
            // Gender field is optional, but if it is filled in, we check it
            (brp_person.personal_data.gender == person.personal_data.gender || person.personal_data.gender.is_none()))
    }
}

// ontbreekt: Aanduiding bijzonder Nederlanderschap
// ontbreekt: Ingangsdatum geldigheid met betrekking tot de elementen van de categorie Nationaliteit
#[derive(Debug, Serialize)]
pub enum BrpField {
    // Personen
    #[serde(rename = "burgerservicenummer")]
    Bsn,
    #[serde(rename = "naam.voornamen")]
    FirstNames,
    #[serde(rename = "naam.voorletters")]
    Initials,
    #[serde(rename = "naam.adellijkeTitelPredicaat")]
    TitleOfNobility,
    #[serde(rename = "naam.voorvoegsel")]
    LastNamePrefix,
    #[serde(rename = "naam.geslachtsnaam")]
    LastName,
    #[serde(rename = "geboorte.datum")]
    DateOfBirth,
    #[serde(rename = "geslacht")]
    Gender,
    #[serde(rename = "naam.aanduidingNaamgebruik")]
    DesignatedNameUsage,

    // Nationaliteit
    #[serde(rename = "nationaliteiten.nationaliteit")]
    Nationality,

    // Partners
    #[serde(rename = "partners.naam.voorvoegsel")]
    PartnerLastNamePrefix,
    #[serde(rename = "partners.naam.geslachtsnaam")]
    PartnerLastName,
    #[serde(rename = "partners.aangaanHuwelijkPartnerschap.datum")]
    DateOfMarriage,
    #[serde(rename = "partners.ontbindingHuwelijkPartnerschap")]
    DateOfDissolutionMarriage,

    // Date of death
    #[serde(rename = "overlijden.datum")]
    DateOfDeath,

    // Place of residence
    // TODO: What to do with Registratie Niet Ingezetenen?
    #[serde(rename = "gemeenteVanInschrijving")]
    RegisteredMunicipality,
    #[serde(rename = "datumInschrijvingInGemeente")]
    DateMunicipalRegistration,
    #[serde(rename = "verblijfplaats.verblijfadres.korteStraatnaam")]
    StreetName,
    // I think this is 'Naam openbare ruimte'
    #[serde(rename = "verblijfplaats.verblijfadres.officieleStraatnaam")]
    OfficialStreetName,
    #[serde(rename = "verblijfplaats.verblijfadres.huisnummer")]
    HouseNumber,
    #[serde(rename = "verblijfplaats.verblijfadres.huisletter")]
    HouseLetter,
    #[serde(rename = "verblijfplaats.verblijfadres.huisnummertoevoeging")]
    HouseNumberAddition,
    #[serde(rename = "verblijfplaats.verblijfadres.postcode")]
    PostalCode,
    #[serde(rename = "verblijfplaats.verblijfadres.woonplaats")]
    PlaceOfResidence,

    // Not sure if these are correct. They should be specifically foreign, but they
    // may also apply to interior addresses
    #[serde(rename = "verblijfplaats.verblijfadres.land")]
    CountryOfResidence, // Land adres buitenland
    #[serde(rename = "verblijfplaats.datumVan")]
    ResidenceDateFrom, // Datum aanvang adres buitenland
    #[serde(rename = "verblijfplaats.verblijfadres.regel1")]
    AddressLine1, // Regel 1 adres buitenland
    #[serde(rename = "verblijfplaats.verblijfadres.regel2")]
    AddressLine2, // Regel 2 adres buitenland
    #[serde(rename = "verblijfplaats.verblijfadres.regel3")]
    AddressLine3, // Regel 3 adres buitenland
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum BrpQuery {
    #[serde(rename = "RaadpleegMetBurgerservicenummer")]
    ConsultWithBsn {
        #[serde(rename = "burgerservicenummer")]
        bsn: Vec<Bsn>,
        fields: Vec<BrpField>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum BrpResponse {
    #[serde(rename = "RaadpleegMetBurgerservicenummer")]
    ConsultWithBsn {
        #[serde(rename = "personen")]
        persons: Vec<BrpPerson>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(from = "BrpPersonRaw")]
pub struct BrpPerson {
    name: FullName,
    personal_data: PersonalData,
    address: Option<DutchAddress>,
}

structstruck::strike! {
    #[structstruck::each[derive(Debug, Deserialize)]]
    struct BrpPersonRaw {
        #[serde(rename = "burgerservicenummer")]
        bsn: Option<String>,
        #[serde(rename = "geslacht")]
        gender: Option<struct BrpGender {
            #[serde(rename = "code")]
            gender: String
        }>,
        #[serde(rename = "naam")]
        name: Option<struct BrpName {
            #[serde(rename = "geslachtsnaam")]
            last_name: Option<String>,
            #[serde(rename = "voorvoegsel")]
            last_name_prefix: Option<String>,
            #[serde(rename = "voorletters")]
            initials: Option<String>
        }>,
        #[serde(rename = "geboorte")]
        birth: Option<struct BrpBirth {
            #[serde(rename = "datum")]
            date: Option<struct BrpDate {
                #[serde(rename = "datum")]
                date: Option<String>
            }>
        }>,
        #[serde(rename = "verblijfplaats")]
        place_of_residence: Option<
            #[serde(tag = "type")]
            enum BrpPlaceOfResidence {
            #[serde(rename = "Adres")]
            Address {
                #[serde(rename = "verblijfadres")]
                residence_address: struct BrpAddress {
                    // TODO: Confirm that this should be officieleStraatnaam
                    // Or handle this by checking if either matches? If this is only used as a correspondence address,
                    // then that should be sufficient
                    #[serde(rename = "officieleStraatnaam")]
                    street_name: Option<String>,
                    #[serde(rename = "huisnummer")]
                    house_number: Option<u32>,
                    #[serde(rename = "huisnummertoevoeging")]
                    house_number_addition: Option<String>,
                    #[serde(rename = "postcode")]
                    postal_code: Option<String>,
                    #[serde(rename = "woonplaats")]
                    place_of_residence: Option<String>,
                }
            },
            #[serde(other)]
            NonDutchAddress
        }>
    }
}

impl From<BrpPersonRaw> for BrpPerson {
    fn from(raw: BrpPersonRaw) -> Self {
        let name = raw
            .name
            .map(|naam| FullName {
                // First name isn't checked, because this does not necessarily need to be the same (roepnaam)
                first_name: None,
                last_name: naam
                    .last_name
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_default(),
                last_name_prefix: naam.last_name_prefix.and_then(|s| s.parse().ok()),
                initials: naam
                    .initials
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_default(),
            })
            .unwrap_or_default();

        let bsn = raw
            .bsn
            .and_then(|s| s.parse::<Bsn>().ok())
            .map(BsnOrNoneConfirmed::Bsn);

        let gender = raw.gender.and_then(|g| g.gender.parse().ok());

        let date_of_birth = raw
            .birth
            .as_ref()
            .and_then(|b| b.date.as_ref())
            .and_then(|d| d.date.as_ref())
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .map(DateOfBirth::from);

        let (address, place_of_residence) = match raw.place_of_residence {
            Some(BrpPlaceOfResidence::Address {
                residence_address: ra,
            }) => {
                let addr = Some(DutchAddress {
                    street_name: ra.street_name.and_then(|s| s.parse().ok()),
                    house_number: ra.house_number.and_then(|s| s.to_string().parse().ok()),
                    house_number_addition: ra.house_number_addition.and_then(|s| s.parse().ok()),
                    locality: ra
                        .place_of_residence
                        .as_deref()
                        .and_then(|s| s.parse().ok()),
                    postal_code: ra.postal_code.and_then(|s| s.parse().ok()),
                    // Known in BRP probably implies known in bag, I guess maybe this could be Some(true), but
                    // I don't think it matters
                    known_in_bag: None,
                });

                // TODO: Is place of residence really the same as locality (above).
                // (though note that above is parsed as `Locality`, and below as `PlaceOfResidence`)
                let por = ra.place_of_residence.and_then(|s| s.parse().ok());

                (addr, por)
            }
            Some(BrpPlaceOfResidence::NonDutchAddress) => {
                // TODO: How to handle this? Set the address to None and conduct an additional BRP check
                // for the Authorised Person?
                todo!("Not a Dutch Address")
            }
            None => {
                eprintln!("Field 'verblijfplaats' not included");
                (None, None)
            }
        };

        BrpPerson {
            name,
            personal_data: PersonalData {
                gender,
                bsn,
                date_of_birth,
                place_of_residence,
                // TODO: Can country be None here? Because we check with the BRP whether the address is international.
                // If it is, then `address` will be None (since we can't verify international addresses) and we know that
                // instead, it is necesarry to verify the Authorised Person's address
                country: None,
            },
            address,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::sample_person_from_brp;

    use super::*;

    #[tokio::test]
    async fn brp_request() {
        let brp_client =
            BrpClient::new("http://localhost:5010", "", "haalcentraal/api/brp/personen");
        let query = BrpQuery::ConsultWithBsn {
            bsn: vec!["100600505".parse().unwrap()],
            fields: vec![BrpField::LastName],
        };

        let response = brp_client.get_persons(&query).await.unwrap();
        let expected = "Digid 1 100600505 geslachtsnaam".parse().unwrap();
        assert!(response.first().unwrap().name.last_name == expected);
    }

    #[tokio::test]
    async fn brp_verify() {
        let brp_client =
            BrpClient::new("http://localhost:5010", "", "haalcentraal/api/brp/personen");

        let person = sample_person_from_brp();

        match brp_client.verify(&person).await {
            Err(e) => panic!("brp verification error: {e}"),
            Ok(false) => panic!(
                "person could not be verified: {}",
                serde_json::to_string_pretty(&person).unwrap()
            ),
            _ => {}
        }
    }
}
