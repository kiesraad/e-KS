use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{BrpField, BrpPerson};
use crate::{
    AppError,
    structs::{
        candidate_lists::CandidateListId,
        common::{Bsn, BsnOrNoneConfirmed, DutchAddress, FullName},
        csb::{Omission, OmissionCategory},
        persons::{Person, PersonId, PersonalData},
    },
};

#[derive(Clone)]
pub struct BrpClient {
    http_client: Client,
    base_url: String,
    api_key: String,
    persons_endpoint: String,
    timeout: Duration,
}

impl BrpClient {
    pub fn new(base_url: &str, api_key: &str, persons_endpoint: &str, timeout: Duration) -> Self {
        Self {
            http_client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            persons_endpoint: persons_endpoint.to_string(),
            timeout,
        }
    }

    #[cfg(test)]
    pub fn new_for_test() -> Self {
        use crate::constants;

        BrpClient::new(
            "http://localhost:5010",
            "",
            constants::BRP_PERSONS_ENDPOINT,
            Duration::from_secs(5),
        )
    }

    pub async fn get_persons(&self, query: &BrpQuery) -> Result<Vec<BrpPerson>, AppError> {
        let url = format!("{}/{}", self.base_url, self.persons_endpoint);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(query)
            .timeout(self.timeout)
            .send()
            .await?;

        match response.json::<BrpResponse>().await? {
            BrpResponse::ConsultWithBsn { persons } => Ok(persons),
        }
    }

    pub async fn verify(
        &self,
        person: &Person,
        candidate_lists: Vec<CandidateListId>,
    ) -> Result<Vec<Omission>, AppError> {
        let query = brp_query_for(person)?;

        let brp_persons = self.get_persons(&query).await?;
        let brp_person = match brp_persons.as_slice() {
            [] => {
                return Ok(vec![build_omission(
                    person.id,
                    &candidate_lists,
                    "Burgerservicenummer onbekend",
                    "Er is geen persoon gevonden met dit burgerservicenummer",
                    "Controleer of er een fout is gemaakt bij het invoeren",
                )?]);
            }
            [brp_person] => brp_person,
            [..] => {
                return Ok(vec![build_omission(
                    person.id,
                    &candidate_lists,
                    "Burgerservicenummer niet uniek",
                    "Er zijn meerder personen gevonden met dit burgerservicenummer",
                    "Controleer of er een fout is gemaakt bij het invoeren",
                )?]);
            }
        };

        let mut omissions = address_omissions(
            person.id,
            &candidate_lists,
            &person.address,
            brp_person.address.as_ref(),
        )?;
        omissions.extend(name_omissions(
            person.id,
            &candidate_lists,
            &person.name,
            &brp_person.name,
        )?);
        omissions.extend(personal_data_omissions(
            person.id,
            &candidate_lists,
            &person.personal_data,
            &brp_person.personal_data,
        )?);

        Ok(omissions)
    }
}

/// Builds the BRP lookup query for a person's BSN. Only BSN-based lookup is
/// supported today; the other two cases are follow-up work.
fn brp_query_for(person: &Person) -> Result<BrpQuery, AppError> {
    match person.personal_data.bsn {
        Some(BsnOrNoneConfirmed::Bsn(ref bsn)) => Ok(BrpQuery::ConsultWithBsn {
            bsn: vec![bsn.clone()],
            fields: vec![
                BrpField::Bsn,
                BrpField::DateOfBirth,
                BrpField::Gender,
                BrpField::Initials,
                BrpField::LastNamePrefix,
                BrpField::LastName,
                BrpField::StreetName,
                BrpField::HouseNumber,
                BrpField::HouseNumberAddition,
                BrpField::PostalCode,
                BrpField::PlaceOfResidence,
            ],
        }),
        Some(BsnOrNoneConfirmed::NoneConfirmed) => {
            let error_text = format!("Person {} has BSN none confirmed", person.id);
            tracing::error!(error_text);
            Err(AppError::BrpError(error_text))
        }
        None => {
            let error_text = format!("Person {} does not have a BSN filled in", person.id);
            tracing::warn!(error_text);
            Err(AppError::BrpError(error_text))
        }
    }
}

