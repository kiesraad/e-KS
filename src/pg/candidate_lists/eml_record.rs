//! Mapping the candidates of an EML 2.10 nomination onto the shared
//! [`CandidateRecord`], so an EML import runs exactly the same validation as a
//! CSV import.
//!
//! The export ([`crate::models::eml210`]) flattens a candidate's address into a
//! single `AddressLine`; [`split_address_line`] takes it apart again.

use eml_nl::{
    common::PersonNameStructure,
    documents::nomination::{NominationAgent, NominationCandidate, NominationContact},
    utils::{Gender, StringValue, XsDate},
};

use crate::{
    candidate_lists::CandidateRecord,
    common::{DutchAddressForm, FullNameForm, MinimalNameForm},
    constants::DEFAULT_DATE_FORMAT,
    persons::{PersonalDataFieldsForm, RepresentativeForm},
};

/// An empty country code defaults to NL, matching the CSV import: the export
/// omits the country for Dutch candidates.
const DEFAULT_COUNTRY: &str = "NL";

impl From<&NominationCandidate> for CandidateRecord {
    fn from(candidate: &NominationCandidate) -> Self {
        let representative = candidate.agent.as_ref().map(representative_form);

        CandidateRecord {
            name: full_name_form(&candidate.full_name),
            personal_data: PersonalDataFieldsForm {
                gender: gender(&candidate.gender),
                date_of_birth: date_of_birth(candidate.date_of_birth.as_ref()),
                bsn: candidate
                    .national_identification_number
                    .as_deref()
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
                place_of_residence: candidate
                    .qualifying_address
                    .locality()
                    .locality_name()
                    .to_string(),
                country: match candidate.qualifying_address.country_name_code() {
                    Some(code) => code.value.to_string(),
                    None => DEFAULT_COUNTRY.to_string(),
                },
            },
            address: address_form(candidate.contact.as_ref()),
            representative: representative.filter(|form| !form.is_empty()),
        }
    }
}

fn full_name_form(name: &PersonNameStructure) -> FullNameForm {
    let name = &name.person_name;

    FullNameForm {
        first_name: optional_value(name.first_name.as_ref().map(|n| n.value.as_ref())),
        last_name: name.last_name.value.to_string(),
        last_name_prefix: optional_value(name.name_prefix.as_ref().map(|n| n.value.as_ref())),
        initials: optional_value(name.name_line_initials.as_ref().map(|n| n.value.as_ref())),
    }
}

fn minimal_name_form(name: &PersonNameStructure) -> MinimalNameForm {
    let name = &name.person_name;

    MinimalNameForm {
        last_name: name.last_name.value.to_string(),
        last_name_prefix: optional_value(name.name_prefix.as_ref().map(|n| n.value.as_ref())),
        initials: optional_value(name.name_line_initials.as_ref().map(|n| n.value.as_ref())),
    }
}

/// The agent (`gemachtigde`) carries the correspondence address in its contact;
/// fall back to the living address, which only holds a locality.
fn representative_form(agent: &NominationAgent) -> RepresentativeForm {
    let mut address = address_form(agent.contact.as_ref());

    if address.locality.trim().is_empty() {
        address.locality = agent.living_address.locality_name.to_string();
    }

    RepresentativeForm {
        name: minimal_name_form(&agent.agent_identifier.agent_name),
        address,
    }
}

fn address_form(contact: Option<&NominationContact>) -> DutchAddressForm {
    let Some(locality) = contact.map(|contact| contact.mailing_address.address.locality()) else {
        return DutchAddressForm::default();
    };

    let address_line = locality
        .address_line
        .as_ref()
        .map(|line| line.value.as_ref())
        .unwrap_or_default();
    let (street_name, house_number, house_number_addition) = split_address_line(address_line);

    DutchAddressForm {
        locality: locality.locality_name().to_string(),
        postal_code: locality
            .postal_code
            .as_ref()
            .map(|code| code.postal_code_number.value.to_string())
            .unwrap_or_default(),
        house_number,
        house_number_addition,
        street_name,
    }
}

/// Unknown is how the export writes "no gender given"; an empty value keeps the
/// optional gender field empty instead of failing validation.
fn gender(gender: &StringValue<Gender>) -> String {
    match gender.raw().as_ref() {
        "unknown" => String::new(),
        raw => raw.to_string(),
    }
}

/// A date that the EML reader could not parse is passed on as-is, so the record
/// validation reports it as a field error like any other bad value.
fn date_of_birth(date_of_birth: Option<&StringValue<XsDate>>) -> String {
    match date_of_birth {
        None => String::new(),
        Some(value) => match value.cloned_value_err() {
            Ok(date) => date.date.format(DEFAULT_DATE_FORMAT).to_string(),
            Err(_) => value.raw().to_string(),
        },
    }
}

fn optional_value(value: Option<&str>) -> String {
    value.unwrap_or_default().to_string()
}

/// Split an address line ("Stationsstraat 10A") into street name, house number
/// and house number addition. The last word starting with a digit is the house
/// number; whatever follows it is the addition. Without such a word the whole
/// line is kept as the street name.
fn split_address_line(line: &str) -> (String, String, String) {
    let words: Vec<&str> = line.split_whitespace().collect();
    let house_number_index = words
        .iter()
        .rposition(|word| word.starts_with(|c: char| c.is_ascii_digit()))
        .filter(|&index| index > 0);

    let Some(index) = house_number_index else {
        return (words.join(" "), String::new(), String::new());
    };

    let digits = words[index]
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(words[index].len());
    let (house_number, addition) = words[index].split_at(digits);
    let addition = std::iter::once(addition)
        .chain(words[index + 1..].iter().copied())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    (words[..index].join(" "), house_number.to_string(), addition)
}

