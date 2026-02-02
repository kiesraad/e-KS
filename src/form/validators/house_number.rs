use crate::form::ValidationError;

/// Max practical length - currently there are no house numbers in the bag with more than 5 digits
const MAX_HOUSE_NUMBER_LENGTH: usize = 7;

/// Validates a Dutch house number (digits only, without additions).
pub fn validate_housenumber() -> impl Fn(&str) -> Result<String, ValidationError> {
    |value: &str| {
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

        Ok(trimmed_value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_prefix_or_suffix_whitespace() {
        assert_eq!((validate_housenumber())("  12  ").unwrap(), "12");
        assert_eq!(
            (validate_housenumber())("   1    24   ").unwrap_err(),
            ValidationError::InvalidValue
        );
    }

    #[test]
    fn rejects_empty_house_number() {
        let err = (validate_housenumber())("   ").unwrap_err();
        assert_eq!(err, ValidationError::ValueShouldNotBeEmpty);
    }

    #[test]
    fn rejects_house_number_with_additions() {
        assert_eq!(
            (validate_housenumber())("12A").unwrap_err(),
            ValidationError::InvalidValue
        );
        assert_eq!(
            (validate_housenumber())("+12").unwrap_err(),
            ValidationError::InvalidValue
        );
        assert_eq!(
            (validate_housenumber())("-23").unwrap_err(),
            ValidationError::InvalidValue
        );
    }

    #[test]
    fn rejects_house_number_too_long() {
        let err = (validate_housenumber())("12345678901234567").unwrap_err();
        assert_eq!(err, ValidationError::ValueTooLong(17, 7));
    }
}
