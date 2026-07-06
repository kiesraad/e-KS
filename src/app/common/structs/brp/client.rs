use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    AppError,
    common::{
        Bsn, BsnOrNoneConfirmed,
        structs::brp::{BrpField, BrpPerson},
    },
    csb::{Omission, OmissionCategory},
    persons::Person,
};

#[derive(Clone)]
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

    pub async fn verify(&self, person: &Person) -> Result<Vec<Omission>, AppError> {
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
                // TODO: This needs to be implemented
                tracing::warn!(
                    "Person {} currently does not have a BSN filled in (or none confirmed)\n{:?}",
                    person.id,
                    person
                );
                return Err(AppError::GenericNotFound);
            }
        };

        let mut omissions = vec![];
        let mut add_omission = |description: &str, help_text: &str| {
            omissions.push(Omission::new(
                OmissionCategory::Candidate {
                    person: person.id,
                    list: None,
                },
                // TODO: These should likely be user configurable and translatable
                description.to_string(),
                help_text.to_string(),
            ));
        };

        let brp_persons = self.get_persons(&query).await?;
        let brp_person = match brp_persons.as_slice() {
            [] => {
                add_omission(
                    "Er is geen persoon gevonden met dit burgerservicenummer",
                    "Controleer of er een fout is gemaakt bij het invoeren",
                );
                return Ok(omissions);
            }
            [brp_person] => brp_person,
            [..] => {
                add_omission(
                    "Er zijn meerder personen gevonden met dit burgerservicenummer",
                    "Controleer of er een fout is gemaakt bij het invoeren",
                );
                return Ok(omissions);
            }
        };

        match &brp_person.address {
            Some(address) => {
                // Check all, except `known_in_bag`
                if person.address.street_name != address.street_name {
                    add_omission(
                        "De straatnaam komt niet overeen met de BRP",
                        "Controleer de straatnaam",
                    );
                }
                if person.address.house_number != address.house_number {
                    add_omission(
                        "Het huisnummer komt niet overeen met de BRP",
                        "Controleer het huisnummer",
                    );
                }
                if person.address.house_number_addition != address.house_number_addition {
                    add_omission(
                        "De huisnummertoevoeging komt niet overeen met de BRP",
                        "Controleer de huisnummertoevoeging",
                    );
                }
                if person.address.locality != address.locality {
                    add_omission(
                        "De woonplaats komt niet overeen met de BRP",
                        "Controleer de woonplaats",
                    );
                }
                if person.address.postal_code != address.postal_code {
                    add_omission(
                        "De postcode komt niet overeen met de BRP",
                        "Controleer de postcode",
                    );
                }
            }
            None => {
                tracing::warn!(
                    "Not a Dutch Address or no address at all (because the field 'verblijfplaats' was not included)"
                );
            }
        };

        // Don't check first name (roepnaam)
        if person.name.last_name != brp_person.name.last_name {
            add_omission(
                "De achternaam komt niet overeen met de BRP",
                "Controleer de achternaam",
            );
        }
        if person.name.last_name_prefix != brp_person.name.last_name_prefix {
            add_omission(
                "Het voorvoegsel komt niet overeen met de BRP",
                "Controleer het voorvoegsel",
            );
        }
        if person.name.initials != brp_person.name.initials {
            add_omission(
                "De voorletters komen niet overeen met de BRP",
                "Controleer de voorletters",
            );
        }

        // Check all fields of personal_data except country, check gender only when filled in
        if brp_person.personal_data.bsn != person.personal_data.bsn {
            add_omission(
                "Het burgerservicenummer komt niet overeen met de BRP",
                "Controleer het burgerservicenummer",
            );
        }
        if brp_person.personal_data.date_of_birth != person.personal_data.date_of_birth {
            add_omission(
                "De geboortedatum komt niet overeen met de BRP",
                "Controleer de geboortedatum",
            );
        }
        // Gender field is optional, but if it is filled in, we check it
        if person.personal_data.gender.is_some()
            && brp_person.personal_data.gender != person.personal_data.gender
        {
            add_omission(
                "Het geslacht komt niet overeen met de BRP",
                "Controleer het geslacht",
            );
        }

        Ok(omissions)
    }
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

#[cfg(test)]
mod tests {
    use crate::{
        persons::PersonId,
        test_utils::{sample_person, sample_person_from_brp},
    };

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
            Ok(omissions) if !omissions.is_empty() => panic!(
                "person could not be verified, omissions: {omissions:?}\nperson: {}",
                serde_json::to_string_pretty(&person).unwrap()
            ),
            _ => {}
        }
    }

    #[tokio::test]
    async fn brp_verify_returns_omissions() {
        let brp_client =
            BrpClient::new("http://localhost:5010", "", "haalcentraal/api/brp/personen");

        let mut person = sample_person(PersonId::new());
        // Dit bsn voldoet aan de 11-proef maar staat niet in de mock brp
        person.personal_data.bsn = Some("123456782".parse().unwrap());
        match brp_client.verify(&person).await {
            Ok(ommissions) => {
                assert_eq!(ommissions.len(), 1);
                let ommission = &ommissions[0];
                assert!(matches!(
                    ommission.category,
                    OmissionCategory::Candidate { .. }
                ));
                assert_eq!(
                    ommission.description,
                    "Er is geen persoon gevonden met dit burgerservicenummer",
                )
            }
            Err(e) => panic!("{e}"),
        }

        let mut person = sample_person_from_brp();
        person.address.house_number_addition = Some("nope".parse().unwrap());
        match brp_client.verify(&person).await {
            Ok(ommissions) => {
                assert_eq!(ommissions.len(), 1);
                let ommission = &ommissions[0];
                assert!(matches!(
                    ommission.category,
                    OmissionCategory::Candidate { .. }
                ));
                assert_eq!(
                    ommission.description,
                    "De huisnummertoevoeging komt niet overeen met de BRP",
                )
            }
            Err(e) => panic!("{e}"),
        }

        let mut person = sample_person(PersonId::new());
        // De gegevens in de brp voor dit bsn komen in zijn geheel niet overeen. Dit zou kunnen voorkomen
        // als het verkeerde bsn is ingevuld.
        person.personal_data.bsn = Some("999992806".parse().unwrap());

        dbg!(&person);
        match brp_client.verify(&person).await {
            Ok(ommissions) => {
                dbg!(&ommissions);
                assert_eq!(ommissions.len(), 8);
            }
            Err(e) => panic!("{e}"),
        }
    }
}