#[cfg(test)]
mod tests {
    use eml_nl::{
        documents::EML,
        io::{EMLParsingMode, EMLRead},
    };

    use super::*;
    use crate::{structs::common::BsnOrNoneConfirmed, test_utils::display_opt};

    const NOMINATION: &str = include_str!("testdata/nomination.eml.xml");

    fn candidates() -> Vec<CandidateRecord> {
        let document =
            EML::parse_eml(NOMINATION, EMLParsingMode::StrictFallback).expect("nomination parses");

        document
            .as_nomination_doc()
            .expect("is a nomination")
            .nomination_data
            .affiliation
            .candidates
            .iter()
            .map(CandidateRecord::from)
            .collect()
    }

    #[test]
    fn maps_a_dutch_candidate_onto_a_record() {
        let person = candidates()[0]
            .clone()
            .validate_create()
            .expect("candidate is valid");

        assert_eq!(person.name.initials.to_string(), "H.A.H.A.");
        assert_eq!(
            display_opt(&person.name.first_name).as_deref(),
            Some("Henk")
        );
        assert_eq!(person.name.last_name.to_string(), "Candidate I");
        assert_eq!(
            person
                .personal_data
                .date_of_birth
                .map(|date| date.format(DEFAULT_DATE_FORMAT).to_string()),
            Some("01-02-1990".to_string())
        );
        assert_eq!(
            person.personal_data.gender,
            Some(crate::structs::common::Gender::Female)
        );
        assert_eq!(
            display_opt(&person.personal_data.place_of_residence).as_deref(),
            Some("Juinen")
        );
        // The export omits the country for Dutch candidates.
        assert_eq!(
            display_opt(&person.personal_data.country).as_deref(),
            Some("NL")
        );
        // The single address line is split back into its parts.
        assert_eq!(
            display_opt(&person.address.street_name).as_deref(),
            Some("Stationsstraat")
        );
        assert_eq!(
            display_opt(&person.address.house_number).as_deref(),
            Some("10")
        );
        assert_eq!(
            display_opt(&person.address.house_number_addition).as_deref(),
            Some("A")
        );
        assert_eq!(
            display_opt(&person.address.postal_code).as_deref(),
            Some("1234AB")
        );
        assert_eq!(
            display_opt(&person.address.locality).as_deref(),
            Some("Juinen")
        );
        assert_eq!(person.representative, None);
        // EML has no equivalent of the "candidate has no BSN" confirmation.
        assert_eq!(person.personal_data.bsn, None);
    }

    #[test]
    fn maps_a_foreign_candidate_with_an_agent_onto_a_record() {
        let person = candidates()[1]
            .clone()
            .validate_create()
            .expect("candidate is valid");

        assert_eq!(
            display_opt(&person.personal_data.country).as_deref(),
            Some("BE")
        );
        assert_eq!(person.personal_data.gender, None);
        assert_eq!(
            person
                .personal_data
                .bsn
                .as_ref()
                .map(BsnOrNoneConfirmed::to_exposed_string),
            Some("999995972".to_string())
        );

        let representative = person.representative.expect("representative is present");

        assert_eq!(representative.name.initials.to_string(), "B.");
        assert_eq!(
            display_opt(&representative.name.last_name_prefix).as_deref(),
            Some("de")
        );
        assert_eq!(representative.name.last_name.to_string(), "Bouwer");
        assert_eq!(
            display_opt(&representative.address.street_name).as_deref(),
            Some("Bouwstraat")
        );
        assert_eq!(
            display_opt(&representative.address.house_number).as_deref(),
            Some("22")
        );
        assert_eq!(
            display_opt(&representative.address.house_number_addition).as_deref(),
            Some("c")
        );
        assert_eq!(
            display_opt(&representative.address.locality).as_deref(),
            Some("Nijmegen")
        );
    }

    #[test]
    fn split_address_line_takes_the_last_word_starting_with_a_digit() {
        assert_eq!(
            split_address_line("Stationsstraat 10A"),
            (
                "Stationsstraat".to_string(),
                "10".to_string(),
                "A".to_string()
            )
        );
        assert_eq!(
            split_address_line("Dam 1"),
            ("Dam".to_string(), "1".to_string(), String::new())
        );
        // A street name may start with a digit itself.
        assert_eq!(
            split_address_line("2e Kruisweg 5"),
            ("2e Kruisweg".to_string(), "5".to_string(), String::new())
        );
        // A house number may be followed by a separate addition.
        assert_eq!(
            split_address_line("Kerkstraat 12 bis"),
            (
                "Kerkstraat".to_string(),
                "12".to_string(),
                "bis".to_string()
            )
        );
        // Digits in the street name do not become the house number.
        assert_eq!(
            split_address_line("Laan 1940-1945 12"),
            (
                "Laan 1940-1945".to_string(),
                "12".to_string(),
                String::new()
            )
        );
        // Without a house number the whole line is the street name.
        assert_eq!(
            split_address_line("  Lange   Voorhout "),
            ("Lange Voorhout".to_string(), String::new(), String::new())
        );
        assert_eq!(
            split_address_line(""),
            (String::new(), String::new(), String::new())
        );
    }
}
