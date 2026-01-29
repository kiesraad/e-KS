use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::form::TokenValue;
use validate::Validate;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum CandidatePositionAction {
    #[default]
    Move,
    Remove,
}

#[derive(Debug, Default, Clone)]
pub struct CandidatePosition {
    pub position: usize,
    pub action: CandidatePositionAction,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "CandidatePosition")]
#[serde(default)]
pub struct CandidatePositionForm {
    #[validate(parse = "usize")]
    pub position: String,
    #[validate(parse = "CandidatePositionAction")]
    pub action: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl From<CandidatePosition> for CandidatePositionForm {
    fn from(position: CandidatePosition) -> Self {
        CandidatePositionForm {
            position: position.position.to_string(),
            action: position.action.to_string(),
            csrf_token: Default::default(),
        }
    }
}
