use crate::form::ValidationError;

pub fn validate_postal_code() -> impl Fn(&str) -> Result<String, ValidationError> {
    |value: &str| {
        let postal_code = value.split_whitespace().collect::<String>();

        if postal_code.len() != 6 {
            return Err(ValidationError::InvalidPostalCode);
        }

        let bytes = postal_code.as_bytes();

        if !bytes[..4].iter().all(|b| b.is_ascii_digit()) {
            return Err(ValidationError::InvalidPostalCode);
        }

        if !bytes[4..].iter().all(|b| b.is_ascii_alphabetic()) {
            return Err(ValidationError::InvalidPostalCode);
        }

        Ok(postal_code.to_ascii_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_postal_code_without_space() {
        let result = (validate_postal_code())("1234AB").unwrap();
        assert_eq!(result, "1234AB");
    }

    #[test]
    fn accepts_postal_code_with_space_and_lowercase_letters() {
        let result = (validate_postal_code())("1234 ab").unwrap();
        assert_eq!(result, "1234AB");
    }

    #[test]
    fn rejects_postal_code_with_invalid_length() {
        let err = (validate_postal_code())("123AB").unwrap_err();
        assert_eq!(err, ValidationError::InvalidPostalCode);

        let err = (validate_postal_code())("12345AB").unwrap_err();
        assert_eq!(err, ValidationError::InvalidPostalCode);
    }

    #[test]
    fn rejects_postal_code_with_non_digit_prefix() {
        let err = (validate_postal_code())("12A4AB").unwrap_err();
        assert_eq!(err, ValidationError::InvalidPostalCode);
    }

    #[test]
    fn rejects_postal_code_with_non_alpha_suffix() {
        let err = (validate_postal_code())("1234A1").unwrap_err();
        assert_eq!(err, ValidationError::InvalidPostalCode);
    }
}
