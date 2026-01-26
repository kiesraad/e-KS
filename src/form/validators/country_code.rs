use crate::{form::ValidationError, persons::COUNTRY_CODES};

/// Validates a country code, using the list of all valid country codes from the RvIG
pub fn validate_country_code() -> impl Fn(&str) -> Result<String, ValidationError> {
    move |value: &str| {
        let trimmed_value = value.trim().to_uppercase();

        if !COUNTRY_CODES.contains(&trimmed_value.as_str()) {
            return Err(ValidationError::InvalidValue);
        }

        Ok(trimmed_value)
    }
}
