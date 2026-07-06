use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    AppError,
    common::{
        Bsn, BsnOrNoneConfirmed,
        structs::brp::{BrpField, BrpPerson},
    },
    persons::Person,
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
