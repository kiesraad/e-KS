use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{Display, EnumString};

use crate::form::{TokenValue, WithCsrfToken};
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
#[validate(target = "CandidatePosition", build = "CandidatePositionForm::build")]
#[serde(default)]
pub struct CandidatePositionForm {
    #[validate(parse = "usize")]
    pub position: String,
    #[validate(parse = "CandidatePositionAction")]
    pub action: String,
    #[validate(csrf)]
    pub csrf_token: TokenValue,
}

impl WithCsrfToken for CandidatePositionForm {
    fn with_csrf_token(self, csrf_token: crate::form::CsrfToken) -> Self {
        CandidatePositionForm {
            csrf_token: csrf_token.value,
            ..self
        }
    }
}

impl CandidatePositionForm {
    fn build(
        validated: CandidatePositionFormValidated,
        current: Option<&CandidatePosition>,
    ) -> CandidatePosition {
        if let Some(_current) = current {
            CandidatePosition {
                position: validated.position,
                action: validated.action,
            }
        } else {
            CandidatePosition::default()
        }
    }
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
