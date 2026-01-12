use crate::{Locale, t};

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
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message(&Locale::default()))
    }
}

impl ValidationError {
    pub fn message(&self, locale: &Locale) -> String {
        match self {
            ValidationError::InvalidValue => t!("validation.invalid_value", locale),
            ValidationError::InvalidEmail => t!("validation.invalid_email", locale),
            ValidationError::ValueShouldNotBeEmpty => {
                t!("validation.value_should_not_be_empty", locale)
            }
            ValidationError::ValueTooLong(actual, max) => {
                t!("validation.value_too_long", locale, actual, max)
            }
            ValidationError::ValueTooShort(actual, min) => {
                t!("validation.value_too_short", locale, actual, min)
            }
            ValidationError::InvalidCsrfToken => t!("validation.invalid_csrf_token", locale),
            ValidationError::InvalidChecksum => t!("validation.invalid_bsn", locale),
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
            ValidationError::InvalidCsrfToken.message(&Locale::En),
            "The CSRF token is invalid."
        );
        assert_eq!(
            ValidationError::ValueTooShort(2, 5).message(&Locale::En),
            "The value is too short (2 characters), minimum 5 characters required."
        );
    }

    #[test]
    fn display_uses_default_locale() {
        let message = ValidationError::InvalidEmail.to_string();
        assert!(!message.is_empty());
    }
}
