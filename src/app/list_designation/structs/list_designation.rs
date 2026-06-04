use crate::form::ValidationError;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListDesignation {
    #[default]
    Standalone,
    Blank,
    Combined,
}

impl std::str::FromStr for ListDesignation {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "standalone" => Ok(ListDesignation::Standalone),
            "blank" => Ok(ListDesignation::Blank),
            "combined" => Ok(ListDesignation::Combined),
            _ => Err(ValidationError::InvalidValue),
        }
    }
}

impl std::fmt::Display for ListDesignation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListDesignation::Standalone => write!(f, "standalone"),
            ListDesignation::Blank => write!(f, "blank"),
            ListDesignation::Combined => write!(f, "combined"),
        }
    }
}
