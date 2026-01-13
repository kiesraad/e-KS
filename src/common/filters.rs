//! Askama template filters for among others, display, translation, and validation errors.
//! Used keep formatting logic out of templates.

use crate::{
    Locale,
    form::{FormData, WithCsrfToken},
};

#[askama::filter_fn]
pub fn display<'a>(value: &'a Option<String>, _: &dyn askama::Values) -> askama::Result<&'a str> {
    Ok(value.as_deref().unwrap_or_default())
}

#[askama::filter_fn]
pub fn trans(key: &[&'static str], values: &dyn askama::Values) -> askama::Result<&'static str> {
    let locale: &Locale = askama::get_value(values, "locale")?;

    Ok(key[locale.as_usize()])
}

#[askama::filter_fn]
pub fn fill<S: AsRef<str>, T: AsRef<str>>(
    value: S,
    _: &dyn askama::Values,
    args: T,
) -> askama::Result<String> {
    Ok(value.as_ref().replacen("{}", args.as_ref(), 1))
}

#[askama::filter_fn]
pub fn error<T: WithCsrfToken>(
    form: &FormData<T>,
    values: &dyn askama::Values,
    name: &str,
) -> askama::Result<Vec<String>> {
    let locale: &Locale = askama::get_value(values, "locale")?;

    Ok(form.error(name, locale))
}

/// Returns a cache buster string based on the current git commit hash (set during build on github).
pub fn cache_buster() -> &'static str {
    option_env!("GITHUB_SHA").unwrap_or("development")
}
