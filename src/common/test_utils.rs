use chrono::{NaiveDate, Utc};
use http_body_util::BodyExt;

use crate::{
    ElectoralDistrict, TokenValue,
    candidate_lists::{CandidateList, CandidateListId},
    persons::{AddressForm, Gender, Person, PersonForm, PersonId},
    political_groups::{
        AuthorisedAgent, AuthorisedAgentForm, AuthorisedAgentId, ListSubmitter, ListSubmitterForm,
        ListSubmitterId, PoliticalGroup, PoliticalGroupForm, PoliticalGroupId,
    },
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
        country_of_residence: Some("NL".to_string()),
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
        country_of_residence: "NL".to_string(),
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

pub fn sample_political_group(id: PoliticalGroupId) -> PoliticalGroup {
    PoliticalGroup {
        id,
        long_list_allowed: Some(false),
        legal_name: Some("Kiesraad Demo Partij".to_string()),
        display_name: Some("Kiesraad Demo".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn sample_authorised_agent(id: AuthorisedAgentId) -> AuthorisedAgent {
    AuthorisedAgent {
        id,
        last_name: "Jansen".to_string(),
        last_name_prefix: Some("de".to_string()),
        initials: "A.B.".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn sample_authorised_agent_form(csrf_token: &TokenValue) -> AuthorisedAgentForm {
    AuthorisedAgentForm {
        last_name: "Jansen".to_string(),
        last_name_prefix: "de".to_string(),
        initials: "A.B.".to_string(),
        csrf_token: csrf_token.clone(),
    }
}

pub fn sample_list_submitter(id: ListSubmitterId) -> ListSubmitter {
    ListSubmitter {
        id,
        last_name: "Bos".to_string(),
        last_name_prefix: None,
        initials: "E.F.".to_string(),
        locality: Some("Rotterdam".to_string()),
        postal_code: Some("3011 CC".to_string()),
        house_number: Some("5".to_string()),
        house_number_addition: Some("B".to_string()),
        street_name: Some("Coolsingel".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

pub fn sample_list_submitter_form(csrf_token: &TokenValue) -> ListSubmitterForm {
    ListSubmitterForm {
        last_name: "Bos".to_string(),
        last_name_prefix: "".to_string(),
        initials: "E.F.".to_string(),
        locality: "Rotterdam".to_string(),
        postal_code: "3011 CC".to_string(),
        house_number: "5".to_string(),
        house_number_addition: "B".to_string(),
        street_name: "Coolsingel".to_string(),
        csrf_token: csrf_token.clone(),
    }
}

pub fn sample_political_group_form(csrf_token: &TokenValue) -> PoliticalGroupForm {
    PoliticalGroupForm {
        long_list_allowed: "true".to_string(),
        legal_name: "Updated Legal Name".to_string(),
        display_name: "Updated Display Name".to_string(),
        csrf_token: csrf_token.clone(),
    }
}
