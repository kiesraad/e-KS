use crate::{Locale, form::FieldErrors};

#[derive(Debug, Clone)]
pub struct FormData<T> {
    pub data: T,
    errors: FieldErrors,
}

impl<T: Default> FormData<T> {
    pub fn new() -> Self {
        Self {
            data: T::default(),
            errors: Vec::new(),
        }
    }
}

impl<T: Default> Default for FormData<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FormData<T> {
    pub fn new_with_data(data: T) -> Self {
        Self {
            data,
            errors: Vec::new(),
        }
    }

    pub fn new_with_errors(data: T, errors: FieldErrors) -> Self {
        Self { data, errors }
    }

    pub fn errors(self) -> FieldErrors {
        self.errors
    }

    pub fn error(&self, name: &str, locale: Locale) -> Vec<String> {
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
    use crate::{Locale, form::ValidationError};

    #[derive(Default)]
    struct DummyForm;

    #[test]
    fn collects_errors_for_named_field() {
        let form: FormData<DummyForm> = FormData::new_with_errors(
            Default::default(),
            vec![
                ("name".to_string(), ValidationError::ValueShouldNotBeEmpty),
                ("other".to_string(), ValidationError::InvalidValue),
            ],
        );

        let messages = form.error("name", Locale::En);
        assert_eq!(messages, vec!["This field must not be empty.".to_string()]);
    }
}
