use std::str::FromStr;

use crate::{form::ValidationError, transparent_string};

transparent_string! {
    pub struct CountryCode(String);
}

impl FromStr for CountryCode {
    type Err = ValidationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed_value = value.trim().to_uppercase();

        if !super::COUNTRY_CODES.contains(&trimmed_value.as_str()) {
            return Err(ValidationError::InvalidValue);
        }

        Ok(CountryCode(trimmed_value))
    }
}
