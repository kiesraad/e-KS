use std::str::FromStr;

use crate::{
    form::{ValidationError, validate_length, validate_teletex_chars},
    transparent_string,
};

pub type FirstName = ConstrainedString;
pub type LegalName = ConstrainedString;
pub type StreetName = ConstrainedString;
pub type Locality = ConstrainedString;
pub type PlaceOfResidence = ConstrainedString;

transparent_string! {
    pub struct ConstrainedString(String);
}

impl FromStr for ConstrainedString {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed_value = validate_length(value, 2, 200)?;
        validate_teletex_chars(&trimmed_value)?;

        Ok(ConstrainedString(trimmed_value))
    }
}
