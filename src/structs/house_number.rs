use derive_more::{Deref, Display, Into};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::form::ValidationError;

/// Max practical length - currently there are no house numbers in the bag with more than 5 digits
const MAX_HOUSE_NUMBER_LENGTH: usize = 7;

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
pub struct HouseNumber(String);

impl FromStr for HouseNumber {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed_value = value.trim();

        if trimmed_value.is_empty() {
            return Err(ValidationError::ValueShouldNotBeEmpty);
        }

        if trimmed_value.len() > MAX_HOUSE_NUMBER_LENGTH {
            return Err(ValidationError::ValueTooLong(
                trimmed_value.len(),
                MAX_HOUSE_NUMBER_LENGTH,
            ));
        }

        if !trimmed_value.chars().all(|c| c.is_ascii_digit()) {
            return Err(ValidationError::InvalidValue);
        }

        Ok(HouseNumber(trimmed_value.to_string()))
    }
}
