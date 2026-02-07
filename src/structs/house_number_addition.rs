use derive_more::{Deref, Display, Into};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::form::ValidationError;

/// Max characters according to the BAG specification
/// See https://catalogus.kadaster.nl/bag/nl/page/Huisnummertoevoeging
const MAX_HOUSE_NUMBER_ADDITION_LENGTH: usize = 4;

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
pub struct HouseNumberAddition(String);

impl FromStr for HouseNumberAddition {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed_value = value.trim();

        if trimmed_value.is_empty() {
            return Err(ValidationError::ValueShouldNotBeEmpty);
        }

        if trimmed_value.len() > MAX_HOUSE_NUMBER_ADDITION_LENGTH {
            return Err(ValidationError::ValueTooLong(
                trimmed_value.len(),
                MAX_HOUSE_NUMBER_ADDITION_LENGTH,
            ));
        }

        if !trimmed_value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err(ValidationError::InvalidValue);
        }

        Ok(HouseNumberAddition(trimmed_value.to_string()))
    }
}
