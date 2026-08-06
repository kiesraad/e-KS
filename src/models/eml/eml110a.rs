//! The EML Election Definition (`110a`) export, built with [`eml_nl`].

use std::num::NonZeroU64;

use eml_nl::{
    common::{ContestIdentifier, ElectionTree, Region},
    documents::{
        EML, ElectionIdentifierBuilder,
        election_definition::{ElectionDefinition, ElectionDefinitionRegisteredParty},
    },
    io::EMLWrite,
    utils::{RegionCategory, VotingMethod},
};

use crate::{AppError, ElectionConfig, ElectoralDistrict};

fn specific_election_tree(
    root: Region,
    districts: &'static [ElectoralDistrict],
    district_level: RegionCategory,
    sub_level: RegionCategory,
) -> ElectionTree {
    let mut district_regions = Vec::new();
    let mut subdistrict_regions = Vec::new();

    for district in districts {
        let region = Region::new(district.title(), district_level)
            .with_number(district.region_number())
            .with_roman_numerals(district.roman_numerals())
            .with_frysian_export_allowed(district.frisian_export_allowed())
            .with_superior_region_key(root.key);

        for sub in district.sub_districts() {
            subdistrict_regions.push(
                Region::new(sub.title(), sub_level)
                    .with_number(sub.region_number())
                    .with_frysian_export_allowed(sub.frisian_export_allowed())
                    .with_superior_region_key(region.key),
            )
        }
        district_regions.push(region);
    }

    let mut regions = vec![root];
    regions.extend(district_regions);
    regions.extend(subdistrict_regions);
    ElectionTree::new(regions)
}

fn build_election_tree(election: &ElectionConfig) -> ElectionTree {
    match *election {
        ElectionConfig::EK27 => {
            specific_election_tree(
                Region::new("Nederland", RegionCategory::State),
                crate::ElectoralDistrict::ek_districts(),
                RegionCategory::Province, // all EK districts, including kiescolleges, are output as PROVINCIE
                RegionCategory::PollingStation,
            )
        }
        ElectionConfig::PS27(province) => specific_election_tree(
            Region::new(province.title(), RegionCategory::Province)
                .with_number(province.region_number())
                .with_frysian_export_allowed(province.frisian_export_allowed()),
            province.ps_districts(),
            RegionCategory::ElectoralDistrict,
            RegionCategory::Municipality,
        ),
        ElectionConfig::WS27(water_council) => specific_election_tree(
            Region::new(water_council.title(), RegionCategory::WaterAuthority)
                .with_number(water_council.region_number())
                .with_frysian_export_allowed(water_council.frisian_export_allowed()),
            water_council.ws_districts(),
            RegionCategory::ElectoralDistrict,
            RegionCategory::Municipality,
        ),
    }
}

/// Build the EML 110a election definition XML for the given election and
/// the list of registered party names.
pub fn eml110a(
    election: &ElectionConfig,
    registered_party_names: Vec<String>,
) -> Result<Vec<u8>, AppError> {
    let contest_identifier = if election.has_only_one_district() {
        ContestIdentifier::geen()
    } else {
        ContestIdentifier::alle()
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
        .election_tree(build_election_tree(election))
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
            &ElectionConfig::PS27(Province::Groningen),
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
            &ElectionConfig::PS27(Province::Limburg),
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
