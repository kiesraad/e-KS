use chrono::{NaiveDate, Utc};
use http_body_util::BodyExt;
use uuid::Uuid;

use crate::{
    ElectoralDistrict,
    candidate_lists::structs::CandidateList,
    persons::structs::{Gender, Person},
};

pub(crate) async fn response_body_string(response: axum::response::Response) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("utf-8 body")
}

pub(crate) fn sample_candidate_list(id: Uuid) -> CandidateList {
    CandidateList {
        id,
        electoral_districts: vec![ElectoralDistrict::UT],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub(crate) fn sample_person(id: Uuid) -> Person {
    Person {
        id,
        gender: Some(Gender::Female),
        last_name: "Jansen".to_string(),
        last_name_prefix: None,
        first_name: Some("Henk".to_string()),
        initials: "H.H.".to_string(),
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

pub(crate) fn sample_person_with_last_name(id: Uuid, last_name: &str) -> Person {
    let sample = sample_person(id);

    Person {
        last_name: last_name.to_string(),
        ..sample
    }
}
