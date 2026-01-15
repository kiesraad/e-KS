use chrono::{NaiveDate, Utc};
use http_body_util::BodyExt;
use uuid::Uuid;

use crate::{
    ElectoralDistrict, TokenValue,
    candidate_lists::CandidateList,
    persons::{AddressForm, Gender, Person, PersonForm},
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

pub fn sample_candidate_list(id: Uuid) -> CandidateList {
    CandidateList {
        id,
        electoral_districts: vec![ElectoralDistrict::UT],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn sample_person(id: Uuid) -> Person {
    Person {
        id,
        gender: Some(Gender::Female),
        last_name: "Jansen".to_string(),
        last_name_prefix: None,
        first_name: Some("Henk".to_string()),
        initials: "H.A.H.A.".to_string(),
        date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 2, 1).unwrap()),
        bsn: None,
        locality: Some("Juinen".to_string()),
        postal_code: Some("1234 AB".to_string()),
        house_number: Some("10".to_string()),
        house_number_addition: Some("A".to_string()),
        street_name: Some("Stationsstraat".to_string()),
        is_dutch: Some(true),
        custom_country: None,
        custom_region: None,
        address_line_1: None,
        address_line_2: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn sample_person_with_last_name(id: Uuid, last_name: &str) -> Person {
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
        custom_country: "".to_string(),
        custom_region: "".to_string(),
        address_line_1: "".to_string(),
        address_line_2: "".to_string(),
        is_dutch: "true".to_string(),
        csrf_token: csrf_token.clone(),
    }
}
