use serde::Deserialize;
use validate::Validate;

use crate::{
    OptionStringExt, TokenValue,
    common::{DisplayName, LegalName, PreviousElectionResults},
    political_groups::PoliticalGroup,
};

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "PoliticalGroup")]
#[serde(default)]
pub struct PoliticalGroupForm {
    #[validate(parse = "PreviousElectionResults", optional)]
    pub previous_election_results: String,
    #[validate(parse = "LegalName", optional)]
    pub legal_name: String,
    #[validate(parse = "DisplayName", optional)]
    pub display_name: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<PoliticalGroup> for PoliticalGroupForm {
    fn from(value: PoliticalGroup) -> Self {
        PoliticalGroupForm {
            previous_election_results: value
                .previous_election_results
                .map(|b| b.to_string())
                .unwrap_or_default(),
            legal_name: value.legal_name.to_string_or_default(),
            display_name: value.display_name.to_string_or_default(),
            csrf_token: Default::default(),
        }
    }
}
