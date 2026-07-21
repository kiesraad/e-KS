//! Initials in normalized dotted form (e.g. `A.B.`).
//!
//! Accepted format rules (see `FromStr`):
//! - Whitespace is ignored.
//! - One or more parts, each exactly one alphanumeric teletex character.
//! - Parts are separated by dots and normalized to a trailing dot.
//! - Maximum length is 20 characters after whitespace is removed.
use crate::{
    form::{ValidationError, is_teletex_char},
    transparent_string,
};

transparent_string! {
    pub struct Initials(String);
}

impl std::str::FromStr for Initials {
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
