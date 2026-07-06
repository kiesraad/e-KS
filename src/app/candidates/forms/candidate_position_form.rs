use serde::{Deserialize, Serialize};
use validate::Validate;

use crate::{candidates::CandidatePosition, common::FormAction};

#[derive(Default, Serialize, Deserialize, Clone, Debug, Validate)]
#[validate(target = "CandidatePosition")]
#[serde(default)]
pub struct CandidatePositionForm {
    #[validate(parse = "usize")]
    pub position: String,
    #[validate(parse = "FormAction")]
    pub action: String,
}

impl From<CandidatePosition> for CandidatePositionForm {
    fn from(position: CandidatePosition) -> Self {
        CandidatePositionForm {
            position: position.position.to_string(),
            action: position.action.to_string(),
        }
    }
}
