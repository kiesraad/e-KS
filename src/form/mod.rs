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

use axum::extract::{FromRequest, Request};
use axum_extra::extract::Form as AxumForm;
use serde::de::DeserializeOwned;

use crate::AppError;

pub type FieldErrors = Vec<(String, ValidationError)>;

/// Wrapper that extracts and validates form data from requests.
#[derive(Debug)]
pub struct Form<T>(pub T);

impl<T, S> FromRequest<S> for Form<T>
where
    S: Sync + Send,
    T: DeserializeOwned,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let AxumForm(form) = AxumForm::<T>::from_request(req, state).await?;

        Ok(Form(form))
    }
}

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
