//! Postal code in normalized Dutch format `1234AB`.
//!
//! Validation rules (via `FromStr`):
//! - Whitespace is ignored.
//! - Exactly 6 characters after whitespace removal.
//! - First 4 are ASCII digits, last 2 are ASCII letters.
//! - Letters are uppercased for normalization.
use crate::{form::ValidationError, transparent_string};

transparent_string! {
    pub struct PostalCode(String);
}

impl std::str::FromStr for PostalCode {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let postal_code = value.split_whitespace().collect::<String>();

        if postal_code.len() != 6 {
            return Err(ValidationError::InvalidPostalCode);
        }

        let bytes = postal_code.as_bytes();

        if !bytes[..4].iter().all(|b| b.is_ascii_digit()) {
            return Err(ValidationError::InvalidPostalCode);
        }

        if !bytes[4..].iter().all(|b| b.is_ascii_alphabetic()) {
            return Err(ValidationError::InvalidPostalCode);
        }

        Ok(PostalCode(postal_code.to_ascii_uppercase()))
    }
}
