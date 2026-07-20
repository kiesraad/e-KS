//! Postal code in normalized Dutch format `1234AB`.
//!
//! Validation rules (via `FromStr`):
//! - Whitespace is ignored.
//! - Exactly 6 characters after whitespace removal.
//! - First 4 are ASCII digits, last 2 are ASCII letters.
//! - Letters are uppercased for normalization.
use crate::{
    form::{ValidationError, validate_length, validate_teletex_chars},
    transparent_string,
};

transparent_string! {
    pub struct PostalCode(String);
}

transparent_string! {
    pub struct InternationalPostalCode(String);
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

impl std::str::FromStr for InternationalPostalCode {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed_value = validate_length(value, 1, 20)?;
        validate_teletex_chars(&trimmed_value)?;

        Ok(InternationalPostalCode(trimmed_value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn dutch_postal_code_normalizes_whitespace_and_case() {
        let postal_code = PostalCode::from_str("1234 ab").expect("postal code");

        assert_eq!(postal_code.to_string(), "1234AB");
    }

    #[test]
    fn dutch_postal_code_rejects_invalid_values() {
        assert_eq!(
            PostalCode::from_str("ABCDE").expect_err("invalid postal code"),
            ValidationError::InvalidPostalCode
        );
    }

    #[test]
    fn international_postal_code_accepts_less_strict_formats() {
        let postal_code = InternationalPostalCode::from_str(" SW1A 1AA ").expect("postal code");

        assert_eq!(postal_code.to_string(), "SW1A 1AA");
    }

    #[test]
    fn international_postal_code_rejects_empty_values() {
        assert_eq!(
            InternationalPostalCode::from_str(" ").expect_err("empty postal code"),
            ValidationError::ValueShouldNotBeEmpty
        );
    }
}
