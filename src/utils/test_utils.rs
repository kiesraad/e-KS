use http_body_util::BodyExt;

use crate::{
    ElectoralDistrict, TokenValue,
    authorised_agents::{AuthorisedAgent, AuthorisedAgentForm, AuthorisedAgentId},
    candidate_lists::{CandidateList, CandidateListId},
    common::{
        CountryCode, Date, DisplayName, DutchAddress, DutchAddressForm, FirstName, FullName,
        FullNameForm, Gender, HouseNumber, HouseNumberAddition, Initials, LastName, LastNamePrefix,
        LegalName, Locality, PlaceOfResidence, PostalCode, StreetName,
    },
    list_submitters::{ListSubmitter, ListSubmitterForm, ListSubmitterId},
    persons::{AddressForm, Person, PersonForm, PersonId, Representative, RepresentativeForm},
    political_groups::{PoliticalGroup, PoliticalGroupForm, PoliticalGroupId},
    substitute_list_submitters::{
        SubstituteSubmitter, SubstituteSubmitterForm, SubstituteSubmitterId,
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

pub fn extract_csrf_token(body: &str) -> Option<TokenValue> {
    let marker = "name=\"csrf_token\" value=\"";
    body.split(marker)
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .map(|token| TokenValue(token.to_string()))
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
        gender: Some(Gender::Female),
        name: FullName {
            last_name: "Jansen".parse::<LastName>().expect("last name"),
            last_name_prefix: None,
            initials: "H.A.H.A.".parse::<Initials>().expect("initials"),
        },
        first_name: Some("Henk".parse::<FirstName>().expect("first name")),
        date_of_birth: Some("01-02-1990".parse::<Date>().unwrap()),
        bsn: None,
        no_bsn_confirmed: false,
        place_of_residence: Some(
            "Juinen"
                .parse::<PlaceOfResidence>()
                .expect("place of residence"),
        ),
        country_of_residence: Some("NL".parse::<CountryCode>().expect("country code")),
        address: DutchAddress {
            locality: Some("Juinen".parse::<Locality>().expect("locality")),
            postal_code: Some("1234 AB".parse::<PostalCode>().expect("postal code")),
            house_number: Some("10".parse::<HouseNumber>().expect("house number")),
            house_number_addition: Some(
                "A".parse::<HouseNumberAddition>()
                    .expect("house number addition"),
            ),
            street_name: Some("Stationsstraat".parse::<StreetName>().expect("street name")),
        },
        representative: Representative::default(),
        ..Default::default()
    }
}

pub fn sample_person_with_last_name(id: PersonId, last_name: &str) -> Person {
    let mut sample = sample_person(id);
    sample.name.last_name = last_name.parse::<LastName>().expect("last name");
    sample
}

pub fn sample_person_form(csrf_token: &TokenValue) -> PersonForm {
    PersonForm {
        gender: "male".to_string(),
        name: FullNameForm {
            last_name: "Jansen".to_string(),
            last_name_prefix: "".to_string(),
            initials: "H.A.H.A.".to_string(),
        },
        first_name: "Henk".to_string(),
        date_of_birth: "01-02-1990".to_string(),
        bsn: "".to_string(),
        no_bsn_confirmed: false,
        place_of_residence: "Juinen".to_string(),
        country_of_residence: "NL".to_string(),
        csrf_token: csrf_token.clone(),
    }
}

pub fn sample_address_form(csrf_token: &TokenValue) -> AddressForm {
    AddressForm {
        address: DutchAddressForm {
            locality: "Juinen".to_string(),
            postal_code: "1234 AB".to_string(),
            house_number: "10".to_string(),
            house_number_addition: "A".to_string(),
            street_name: "Stationsstraat".to_string(),
        },
        csrf_token: csrf_token.clone(),
    }
}

pub fn sample_representative_form(csrf_token: &TokenValue) -> RepresentativeForm {
    RepresentativeForm {
        name: FullNameForm {
            last_name: "Bakker".to_string(),
            last_name_prefix: "".to_string(),
            initials: "A.B.".to_string(),
        },
        address: DutchAddressForm {
            locality: "Juinen".to_string(),
            postal_code: "1234 AB".to_string(),
            house_number: "10".to_string(),
            house_number_addition: "A".to_string(),
            street_name: "Stationsstraat".to_string(),
        },
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
        name: FullName {
            last_name: "Jansen".parse::<LastName>().expect("last name"),
            last_name_prefix: Some("de".parse::<LastNamePrefix>().expect("last name prefix")),
            initials: "A.B.".parse::<Initials>().expect("initials"),
        },
    }
}

pub fn sample_authorised_agent_form(csrf_token: &TokenValue) -> AuthorisedAgentForm {
    AuthorisedAgentForm {
        name: FullNameForm {
            last_name: "Jansen".to_string(),
            last_name_prefix: "de".to_string(),
            initials: "A.B.".to_string(),
        },
        csrf_token: csrf_token.clone(),
    }
}

pub fn sample_list_submitter(id: ListSubmitterId) -> ListSubmitter {
    ListSubmitter {
        id,
        name: FullName {
            last_name: "Bos".parse::<LastName>().expect("last name"),
            last_name_prefix: None,
            initials: "E.F.".parse::<Initials>().expect("initials"),
        },
        address: DutchAddress {
            locality: Some("Rotterdam".parse::<Locality>().expect("locality")),
            postal_code: Some("3011 CC".parse::<PostalCode>().expect("postal code")),
            house_number: Some("5".parse::<HouseNumber>().expect("house number")),
            house_number_addition: Some(
                "B".parse::<HouseNumberAddition>()
                    .expect("house number addition"),
            ),
            street_name: Some("Coolsingel".parse::<StreetName>().expect("street name")),
        },
    }
}

pub fn sample_list_submitter_form(csrf_token: &TokenValue) -> ListSubmitterForm {
    ListSubmitterForm {
        name: FullNameForm {
            last_name: "Bos".to_string(),
            last_name_prefix: "".to_string(),
            initials: "E.F.".to_string(),
        },
        address: DutchAddressForm {
            locality: "Rotterdam".to_string(),
            postal_code: "3011 CC".to_string(),
            house_number: "5".to_string(),
            house_number_addition: "B".to_string(),
            street_name: "Coolsingel".to_string(),
        },
        csrf_token: csrf_token.clone(),
    }
}

pub fn sample_substitute_submitter(id: SubstituteSubmitterId) -> SubstituteSubmitter {
    SubstituteSubmitter {
        id,
        name: FullName {
            last_name: "Bakker".parse::<LastName>().expect("last name"),
            last_name_prefix: None,
            initials: "I.J.".parse::<Initials>().expect("initials"),
        },
        address: DutchAddress {
            locality: Some("Utrecht".parse::<Locality>().expect("locality")),
            postal_code: Some("3511 AA".parse::<PostalCode>().expect("postal code")),
            house_number: Some("21".parse::<HouseNumber>().expect("house number")),
            house_number_addition: Some(
                "C".parse::<HouseNumberAddition>()
                    .expect("house number addition"),
            ),
            street_name: Some("Oudegracht".parse::<StreetName>().expect("street name")),
        },
    }
}

pub fn sample_substitute_submitter_form(csrf_token: &TokenValue) -> SubstituteSubmitterForm {
    SubstituteSubmitterForm {
        name: FullNameForm {
            last_name: "Bakker".to_string(),
            last_name_prefix: "".to_string(),
            initials: "I.J.".to_string(),
        },
        address: DutchAddressForm {
            locality: "Utrecht".to_string(),
            postal_code: "3511 AA".to_string(),
            house_number: "21".to_string(),
            house_number_addition: "C".to_string(),
            street_name: "Oudegracht".to_string(),
        },
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
