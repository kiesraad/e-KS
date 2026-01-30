use crate::form::ValidationError;

/// Validates a Dutch house number (digits only, without additions).
pub fn validate_housenumber() -> impl Fn(&str) -> Result<String, ValidationError> {
    |value: &str| {
        let trimmed_value = value.trim();

        if trimmed_value.is_empty() {
            return Err(ValidationError::ValueShouldNotBeEmpty);
        }

        if trimmed_value.len() > 7 {
            return Err(ValidationError::ValueTooLong(trimmed_value.len(), 7));
        }

        if !trimmed_value.chars().all(|c| c.is_ascii_digit()) {
            return Err(ValidationError::InvalidValue);
        }

        Ok(trimmed_value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_trimmed_digits_only() {
        let result = (validate_housenumber())("  12  ").unwrap();
        assert_eq!(result, "12");
    }

    #[test]
    fn rejects_empty_house_number() {
        let err = (validate_housenumber())("   ").unwrap_err();
        assert_eq!(err, ValidationError::ValueShouldNotBeEmpty);
    }

    #[test]
    fn rejects_house_number_with_additions() {
        let err = (validate_housenumber())("12A").unwrap_err();
        assert_eq!(err, ValidationError::InvalidValue);
    }

    #[test]
    fn rejects_house_number_too_long() {
        let err = (validate_housenumber())("12345678901234567").unwrap_err();
        assert_eq!(err, ValidationError::ValueTooLong(17, 7));
    }
}
