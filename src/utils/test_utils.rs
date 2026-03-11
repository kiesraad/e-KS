//! Test helpers for building sample domain data and reading responses.
use http_body_util::BodyExt;

use crate::{
    ElectoralDistrict, PoliticalGroupId, TokenValue,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentForm, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    common::{
        BsnOrNoneConfirmed, CountryCode, Date, DisplayName, DutchAddress, DutchAddressForm,
        FirstName, FullName, FullNameForm, Gender, HouseNumber, HouseNumberAddition, Initials,
        LastName, LastNamePrefix, LegalName, Locality, PlaceOfResidence, PostalCode, StreetName,
    },
    list_submitters::{ListSubmitter, ListSubmitterForm, ListSubmitterId},
    persons::{
        AddressForm, Person, PersonId, PersonalDataForm, Representative, RepresentativeForm,
    },
    political_groups::{PoliticalGroup, PoliticalGroupForm},
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

pub fn extract_csrf_token(body: &str) -> Option<TokenValue> {
    let marker = "name=\"csrf_token\" value=\"";
    body.split(marker)
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .map(|token| TokenValue(token.to_string()))
}

pub fn sample_full_name(
    last_name: &str,
    last_name_prefix: Option<&str>,
    initials: &str,
) -> FullName {
    FullName {
        last_name: parse_last_name(last_name),
        last_name_prefix: last_name_prefix.map(parse_last_name_prefix),
        initials: parse_initials(initials),
    }
}

pub fn parse_last_name(value: &str) -> LastName {
    value.parse::<LastName>().expect("last name")
}

pub fn parse_last_name_prefix(value: &str) -> LastNamePrefix {
    value.parse::<LastNamePrefix>().expect("last name prefix")
}

pub fn parse_initials(value: &str) -> Initials {
    value.parse::<Initials>().expect("initials")
}

pub fn parse_first_name(value: &str) -> FirstName {
    value.parse::<FirstName>().expect("first name")
}

pub fn parse_place_of_residence(value: &str) -> PlaceOfResidence {
    value
        .parse::<PlaceOfResidence>()
        .expect("place of residence")
}

pub fn parse_country_code(value: &str) -> CountryCode {
    value.parse::<CountryCode>().expect("country code")
}

fn sample_full_name_form(last_name: &str, last_name_prefix: &str, initials: &str) -> FullNameForm {
    FullNameForm {
        last_name: last_name.to_string(),
        last_name_prefix: last_name_prefix.to_string(),
        initials: initials.to_string(),
    }
}

fn sample_dutch_address(
    locality: &str,
    postal_code: &str,
    house_number: &str,
    house_number_addition: &str,
    street_name: &str,
) -> DutchAddress {
    DutchAddress {
        locality: Some(locality.parse::<Locality>().expect("locality")),
        postal_code: Some(postal_code.parse::<PostalCode>().expect("postal code")),
        house_number: Some(house_number.parse::<HouseNumber>().expect("house number")),
        house_number_addition: Some(
            house_number_addition
                .parse::<HouseNumberAddition>()
                .expect("house number addition"),
        ),
        street_name: Some(street_name.parse::<StreetName>().expect("street name")),
    }
}

fn sample_dutch_address_form(
    locality: &str,
    postal_code: &str,
    house_number: &str,
    house_number_addition: &str,
    street_name: &str,
) -> DutchAddressForm {
    DutchAddressForm {
        locality: locality.to_string(),
        postal_code: postal_code.to_string(),
        house_number: house_number.to_string(),
        house_number_addition: house_number_addition.to_string(),
        street_name: street_name.to_string(),
    }
}

pub fn sample_candidate_list(id: CandidateListId) -> CandidateList {
    CandidateList {
        id,
        electoral_districts: vec![ElectoralDistrict::UT],
        ..Default::default()
    }
}

pub fn sample_person(id: PersonId) -> Person {
    Person {
        id,
        name: sample_full_name("Jansen", None, "H.A.H.A."),
        personal_data: crate::persons::PersonalData {
            gender: Some(Gender::Female),
            first_name: Some(parse_first_name("Henk")),
            date_of_birth: Some("01-02-1990".parse::<Date>().unwrap()),
            bsn: Some(BsnOrNoneConfirmed::NoneConfirmed),
            place_of_residence: Some(parse_place_of_residence("Juinen")),
            country_of_residence: Some(parse_country_code("NL")),
        },
        address: sample_dutch_address("Juinen", "1234 AB", "10", "A", "Stationsstraat"),
        representative: Representative::default(),
        ..Default::default()
    }
}

pub fn sample_person_with_last_name(id: PersonId, last_name: &str) -> Person {
    sample_person_with(id, last_name, None, "H.A.H.A.")
}

pub fn sample_person_with(
    id: PersonId,
    last_name: &str,
    last_name_prefix: Option<&str>,
    initials: &str,
) -> Person {
    let mut person = sample_person(id);
    person.name = sample_full_name(last_name, last_name_prefix, initials);
    person
}

pub fn sample_person_form(csrf_token: &TokenValue) -> PersonalDataForm {
    PersonalDataForm {
        name: sample_full_name_form("Jansen", "", "H.A.H.A."),
        personal_data: crate::persons::PersonalDataFieldsForm {
            gender: "male".to_string(),
            first_name: "Henk".to_string(),
            date_of_birth: "01-02-1990".to_string(),
            bsn: "none-confirmed".to_string(),
            place_of_residence: "Juinen".to_string(),
            country_of_residence: "NL".to_string(),
        },
        csrf_token: csrf_token.clone(),
    }
}

pub fn sample_address_form(csrf_token: &TokenValue) -> AddressForm {
    AddressForm {
        address: sample_dutch_address_form("Juinen", "1234 AB", "10", "A", "Stationsstraat"),
        csrf_token: csrf_token.clone(),
    }
}

pub fn sample_representative_form(csrf_token: &TokenValue) -> RepresentativeForm {
    RepresentativeForm {
        name: sample_full_name_form("Bakker", "", "A.B."),
        address: sample_dutch_address_form("Juinen", "1234 AB", "10", "A", "Stationsstraat"),
        csrf_token: csrf_token.clone(),
    }
}

pub fn sample_political_group(id: PoliticalGroupId) -> PoliticalGroup {
    PoliticalGroup {
        id,
        long_list_allowed: Some(false),
        legal_name: Some(
            "Kiesraad Demo Partij"
                .parse::<LegalName>()
                .expect("legal name"),
        ),
        display_name: Some(
            "Kiesraad Demo"
                .parse::<DisplayName>()
                .expect("display name"),
        ),
    }
}

pub fn sample_authorised_agent(id: AuthorisedAgentId) -> AuthorisedAgent {
    AuthorisedAgent {
        id,
        name: sample_full_name("Jansen", Some("de"), "A.B."),
    }
}

pub fn sample_authorised_agent_form(csrf_token: &TokenValue) -> AuthorisedAgentForm {
    AuthorisedAgentForm {
        name: sample_full_name_form("Jansen", "de", "A.B."),
        csrf_token: csrf_token.clone(),
    }
}

pub fn sample_list_submitter(id: ListSubmitterId) -> ListSubmitter {
    ListSubmitter {
        id,
        name: sample_full_name("Bos", None, "E.F."),
        address: sample_dutch_address("Rotterdam", "3011 CC", "5", "B", "Coolsingel"),
    }
}

pub fn sample_list_submitter_form(csrf_token: &TokenValue) -> ListSubmitterForm {
    ListSubmitterForm {
        name: sample_full_name_form("Bos", "", "E.F."),
        address: sample_dutch_address_form("Rotterdam", "3011 CC", "5", "B", "Coolsingel"),
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
