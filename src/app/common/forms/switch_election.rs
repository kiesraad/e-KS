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
