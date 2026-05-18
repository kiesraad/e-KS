//! Locality (place / city name).
//!
//! Validation rules (via `FromStr`):
//! - Whitespace is trimmed; the value must be 2..=200 characters.
//! - Only Teletex characters are allowed.
//! - Known non-official names are replaced by their official counterpart
//!   (see [`replace_locality_alias`]).
use crate::{
    form::{ValidationError, validate_length, validate_teletex_chars},
    transparent_string,
    utils::locality_aliases::replace_locality_alias,
};

transparent_string! {
    pub struct Locality(String);
}

pub type PlaceOfResidence = Locality;

impl std::str::FromStr for Locality {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed_value = validate_length(value, 2, 200)?;
        validate_teletex_chars(&trimmed_value)?;

        let normalized = replace_locality_alias(&trimmed_value).unwrap_or(trimmed_value);

        Ok(Locality(normalized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn keeps_official_name_unchanged() {
        let locality = Locality::from_str("Amsterdam").expect("locality");

        assert_eq!(locality.to_string(), "Amsterdam");
    }

    #[test]
    fn replaces_known_alias_with_official_name() {
        let locality = Locality::from_str("Den Haag").expect("locality");

        assert_eq!(locality.to_string(), "'s-Gravenhage");
    }

    #[test]
    fn rejects_too_short_values() {
        assert_eq!(
            Locality::from_str("A").expect_err("too short"),
            ValidationError::ValueTooShort(1, 2)
        );
    }
}
