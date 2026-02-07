mod csrf;
mod empty_form;
mod form_data;
mod string_validators;
mod validation_error;

pub use csrf::{CsrfToken, CsrfTokens, TokenValue, WithCsrfToken};
pub use empty_form::EmptyForm;
pub use form_data::FormData;
pub use string_validators::{is_teletex_char, validate_length, validate_teletex_chars};
pub use validation_error::ValidationError;

pub type FieldErrors = Vec<(String, ValidationError)>;

pub trait IntoValidationError {
    fn into_validation_error(self) -> ValidationError;
}

impl IntoValidationError for ValidationError {
    fn into_validation_error(self) -> ValidationError {
        self
    }
}

impl IntoValidationError for std::num::ParseIntError {
    fn into_validation_error(self) -> ValidationError {
        ValidationError::InvalidValue
    }
}

impl IntoValidationError for std::str::ParseBoolError {
    fn into_validation_error(self) -> ValidationError {
        ValidationError::InvalidValue
    }
}

impl IntoValidationError for derive_more::FromStrError {
    fn into_validation_error(self) -> ValidationError {
        ValidationError::InvalidValue
    }
}

impl IntoValidationError for uuid::Error {
    fn into_validation_error(self) -> ValidationError {
        ValidationError::InvalidValue
    }
}

impl IntoValidationError for chrono::ParseError {
    fn into_validation_error(self) -> ValidationError {
        ValidationError::InvalidValue
    }
}
