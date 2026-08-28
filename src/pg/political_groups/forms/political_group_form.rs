use serde::Deserialize;
use validate::Validate;

use crate::{
    OptionStringExt,
    structs::{
        common::{Appellation, PreviousElectionResults},
        political_groups::PoliticalGroup,
    },
};

#[derive(Default, Deserialize, Debug, Validate)]
#[validate(target = "PoliticalGroup")]
#[serde(default)]
pub struct PoliticalGroupForm {
    #[validate(parse = "PreviousElectionResults", optional)]
    pub previous_election_results: String,
    #[validate(parse = "Appellation", optional)]
    pub appellation: String,
}

impl From<PoliticalGroup> for PoliticalGroupForm {
    fn from(value: PoliticalGroup) -> Self {
        PoliticalGroupForm {
            previous_election_results: value
                .previous_election_results
                .map(|r| r.to_string())
                .unwrap_or_default(),
            appellation: value.appellation.to_string_or_default(),
        }
    }
}
