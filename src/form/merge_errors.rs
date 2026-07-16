use crate::form::{FieldErrors, FormData};

/// Merge additional field errors from extra checks into a validation result.
pub trait MergeErrors<T, F> {
    fn merge_errors(self, form: F, additional_errors: FieldErrors) -> Result<T, Box<FormData<F>>>;
}

impl<T, F> MergeErrors<T, F> for Result<T, FormData<F>> {
    fn merge_errors(self, form: F, additional_errors: FieldErrors) -> Result<T, Box<FormData<F>>> {
        if additional_errors.is_empty() {
            return Ok(self?);
        }

        let mut errors = match self {
            Ok(_) => Vec::new(),
            Err(form_data) => form_data.errors(),
        };
        errors.extend(additional_errors);
        Err(Box::new(FormData::new_with_errors(form, errors)))
    }
}