// TODO: These should likely be user configurable and translatable
fn build_omission(
    person_id: PersonId,
    candidate_lists: &[CandidateListId],
    title: &str,
    description: &str,
    help_text: &str,
) -> Result<Omission, AppError> {
    Ok(Omission::new(
        OmissionCategory::Candidate {
            person: person_id,
            lists: candidate_lists.to_vec(),
        },
        title.parse().map_err(|_| AppError::InternalServerError)?,
        description
            .parse()
            .map_err(|_| AppError::InternalServerError)?,
        Some(
            help_text
                .parse()
                .map_err(|_| AppError::InternalServerError)?,
        ),
    ))
}

fn address_omissions(
    person_id: PersonId,
    candidate_lists: &[CandidateListId],
    person_address: &DutchAddress,
    brp_address: Option<&DutchAddress>,
) -> Result<Vec<Omission>, AppError> {
    let mut omissions = Vec::new();

    // Check all, except `known_in_bag`
    let Some(address) = brp_address else {
        tracing::warn!(
            "Not a Dutch Address or no address at all (because the field 'verblijfplaats' was not included)"
        );
        return Ok(omissions);
    };

    if person_address.street_name != address.street_name {
        omissions.push(build_omission(
            person_id,
            candidate_lists,
            "Onjuiste straatnaam",
            "De straatnaam komt niet overeen met de BRP",
            "Controleer de straatnaam",
        )?);
    }
    if person_address.house_number != address.house_number {
        omissions.push(build_omission(
            person_id,
            candidate_lists,
            "Onjuist huisnummer",
            "Het huisnummer komt niet overeen met de BRP",
            "Controleer het huisnummer",
        )?);
    }
    if person_address.house_number_addition != address.house_number_addition {
        omissions.push(build_omission(
            person_id,
            candidate_lists,
            "Onjuiste huisnummertoevoeging",
            "De huisnummertoevoeging komt niet overeen met de BRP",
            "Controleer de huisnummertoevoeging",
        )?);
    }
    if person_address.locality != address.locality {
        omissions.push(build_omission(
            person_id,
            candidate_lists,
            "Onjuiste woonplaats",
            "De woonplaats komt niet overeen met de BRP",
            "Controleer de woonplaats",
        )?);
    }
    if person_address.postal_code != address.postal_code {
        omissions.push(build_omission(
            person_id,
            candidate_lists,
            "Onjuiste postcode",
            "De postcode komt niet overeen met de BRP",
            "Controleer de postcode",
        )?);
    }

    Ok(omissions)
}

fn name_omissions(
    person_id: PersonId,
    candidate_lists: &[CandidateListId],
    person_name: &FullName,
    brp_name: &FullName,
) -> Result<Vec<Omission>, AppError> {
    let mut omissions = Vec::new();

    // Don't check first name (roepnaam)
    if person_name.last_name != brp_name.last_name {
        omissions.push(build_omission(
            person_id,
            candidate_lists,
            "Onjuiste achternaam",
            "De achternaam komt niet overeen met de BRP",
            "Controleer de achternaam",
        )?);
    }
    if person_name.last_name_prefix != brp_name.last_name_prefix {
        omissions.push(build_omission(
            person_id,
            candidate_lists,
            "Onjuist voorvoegsel",
            "Het voorvoegsel komt niet overeen met de BRP",
            "Controleer het voorvoegsel",
        )?);
    }
    if person_name.initials != brp_name.initials {
        omissions.push(build_omission(
            person_id,
            candidate_lists,
            "Onjuiste voorletters",
            "De voorletters komen niet overeen met de BRP",
            "Controleer de voorletters",
        )?);
    }

    Ok(omissions)
}

