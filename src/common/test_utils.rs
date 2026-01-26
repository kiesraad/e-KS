use chrono::{NaiveDate, Utc};
use http_body_util::BodyExt;

use crate::{
    ElectoralDistrict, TokenValue,
    candidate_lists::{CandidateList, CandidateListId},
    persons::{AddressForm, Gender, Person, PersonForm, PersonId},
};

pub async fn response_body_string(response: axum::response::Response) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("utf-8 body")
}

pub fn sample_candidate_list(id: CandidateListId) -> CandidateList {
    CandidateList {
        id,
        electoral_districts: vec![ElectoralDistrict::UT],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn sample_person(id: PersonId) -> Person {
    Person {
        id,
        gender: Some(Gender::Female),
        last_name: "Jansen".to_string(),
        last_name_prefix: None,
        first_name: Some("Henk".to_string()),
        initials: "H.A.H.A.".to_string(),
        date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 2, 1).unwrap()),
        bsn: None,
        place_of_residence: Some("Juinen".to_string()),
        country_of_residence: Some("Netherlands".to_string()),
        locality: Some("Juinen".to_string()),
        postal_code: Some("1234 AB".to_string()),
        house_number: Some("10".to_string()),
        house_number_addition: Some("A".to_string()),
        street_name: Some("Stationsstraat".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn sample_person_with_last_name(id: PersonId, last_name: &str) -> Person {
    let sample = sample_person(id);

    Person {
        last_name: last_name.to_string(),
        ..sample
    }
}

pub fn sample_person_form(csrf_token: &TokenValue) -> PersonForm {
    PersonForm {
        gender: "male".to_string(),
        last_name: "Jansen".to_string(),
        last_name_prefix: "".to_string(),
        first_name: "Henk".to_string(),
        initials: "H.A.H.A.".to_string(),
        date_of_birth: "01-02-1990".to_string(),
        bsn: "".to_string(),
        place_of_residence: "Juinen".to_string(),
        country_of_residence: "Netherlands".to_string(),
        csrf_token: csrf_token.clone(),
    }
}

pub fn sample_address_form(csrf_token: &TokenValue) -> AddressForm {
    AddressForm {
        locality: "Juinen".to_string(),
        postal_code: "1234 AB".to_string(),
        house_number: "10".to_string(),
        house_number_addition: "A".to_string(),
        street_name: "Stationsstraat".to_string(),
        csrf_token: csrf_token.clone(),
    }
}
