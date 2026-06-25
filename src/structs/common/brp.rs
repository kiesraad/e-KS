use chrono::NaiveDate;
use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    AppError,
    common::{
        Bsn, BsnOrNoneConfirmed, CountryCode, DateOfBirth, DutchAddress, FullName, PlaceOfResidence,
    },
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

    async fn verify(&self, person: &Person) -> Result<bool, AppError> {
        let query = match person.personal_data.bsn {
            Some(BsnOrNoneConfirmed::Bsn(ref bsn)) => BrpQuery::ConsultWithBsn {
                bsn: vec![bsn.clone()],
                fields: vec![
                    // TODO: Create a Field type?
                    "burgerservicenummer".to_string(),
                    "geboorte".to_string(),
                    "geslacht".to_string(),
                    "naam".to_string(),
                    "verblijfplaats".to_string(),
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

        Ok([
            // Check all, except `known_in_bag`
            person.address.street_name == brp_person.address.street_name,
            person.address.house_number == brp_person.address.house_number,
            person.address.house_number_addition == brp_person.address.house_number_addition,
            person.address.locality == brp_person.address.locality,
            person.address.postal_code == brp_person.address.postal_code,
            // Don't check First name (roepnaam)
            person.name.last_name == brp_person.name.last_name,
            person.name.last_name_prefix == brp_person.name.last_name_prefix,
            person.name.initials == brp_person.name.initials,
            // Check all fields of personal_data
            brp_person.personal_data == person.personal_data,
        ]
        .iter()
        .inspect(|f| println!("{f:?}"))
        .all(|&b| b))
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum BrpQuery {
    #[serde(rename = "RaadpleegMetBurgerservicenummer")]
    ConsultWithBsn {
        #[serde(rename = "burgerservicenummer")]
        bsn: Vec<Bsn>,
        fields: Vec<String>,
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

// --- Intermediate deserialization structs for the BRP JSON format ---
#[derive(Deserialize)]
struct BrpGender {
    #[serde(rename = "code")]
    gender: String,
}

#[derive(Deserialize)]
struct BrpDate {
    #[serde(rename = "datum")]
    date: Option<String>,
}

#[derive(Deserialize)]
struct BrpName {
    #[serde(rename = "voornamen")]
    first_names: Option<String>,
    #[serde(rename = "geslachtsnaam")]
    last_name: Option<String>,
    #[serde(rename = "voorvoegsel")]
    last_name_prefix: Option<String>,
    #[serde(rename = "voorletters")]
    initials: Option<String>,
}

#[derive(Deserialize)]
struct BrpBirth {
    #[serde(rename = "datum")]
    date: Option<BrpDate>,
}

#[derive(Deserialize)]
struct BrpAddress {
    // TODO: Confirm that this should be officieleStraatnaam
    // Or handle this by checking if either matches? If this is only used as a correspondence address,
    // than that should be sufficient
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

#[derive(Deserialize)]
#[serde(tag = "type")]
enum BrpPlaceOfResidence {
    #[serde(rename = "Adres")]
    Address {
        #[serde(rename = "verblijfadres")]
        residence_address: Option<BrpAddress>,
    },
    #[serde(rename = "VerblijfPlaatsBuitenland")]
    InternationalResidenceAddress {
        #[serde(rename = "verblijfadres")]
        residence_address: Option<BrpAddress>,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct BrpPersonRaw {
    #[serde(rename = "burgerservicenummer")]
    bsn: Option<String>,
    #[serde(rename = "geslacht")]
    gender: Option<BrpGender>,
    #[serde(rename = "naam")]
    name: Option<BrpName>,
    #[serde(rename = "geboorte")]
    birth: Option<BrpBirth>,
    #[serde(rename = "verblijfplaats")]
    place_of_residence: Option<BrpPlaceOfResidence>,
}

#[derive(Debug, Deserialize)]
#[serde(from = "BrpPersonRaw")]
pub struct BrpPerson {
    name: FullName,
    personal_data: PersonalData,
    address: DutchAddress,
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

        let (place_of_residence, country, address) = match raw.place_of_residence {
            Some(BrpPlaceOfResidence::Address { residence_address }) => {
                let country: Option<CountryCode> = "NL".parse().ok();
                let (por, addr) = residence_address
                    .map(|va| {
                        let por: Option<PlaceOfResidence> = va
                            .place_of_residence
                            .as_deref()
                            .and_then(|s| s.parse().ok());
                        let addr = DutchAddress {
                            street_name: va.street_name.and_then(|s| s.parse().ok()),
                            house_number: va.house_number.and_then(|n| n.to_string().parse().ok()),
                            house_number_addition: va
                                .house_number_addition
                                .and_then(|s| s.parse().ok()),
                            locality: va.place_of_residence.and_then(|s| s.parse().ok()),
                            postal_code: va.postal_code.and_then(|s| s.parse().ok()),
                            known_in_bag: None,
                        };
                        (por, addr)
                    })
                    .unzip();
                (por.flatten(), country, addr.unwrap_or_default())
            }
            _ => (None, None, DutchAddress::default()),
        };

        BrpPerson {
            name,
            personal_data: PersonalData {
                gender,
                bsn,
                date_of_birth,
                place_of_residence,
                country,
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
            fields: vec!["naam".to_string()],
        };

        let response = brp_client.get_persons(&query).await.unwrap();
        println!("{:?}", response);
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