fn personal_data_omissions(
    person_id: PersonId,
    candidate_lists: &[CandidateListId],
    person_data: &PersonalData,
    brp_data: &PersonalData,
) -> Result<Vec<Omission>, AppError> {
    let mut omissions = Vec::new();

    // Check all fields of personal_data except country, check gender only when filled in
    if brp_data.bsn != person_data.bsn {
        omissions.push(build_omission(
            person_id,
            candidate_lists,
            "Onjuist burgerservicenummer",
            "Het burgerservicenummer komt niet overeen met de BRP",
            "Controleer het burgerservicenummer",
        )?);
    }
    if brp_data.date_of_birth != person_data.date_of_birth {
        omissions.push(build_omission(
            person_id,
            candidate_lists,
            "Onjuiste geboortedatum",
            "De geboortedatum komt niet overeen met de BRP",
            "Controleer de geboortedatum",
        )?);
    }
    // Gender field is optional, but if it is filled in, we check it
    if person_data.gender.is_some() && brp_data.gender != person_data.gender {
        omissions.push(build_omission(
            person_id,
            candidate_lists,
            "Onjuist geslacht",
            "Het geslacht komt niet overeen met de BRP",
            "Controleer het geslacht",
        )?);
    }

    Ok(omissions)
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
    use std::collections::HashSet;

    use crate::{
        structs::persons::PersonId,
        test_utils::{sample_person, sample_person_from_brp},
    };

    use super::*;

    #[tokio::test]
    async fn brp_request() {
        let brp_client = BrpClient::new_for_test();
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
        let brp_client = BrpClient::new_for_test();

        let person = sample_person_from_brp();

        match brp_client.verify(&person, Vec::new()).await {
            Err(e) => panic!("brp verification error: {e}"),
            Ok(omissions) if !omissions.is_empty() => panic!(
                "person could not be verified: {}\nFollowing omissions were found: {:?}",
                serde_json::to_string_pretty(&person).unwrap(),
                omissions
            ),
            _ => {}
        }
    }

    #[tokio::test]
    async fn brp_verify_returns_omissions() {
        let brp_client = BrpClient::new_for_test();

        let list_id = CandidateListId::new();
        let mut person = sample_person(PersonId::new());
        // Dit bsn voldoet aan de 11-proef maar staat niet in de mock brp
        person.personal_data.bsn = Some("123456782".parse().unwrap());
        match brp_client.verify(&person, vec![list_id]).await {
            Ok(omissions) => {
                assert_eq!(omissions.len(), 1);
                let omission = &omissions[0];
                let OmissionCategory::Candidate { lists, .. } = &omission.category else {
                    panic!("Unexpected omission category")
                };
                assert_eq!(lists, &[list_id]);
                assert_eq!(
                    omission.description.as_str(),
                    "Er is geen persoon gevonden met dit burgerservicenummer",
                )
            }
            Err(e) => panic!("{e}"),
        }

        let mut person = sample_person_from_brp();
        person.address.house_number_addition = Some("nope".parse().unwrap());
        match brp_client.verify(&person, Vec::new()).await {
            Ok(omissions) => {
                assert_eq!(omissions.len(), 1);
                let omission = &omissions[0];
                assert!(matches!(
                    omission.category,
                    OmissionCategory::Candidate { .. }
                ));
                assert_eq!(
                    omission.description.as_str(),
                    "De huisnummertoevoeging komt niet overeen met de BRP",
                )
            }
            Err(e) => panic!("{e}"),
        }

        let mut person = sample_person(PersonId::new());
        // De gegevens in de brp voor dit bsn komen in zijn geheel niet overeen. Dit zou kunnen voorkomen
        // als het verkeerde bsn is ingevuld.
        person.personal_data.bsn = Some("999992806".parse().unwrap());

        let expected_titles: HashSet<String> = [
            "Onjuist huisnummer".to_string(),
            "Onjuiste achternaam".to_string(),
            "Onjuiste geboortedatum".to_string(),
            "Onjuiste huisnummertoevoeging".to_string(),
            "Onjuiste postcode".to_string(),
            "Onjuiste straatnaam".to_string(),
            "Onjuiste voorletters".to_string(),
            "Onjuiste woonplaats".to_string(),
        ]
        .into();

        match brp_client.verify(&person, Vec::new()).await {
            Ok(omissions) => {
                let actual_titles =
                    HashSet::from_iter(omissions.into_iter().map(|o| o.title.to_string()));
                assert_eq!(
                    expected_titles.symmetric_difference(&actual_titles).count(),
                    0
                )
            }
            Err(e) => panic!("{e}"),
        }
    }

    #[tokio::test]
    async fn omission_includes_candidate_lists() {
        let brp_client = BrpClient::new_for_test();

        let person = sample_person_from_brp();
        let list_id = CandidateListId::new();

        match brp_client.verify(&person, vec![list_id]).await {
            Err(e) => panic!("brp verification error: {e}"),
            Ok(omissions) if !omissions.is_empty() => panic!(
                "person could not be verified: {}\nFollowing omissions were found: {:?}",
                serde_json::to_string_pretty(&person).unwrap(),
                omissions
            ),
            _ => {}
        }
    }
}
