use serde::{Deserialize, Serialize};

use crate::{UtcDateTime, form::TokenValue, structs::FormAction};
use validate::Validate;

#[derive(Debug, Default, Clone)]
pub struct CandidatePosition {
    pub position: usize,
    pub action: FormAction,
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
    #[validate(parse = "FormAction")]
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
