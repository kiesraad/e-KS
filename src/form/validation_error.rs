use crate::{Locale, trans};

type ActualLength = usize;
type MaxLength = usize;
type MinLength = usize;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValidationError {
    InvalidValue,
    InvalidEmail,
    ValueShouldNotBeEmpty,
    InvalidCsrfToken,
    ValueTooLong(ActualLength, MaxLength),
    ValueTooShort(ActualLength, MinLength),
    InvalidChecksum,
    StartsWithLastNamePrefix,
    InvalidPostalCode,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message(Locale::default()))
    }
}

impl ValidationError {
    pub fn message(&self, locale: Locale) -> String {
        match self {
            ValidationError::InvalidValue => trans!("validation.invalid_value", locale),
            ValidationError::InvalidEmail => trans!("validation.invalid_email", locale),
            ValidationError::ValueShouldNotBeEmpty => {
                trans!("validation.value_should_not_be_empty", locale)
            }
            ValidationError::ValueTooLong(actual, max) => {
                trans!("validation.value_too_long", locale, actual, max)
            }
            ValidationError::ValueTooShort(actual, min) => {
                trans!("validation.value_too_short", locale, actual, min)
            }
            ValidationError::InvalidCsrfToken => trans!("validation.invalid_csrf_token", locale),
            ValidationError::InvalidChecksum => trans!("validation.invalid_bsn", locale),
            ValidationError::StartsWithLastNamePrefix => {
                trans!("validation.starts_with_last_name_prefix", locale)
            }
            ValidationError::InvalidPostalCode => {
                trans!("validation.invalid_postal_code", locale)
            }
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_messages_in_english() {
        assert_eq!(
            ValidationError::InvalidCsrfToken.message(Locale::En),
            "The CSRF token is invalid."
        );
        assert_eq!(
            ValidationError::InvalidValue.message(Locale::En),
            "The provided value is not valid."
        );
        assert_eq!(
            ValidationError::InvalidEmail.message(Locale::En),
            "Invalid email address."
        );
        assert_eq!(
            ValidationError::ValueShouldNotBeEmpty.message(Locale::En),
            "This field must not be empty."
        );
        assert_eq!(
            ValidationError::ValueTooLong(10, 5).message(Locale::En),
            "The value is too long (10 characters), maximum 5 characters allowed."
        );
        assert_eq!(
            ValidationError::ValueTooShort(2, 5).message(Locale::En),
            "The value is too short (2 characters), minimum 5 characters required."
        );
        assert_eq!(
            ValidationError::InvalidChecksum.message(Locale::En),
            "Invalid BSN."
        );
        assert_eq!(
            ValidationError::StartsWithLastNamePrefix.message(Locale::En),
            "Please put the prefix in the correct field."
        );
    }

    #[test]
    fn display_uses_default_locale() {
        let message = ValidationError::InvalidEmail.to_string();
        assert!(!message.is_empty());
    }
}
