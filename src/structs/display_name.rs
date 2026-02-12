use derive_more::{Deref, Display, Into};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::form::{ValidationError, validate_teletex_chars};

const MAX_LENGTH: usize = 35;

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
pub struct DisplayName(String);

impl FromStr for DisplayName {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let words: Vec<_> = value.split_whitespace().collect();
        let trimmed_value = words.join(" ");
        let char_count: usize = words.iter().map(|w| w.chars().count()).sum();

        if char_count < 2 {
            return Err(ValidationError::ValueTooShort(char_count, MAX_LENGTH));
        }

        if char_count > MAX_LENGTH {
            return Err(ValidationError::ValueTooLong(char_count, MAX_LENGTH));
        }
        validate_teletex_chars(&trimmed_value)?;
        Ok(DisplayName(trimmed_value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_name() {
        assert_eq!(
            Ok(DisplayName("De Tegen Partij".to_string())),
            DisplayName::from_str("De Tegen Partij")
        );
    }

    #[test]
    fn valid_name_with_extra_spaces() {
        assert_eq!(
            Ok(DisplayName("De Tegen Partij".to_string())),
            DisplayName::from_str("\t  De  \t  Tegen   Partij ")
        );

        assert_eq!(
            Ok(DisplayName("De Tegen Partij".to_string())),
            DisplayName::from_str("\t  De  \t  Tegen   Partij \t")
        );
    }

    #[test]
    fn too_long() {
        assert_eq!(
            Err(ValidationError::ValueTooLong(36, 35)),
            DisplayName::from_str("a string of exactly 36 chars long ex. spaces")
        );
    }

    #[test]
    fn too_short() {
        assert_eq!(
            Err(ValidationError::ValueTooShort(1, 35)),
            DisplayName::from_str("     f   \t      ")
        );
    }
}
