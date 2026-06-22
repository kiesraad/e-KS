use chrono::NaiveDate;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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

        let parsed_response = response.json::<BrpResponse>().await?;

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
    ConsultWithBsn {
        #[serde(rename = "personen")]
        persons: Vec<BrpPerson>,
    },
}

// --- Intermediate deserialization structs for the BRP JSON format ---
#[derive(Deserialize)]
struct BrpGender {
    gender: String,
}

#[derive(Deserialize)]
struct BrpDatum {
    datum: Option<String>,
}

#[derive(Deserialize)]
struct BrpNaam {
    voornamen: Option<String>,
    geslachtsnaam: Option<String>,
    voorvoegsel: Option<String>,
    voorletters: Option<String>,
}

#[derive(Deserialize)]
struct BrpGeboorte {
    datum: Option<BrpDatum>,
}

#[derive(Deserialize)]
struct BrpVerblijfadres {
    #[serde(rename = "korteStraatnaam")]
    korte_straatnaam: Option<String>,
    huisnummer: Option<u32>,
    huisnummertoevoeging: Option<String>,
    postcode: Option<String>,
    woonplaats: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum BrpVerblijfplaats {
    Adres {
        verblijfadres: Option<BrpVerblijfadres>,
    },
    VerblijfplaatsBuitenland {
        verblijfadres: Option<BrpVerblijfadres>,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct BrpPersonRaw {
    burgerservicenummer: Option<String>,
    geslacht: Option<BrpGender>,
    naam: Option<BrpNaam>,
    geboorte: Option<BrpGeboorte>,
    verblijfplaats: Option<BrpVerblijfplaats>,
}

// --- BrpPerson with custom deserialization via BrpPersonRaw ---

#[derive(Debug, Deserialize)]
#[serde(try_from = "BrpPersonRaw")]
struct BrpPerson {
    name: FullName,
    personal_data: PersonalData,
    address: DutchAddress,
}

impl From<BrpPersonRaw> for BrpPerson {
    fn from(raw: BrpPersonRaw) -> Self {
        let name = raw
            .naam
            .map(|naam| FullName {
                first_name: naam.voornamen.and_then(|s| s.parse().ok()),
                last_name: naam
                    .geslachtsnaam
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_default(),
                last_name_prefix: naam.voorvoegsel.and_then(|s| s.parse().ok()),
                initials: naam
                    .voorletters
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_default(),
            })
            .unwrap_or_default();

        let bsn = raw
            .burgerservicenummer
            .and_then(|s| s.parse::<Bsn>().ok())
            .map(BsnOrNoneConfirmed::Bsn);

        let gender = raw.geslacht.and_then(|g| g.gender.parse().ok());

        let date_of_birth = raw
            .geboorte
            .as_ref()
            .and_then(|g| g.datum.as_ref())
            .and_then(|d| d.datum.as_ref())
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
            .map(DateOfBirth::from);

        let (place_of_residence, country, address) = match raw.verblijfplaats {
            Some(BrpVerblijfplaats::Adres { verblijfadres }) => {
                let country: Option<CountryCode> = "NL".parse().ok();
                let (por, addr) = verblijfadres
                    .map(|va| {
                        let por: Option<PlaceOfResidence> =
                            va.woonplaats.as_deref().and_then(|s| s.parse().ok());
                        let addr = DutchAddress {
                            street_name: va.korte_straatnaam.and_then(|s| s.parse().ok()),
                            house_number: va.huisnummer.and_then(|n| n.to_string().parse().ok()),
                            house_number_addition: va
                                .huisnummertoevoeging
                                .and_then(|s| s.parse().ok()),
                            locality: va.woonplaats.and_then(|s| s.parse().ok()),
                            postal_code: va.postcode.and_then(|s| s.parse().ok()),
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

// Misschien beter equality?
impl TryFrom<BrpPerson> for Person {
    type Error = AppError;

    fn try_from(_value: BrpPerson) -> Result<Self, Self::Error> {
        Err(AppError::InternalServerError)
    }
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
