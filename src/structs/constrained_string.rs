use derive_more::{Deref, Display, Into};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::form::{ValidationError, validate_length, validate_teletex_chars};

pub type FirstName = ConstrainedString;
pub type LegalName = ConstrainedString;
pub type StreetName = ConstrainedString;
pub type Locality = ConstrainedString;
pub type PlaceOfResidence = ConstrainedString;

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
pub struct ConstrainedString(String);

impl FromStr for ConstrainedString {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed_value = validate_length(value, 2, 200)?;
        validate_teletex_chars(&trimmed_value)?;

        Ok(ConstrainedString(trimmed_value))
    }
}
