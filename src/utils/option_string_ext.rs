//! Convenience helpers for working with `Option<String>` values.

pub trait OptionAsStrExt {
    fn as_str_or_empty(&self) -> &str;
    fn is_empty_or_none(&self) -> bool {
        self.as_str_or_empty().is_empty()
    }
}

impl<T> OptionAsStrExt for Option<T>
where
    T: std::ops::Deref<Target = String>,
{
    fn as_str_or_empty(&self) -> &str {
        self.as_deref()
            .map(|value| value.as_str())
            .unwrap_or_default()
    }
}

pub trait OptionStringExt {
    fn to_string_or_default(self) -> String;
}

impl<T> OptionStringExt for Option<T>
where
    T: std::fmt::Display,
{
    fn to_string_or_default(self) -> String {
        self.map(|value| value.to_string()).unwrap_or_default()
    }
}
