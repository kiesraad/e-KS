use crate::form::ValidationError;

/// Validates a name string, checking for length and emptiness.
pub fn validate_length(
    min_length: usize,
    max_length: usize,
) -> impl Fn(&str) -> Result<String, ValidationError> {
    move |value: &str| {
        let trimmed_value = value.trim();

        if trimmed_value.is_empty() {
            return Err(ValidationError::ValueShouldNotBeEmpty);
        }

        if trimmed_value.len() < min_length {
            return Err(ValidationError::ValueTooShort(
                trimmed_value.len(),
                min_length,
            ));
        }

        if trimmed_value.len() > max_length {
            return Err(ValidationError::ValueTooLong(
                trimmed_value.len(),
                max_length,
            ));
        }

        Ok(trimmed_value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_trimmed_value_within_bounds() {
        let result = (validate_length(2, 5))("  abcd  ").unwrap();
        assert_eq!(result, "abcd");
    }

    #[test]
    fn rejects_empty_value() {
        let err = (validate_length(1, 3))("").unwrap_err();
        assert_eq!(err, ValidationError::ValueShouldNotBeEmpty);

        let err = (validate_length(1, 3))("   ").unwrap_err();
        assert_eq!(err, ValidationError::ValueShouldNotBeEmpty);
    }

    #[test]
    fn rejects_value_too_short() {
        let err = (validate_length(3, 10))(" a ").unwrap_err();
        assert_eq!(err, ValidationError::ValueTooShort(1, 3));
    }

    #[test]
    fn rejects_value_too_long() {
        let err = (validate_length(1, 3))("  abcde  ").unwrap_err();
        assert_eq!(err, ValidationError::ValueTooLong(5, 3));
    }
}
