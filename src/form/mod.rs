mod csrf;
mod form_data;
mod validation_error;
mod validators;

pub use csrf::{CsrfToken, CsrfTokens, WithCsrfToken};
pub use form_data::FormData;
pub use validation_error::ValidationError;
pub use validators::*;

pub type FieldErrors = Vec<(String, ValidationError)>;

pub trait Validate<T>
where
    Self: Sized,
{
    fn validate(&self, current: Option<&T>, csrf_tokens: &CsrfTokens) -> Result<T, FormData<Self>>;
}
