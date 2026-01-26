mod csrf;
mod empty_form;
mod form_data;
mod validation_error;
mod validators;

pub use csrf::{CsrfToken, CsrfTokens, TokenValue, WithCsrfToken};
pub use empty_form::EmptyForm;
pub use form_data::FormData;
pub use validation_error::ValidationError;
pub use validators::*;

pub type FieldErrors = Vec<(String, ValidationError)>;

pub trait Validate<T>
where
    Self: Sized,
{
    fn validate(&self, current: Option<&T>, csrf_tokens: &CsrfTokens) -> Result<T, FormData<Self>>;

    fn validate_update(&self, current: &T, csrf_tokens: &CsrfTokens) -> Result<T, FormData<Self>> {
        self.validate(Some(current), csrf_tokens)
    }

    fn validate_create(&self, csrf_tokens: &CsrfTokens) -> Result<T, FormData<Self>> {
        self.validate(None, csrf_tokens)
    }
}
