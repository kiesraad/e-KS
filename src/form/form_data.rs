use crate::{
    CsrfTokens, Locale,
    form::{FieldErrors, WithCsrfToken},
};

#[derive(Debug, Clone)]
pub struct FormData<T> {
    pub data: T,
    errors: FieldErrors,
}

impl<T: WithCsrfToken> FormData<T> {
    pub fn new(csrf_tokens: &CsrfTokens) -> Self {
        Self {
            data: T::default().with_csrf_token(csrf_tokens.issue()),
            errors: Vec::new(),
        }
    }

    pub fn new_with_data(data: T, csrf_tokens: &CsrfTokens) -> Self {
        Self {
            data: data.with_csrf_token(csrf_tokens.issue()),
            errors: Vec::new(),
        }
    }

    pub fn new_with_errors(data: T, csrf_tokens: &CsrfTokens, errors: FieldErrors) -> Self {
        Self {
            data: data.with_csrf_token(csrf_tokens.issue()),
            errors,
        }
    }

    #[cfg(test)]
    pub fn errors(&self) -> &FieldErrors {
        &self.errors
    }

    pub fn error(&self, name: &str, locale: &Locale) -> Vec<String> {
        self.errors
            .iter()
            .filter(|(field_name, _)| field_name == name)
            .map(|(_, error)| error.message(locale))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CsrfToken, Locale, form::ValidationError};

    #[derive(Default)]
    struct DummyForm;

    impl WithCsrfToken for DummyForm {
        fn with_csrf_token(self, _csrf_token: CsrfToken) -> Self {
            self
        }
    }

    #[test]
    fn collects_errors_for_named_field() {
        let form: FormData<DummyForm> = FormData::new_with_errors(
            Default::default(),
            &CsrfTokens::default(),
            vec![
                ("name".to_string(), ValidationError::ValueShouldNotBeEmpty),
                ("other".to_string(), ValidationError::InvalidValue),
            ],
        );

        let messages = form.error("name", &Locale::En);
        assert_eq!(messages, vec!["This field must not be empty.".to_string()]);
    }
}
