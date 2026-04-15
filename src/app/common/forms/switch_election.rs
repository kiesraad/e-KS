use serde::Deserialize;

use crate::ElectionConfig;

/// Separate fields per region type so the form deserializes cleanly even when
/// JavaScript is disabled and every region picker submits a value.
#[derive(Deserialize)]
pub struct SwitchElectionForm {
    pub csrf_token: String,
    election: String,
    region_province: Option<String>,
    region_water_council: Option<String>,
}

impl SwitchElectionForm {
    pub fn into_election_config(self) -> Option<ElectionConfig> {
        // Try each submitted region in turn; only the one whose code matches
        // the election's region type produces a valid config. Falls back to
        // `None` for region-less elections.
        [
            self.region_province.as_deref(),
            self.region_water_council.as_deref(),
        ]
        .into_iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .find_map(|r| ElectionConfig::from_code_and_region(&self.election, Some(r)))
        .or_else(|| ElectionConfig::from_code_and_region(&self.election, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Province, WaterCouncil};

    fn parse(body: &str) -> SwitchElectionForm {
        serde_urlencoded::from_str(body).expect("form body")
    }

    #[test]
    fn deserializes_with_optional_region_fields_absent() {
        let form = parse("csrf_token=abc&election=EK27");
        assert_eq!(form.csrf_token, "abc");
        assert_eq!(form.into_election_config(), Some(ElectionConfig::EK27));
    }

    #[test]
    fn ps27_uses_region_province() {
        let form = parse("csrf_token=t&election=PS27&region_province=GR");
        assert_eq!(
            form.into_election_config(),
            Some(ElectionConfig::PS27(Province::GR))
        );
    }

    #[test]
    fn ws27_uses_region_water_council() {
        let form = parse("csrf_token=t&election=WS27&region_water_council=WS-FRY");
        assert_eq!(
            form.into_election_config(),
            Some(ElectionConfig::WS27(WaterCouncil::Fryslan))
        );
    }

    #[test]
    fn ek27_ignores_submitted_region_fields() {
        // When JS is disabled, every region picker submits a value. The form
        // should still resolve to EK27 because EK27 has no region.
        let form =
            parse("csrf_token=t&election=EK27&region_province=GR&region_water_council=WS-FRY");
        assert_eq!(form.into_election_config(), Some(ElectionConfig::EK27));
    }

    #[test]
    fn ps27_ignores_unrelated_water_council_field() {
        // The province field is empty (placeholder option) but the water
        // council field is filled — it must not satisfy a PS27 election.
        let form = parse("csrf_token=t&election=PS27&region_province=&region_water_council=WS-FRY");
        assert_eq!(form.into_election_config(), None);
    }

    #[test]
    fn ps27_with_empty_region_returns_none() {
        let form = parse("csrf_token=t&election=PS27&region_province=");
        assert_eq!(form.into_election_config(), None);
    }

    #[test]
    fn ps27_with_invalid_region_returns_none() {
        let form = parse("csrf_token=t&election=PS27&region_province=XX");
        assert_eq!(form.into_election_config(), None);
    }

    #[test]
    fn ps27_without_region_returns_none() {
        let form = parse("csrf_token=t&election=PS27");
        assert_eq!(form.into_election_config(), None);
    }

    #[test]
    fn unknown_election_code_returns_none() {
        let form = parse("csrf_token=t&election=ZZ99&region_province=GR");
        assert_eq!(form.into_election_config(), None);
    }
}
