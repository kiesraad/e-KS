use derive_more::{Deref, Display, Into};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::form::{ValidationError, is_teletex_char};

#[derive(
    Default,
    Debug,
    Deref,
    Clone,
    Into,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct Initials(String);

impl FromStr for Initials {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let initials = value.split_whitespace().collect::<String>();

        if initials.is_empty() {
            return Err(ValidationError::ValueShouldNotBeEmpty);
        }

        if initials.len() > 20 {
            return Err(ValidationError::ValueTooLong(initials.len(), 20));
        }

        let parts: Vec<&str> = initials
            .split('.')
            .filter(|part| !part.is_empty())
            .collect();

        for part in &parts {
            let chars: Vec<char> = part.chars().collect();
            if chars.len() != 1 || !chars[0].is_alphanumeric() || !is_teletex_char(chars[0]) {
                return Err(ValidationError::InvalidValue);
            }
        }

        let result = parts.join(".") + ".";

        Ok(Initials(result))
    }
}
