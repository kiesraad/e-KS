use serde::{Deserialize, Serialize};

use crate::{core::AnyLocale, form::ValidationError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Female,
    Male,
}

impl std::str::FromStr for Gender {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "female" => Ok(Gender::Female),
            "male" => Ok(Gender::Male),
            _ => Err(ValidationError::InvalidValue),
        }
    }
}

impl std::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gender::Female => write!(f, "female"),
            Gender::Male => write!(f, "male"),
        }
    }
}

impl Gender {
    pub fn abbreviation(&self, locale: AnyLocale) -> &'static str {
        match self {
            Gender::Female => match locale {
                AnyLocale::En | AnyLocale::Fry => "f",
                AnyLocale::Nl => "v",
            },
            Gender::Male => match locale {
                AnyLocale::En | AnyLocale::Fry | AnyLocale::Nl => "m",
            },
        }
    }
}
