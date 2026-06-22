use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

use crate::{AppError, common::Bsn, persons::Person};

pub struct BrpClient {
    http_client: Client,
    base_url: String,
    api_key: String, // Assuming an API key or Bearer token is needed for the real environment
}

impl BrpClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            http_client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Zoek personen endpoint (POST /personen)
    pub async fn get_persons(&self, query: &BrpQuery) -> Result<BrpResponse, Error> {
        let url = format!("{}/personen", self.base_url);

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

        // Parse the JSON response
        // let parsed_response = response.json::<BrpResponse>().await?;
        let parsed_response = BrpResponse::RaadpleegMetBurgerservicenummerResponse {
            personen: vec![BrpPerson {
                name: "Stefan".to_string(),
            }],
        };
        Ok(parsed_response)
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum BrpQuery {
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
struct BrpPerson {
    name: String,
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

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum BrpResponse {
    RaadpleegMetBurgerservicenummerResponse { personen: Vec<BrpPerson> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_name() {
        let brp_client = BrpClient::new("http://localhost:5010", "");
        let query = BrpQuery::ConsultWithBsn {
            burgerservicenummer: vec!["100600505".parse().unwrap()],
            fields: vec!["naam".to_string()],
        };
        brp_client.get_persons(&query).await;
    }
}
