use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};

use crate::{UtcDateTime, form::TokenValue};
use validate::Validate;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Display, FromStr)]
#[display(rename_all = "snake_case")]
#[from_str(rename_all = "snake_case")]
pub enum CandidatePositionAction {
    #[default]
    Move,
    Remove,
}

#[derive(Debug, Default, Clone)]
pub struct CandidatePosition {
    pub position: usize,
    pub action: CandidatePositionAction,
    #[allow(unused)]
    pub created_at: UtcDateTime,
    #[allow(unused)]
    pub updated_at: UtcDateTime,
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
