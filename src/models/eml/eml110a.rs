//! The EML Election Definition (`110a`) export, built with [`eml_nl`].

use std::num::NonZeroU64;

use eml_nl::{
    common::ContestIdentifier,
    documents::{
        EML, ElectionIdentifierBuilder,
        election_definition::{ElectionDefinition, ElectionDefinitionRegisteredParty},
    },
    io::EMLWrite,
    utils::{ElectionSubcategory, VotingMethod},
};

use crate::{AppError, ElectionConfig};

/// Build the EML 110a election definition XML for the given election and
/// the list of registered party names.
pub fn eml110a(
    election: &ElectionConfig,
    registered_party_names: Vec<String>,
) -> Result<Vec<u8>, AppError> {
    let subcategory = eml_nl::utils::ElectionSubcategory::from(election);

    let contest_identifier = if election.has_only_one_district() {
        ContestIdentifier::geen()
    } else {
        ContestIdentifier::alle()
    };

    // The "voorkeursdrempel", see Kieswet P 15
    let preference_threshold: u64 = if subcategory == ElectionSubcategory::GR1 {
        50
    } else {
        25
    };

    let now = chrono::Utc::now();
    let definition = ElectionDefinition::builder()
        .transaction_id(1)
        .issue_date(now.date_naive())
        .creation_date_time(now)
        .election_identifier(
            ElectionIdentifierBuilder::try_from(*election)?.build_for_definition()?,
        )
        .contest_identifier(contest_identifier)
        .voting_method(VotingMethod::SPV)
        .max_votes(NonZeroU64::new(1).expect("1 is non-zero")) // 1 is the default max votes => always empty
        .number_of_seats(election.number_of_seats())
        .preference_threshold(preference_threshold)
        .registered_parties(
            registered_party_names
                .into_iter()
                .map(ElectionDefinitionRegisteredParty::new)
                .collect::<Vec<_>>(),
        )
        .build()?;

    Ok(EML::from_election_definition_doc(definition).write_eml_root(true, true)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ElectionConfig, Province, WaterCouncil};

    fn check_eml(response: &str, expected: &str) {
        let stringify_election_event = |eml: EML| {
            format!(
                "{:?}",
                eml.as_election_definition_doc().unwrap().election_event
            )
        };

        let received = stringify_election_event(response.parse().unwrap());
        let expected = stringify_election_event(expected.parse().unwrap());

        assert_eq!(received, expected, "received XML:\n{}", response);
    }

    #[test]
    fn ek_export() {
        let eml = eml110a(
            &ElectionConfig::EK27,
            vec!["Kiesraad Demo".to_string(), "Andere Partij".to_string()],
        )
        .unwrap();
        check_eml(
            &String::from_utf8(eml).unwrap(),
            include_str!("testdata/110a-ek27.eml.xml"),
        );
    }

    #[test]
    fn ps1_export() {
        let eml = eml110a(
            &ElectionConfig::PS27(Province::GR),
            vec!["Kiesraad Demo".to_string(), "Andere Partij".to_string()],
        )
        .unwrap();
        check_eml(
            &String::from_utf8(eml).unwrap(),
            include_str!("testdata/110a-ps27-1.eml.xml"),
        );
    }

    #[test]
    fn ps2_export() {
        let eml = eml110a(
            &ElectionConfig::PS27(Province::LI),
            vec!["Kiesraad Demo".to_string(), "Andere Partij".to_string()],
        )
        .unwrap();
        check_eml(
            &String::from_utf8(eml).unwrap(),
            include_str!("testdata/110a-ps27-2.eml.xml"),
        );
    }

    #[test]
    fn ws_export() {
        let eml = eml110a(
            &ElectionConfig::WS27(WaterCouncil::Fryslan),
            vec!["Water Water".to_string()],
        )
        .unwrap();
        check_eml(
            &String::from_utf8(eml).unwrap(),
            include_str!("testdata/110a-ws27.eml.xml"),
        );
    }
}
