use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    AppError,
    common::{Bsn, DutchAddress, FullName},
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

    /// Zoek personen endpoint (POST /personen)
    pub async fn get_persons(&self, query: &BrpQuery) -> Result<BrpResponse, AppError> {
        let url = format!("{}/{}", self.base_url, self.persons_endpoint);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(query)
            .send()
            .await?;

        // Check for HTTP errors (400, 401, 403, etc.)
        let response = response.error_for_status()?;

        dbg!(&response);

        Ok(parsed_response)
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum BrpQuery {
    #[serde(rename = "RaadpleegMetBurgerservicenummer")]
    ConsultWithBsn {
        burgerservicenummer: Vec<Bsn>,
        fields: Vec<String>,
    },
    // SearchWithLastNameAndDateOfBirth,
    // SearchWithLastNameAndRegisteredMunicipality,
    // SearchWithPostalCodeAndHouseNumber,
    // SearchWithStreetHouseNumberAndRegisteredMunicipality,
    // // TODO: translate this better if we ever end up using it.
    // // Seems to be related to identifying a residence
    // SearchWithNumberingIdentification,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum BrpResponse {
    #[serde(rename = "RaadpleegMetBurgerservicenummer")]
    ConsultWithBsn { personen: Vec<BrpPerson> },
}

#[derive(Debug, Deserialize)]
struct BrpPerson {
    name: FullName,
    personal_data: PersonalData,
    address: DutchAddress,
}

// Misschien beter equality?
impl TryFrom<BrpPerson> for Person {
    type Error = AppError;

    fn try_from(value: BrpPerson) -> Result<Self, Self::Error> {
        Err(AppError::InternalServerError)
    }
}

pub trait BrpVerification {
    // should become AppError
    // async fn verify(&self) -> Result<bool, String>;
    fn verify(&self) -> impl std::future::Future<Output = Result<bool, String>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn brp_request() {
        let brp_client =
            BrpClient::new("http://localhost:5010", "", "haalcentraal/api/brp/personen");
        let query = BrpQuery::ConsultWithBsn {
            burgerservicenummer: vec!["100600505".parse().unwrap()],
            fields: vec!["naam".to_string()],
        };

        let response = brp_client.get_persons(&query).await.unwrap();
        println!("{:?}", response);
    }
}
