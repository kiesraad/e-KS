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
        let trimmed_value = value.split_whitespace().collect::<Vec<&str>>().join(" ");
        let char_count = trimmed_value
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("")
            .chars()
            .count();

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
    use std::str::FromStr;

    use crate::{form::ValidationError, structs::DisplayName};

    fn test_display_name(input: &str, expected: Result<&str, ValidationError>) {
        let result = DisplayName::from_str(input);
        match result {
            Ok(actual) => assert_eq!(expected.unwrap(), actual.0),
            Err(err) => assert_eq!(expected.unwrap_err(), err),
        }
    }

    #[test]
    fn valid_name() {
        test_display_name("De Tegen Partij", Ok("De Tegen Partij"));
    }

    #[test]
    fn valid_name_with_extra_spaces() {
        test_display_name("\t  De  \t  Tegen   Partij ", Ok("De Tegen Partij"));

        test_display_name("\t  De  \t  Tegen   Partij \t", Ok("De Tegen Partij"));
    }

    #[test]
    fn too_long() {
        test_display_name(
            "a string of exactly 36 chars long ex. spaces",
            Err(ValidationError::ValueTooLong(36, 35)),
        );
    }

    #[test]
    fn too_short() {
        test_display_name(
            "     f   \t      ",
            Err(ValidationError::ValueTooShort(1, 35)),
        );
    }
}
