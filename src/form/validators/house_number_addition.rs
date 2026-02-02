use crate::form::ValidationError;

/// Max characters according to the BAG specification
/// See https://catalogus.kadaster.nl/bag/nl/page/Huisnummertoevoeging
const MAX_HOUSE_NUMBER_ADDITION_LENGTH: usize = 4;

/// Validates a Dutch house number addition (alphanumeric, no spaces).
pub fn validate_house_number_addition() -> impl Fn(&str) -> Result<String, ValidationError> {
    |value: &str| {
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

        Ok(trimmed_value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_dashes() {
        assert_eq!((validate_house_number_addition())("12-B").unwrap(), "12-B");
        assert_eq!((validate_house_number_addition())("12a").unwrap(), "12a");
        assert_eq!((validate_house_number_addition())("12-").unwrap(), "12-");
        assert_eq!((validate_house_number_addition())("A1").unwrap(), "A1");
    }

    #[test]
    fn accepts_trimmed_alphanumeric_addition() {
        let result = (validate_house_number_addition())("  12B  ").unwrap();
        assert_eq!(result, "12B");
    }

    #[test]
    fn rejects_empty_addition() {
        let err = (validate_house_number_addition())("   ").unwrap_err();
        assert_eq!(err, ValidationError::ValueShouldNotBeEmpty);
    }

    #[test]
    fn rejects_addition_with_invalid_chars() {
        let err = (validate_house_number_addition())("A-1!").unwrap_err();
        assert_eq!(err, ValidationError::InvalidValue);
    }

    #[test]
    fn rejects_addition_too_long() {
        let err = (validate_house_number_addition())("12345678901234567").unwrap_err();
        assert_eq!(err, ValidationError::ValueTooLong(17, 4));
    }
}
