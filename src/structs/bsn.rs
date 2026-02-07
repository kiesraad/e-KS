use derive_more::{Deref, Display, Into};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::form::ValidationError;

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
pub struct Bsn(String);

impl FromStr for Bsn {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed_value = value.trim();

        if trimmed_value.is_empty() {
            return Err(ValidationError::ValueShouldNotBeEmpty);
        }

        if trimmed_value.len() < 9 {
            return Err(ValidationError::ValueTooShort(trimmed_value.len(), 9));
        }

        if trimmed_value.len() > 9 {
            return Err(ValidationError::ValueTooLong(trimmed_value.len(), 9));
        }

        let mut checksum = 0;
        for (i, digit) in trimmed_value.chars().rev().enumerate() {
            checksum += (if i == 0 { -1 } else { i as i32 + 1 })
                * (digit.to_digit(10).ok_or(ValidationError::InvalidValue)?) as i32;
        }

        if checksum % 11 != 0 {
            return Err(ValidationError::InvalidChecksum);
        }

        Ok(Bsn(trimmed_value.to_string()))
    }
}
