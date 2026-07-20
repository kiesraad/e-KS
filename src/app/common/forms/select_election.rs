use serde::Deserialize;

use crate::ElectionConfig;

/// Form for the post-login election selection page. Same region semantics as
/// `SwitchElectionForm` but adds an optional `load_fixtures` toggle that only
/// has an effect when the `dev-features` / `fixtures` features are compiled in.
#[derive(Deserialize)]
pub struct SelectElectionForm {
    election: String,
    region_province: Option<String>,
    region_water_council: Option<String>,
    load_fixtures: Option<String>,
    #[cfg(feature = "fixtures")]
    login_as_csb: Option<String>,
}

impl SelectElectionForm {
    pub fn election_config(&self) -> Option<ElectionConfig> {
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

    pub fn load_fixtures(&self) -> bool {
        self.load_fixtures.is_some()
    }

    #[cfg(feature = "fixtures")]
    pub fn login_as_csb(&self) -> bool {
        self.login_as_csb.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Province;

    fn parse(body: &str) -> SelectElectionForm {
        serde_urlencoded::from_str(body).expect("form body")
    }

    #[test]
    fn deserializes_without_load_fixtures() {
        let form = parse("election=EK27");
        assert_eq!(form.election_config(), Some(ElectionConfig::EK27));
        assert!(!form.load_fixtures());
    }

    #[test]
    fn load_fixtures_checkbox_present_is_true() {
        let form = parse("election=EK27&load_fixtures=true");
        assert!(form.load_fixtures());
    }

    #[test]
    fn ps27_uses_region_province() {
        let form = parse("election=PS27&region_province=GR");
        assert_eq!(
            form.election_config(),
            Some(ElectionConfig::PS27(Province::GR))
        );
    }
}
