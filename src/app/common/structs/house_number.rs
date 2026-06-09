use std::str::FromStr;

use crate::{form::ValidationError, transparent_string};

/// Max practical length - currently there are no house numbers in the bag with more than 5 digits
const MAX_HOUSE_NUMBER_LENGTH: usize = 7;

transparent_string! {
    pub struct HouseNumber(String);
}

impl HouseNumber {
    /// Return the house number as an integer, for BAG lookups that take a
    /// numeric house number.
    pub fn as_number(&self) -> u32 {
        // Note that parse will always succeed because the FromStr implementation only allows valid numeric strings
        self.0.parse().unwrap_or_default()
    }
}

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
