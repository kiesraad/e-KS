//! Askama template filters for, among others, display, translation, and validation errors.
//! Used to keep the formatting logic out of the templates.

use crate::{
    ElectionConfig, Locale,
    candidate_lists::CandidateList,
    form::{FormData, WithCsrfToken},
    t,
};

#[askama::filter_fn]
pub fn display<'a>(value: &'a Option<String>, _: &dyn askama::Values) -> askama::Result<&'a str> {
    Ok(value.as_deref().unwrap_or_default())
}

#[askama::filter_fn]
pub fn trans(key: &[&'static str], values: &dyn askama::Values) -> askama::Result<&'static str> {
    let locale: Locale = *askama::get_value(values, "locale")?;

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
    let locale: Locale = *askama::get_value(values, "locale")?;

    Ok(form.error(name, locale))
}

#[askama::filter_fn]
pub fn display_districts(
    list: &CandidateList,
    values: &dyn askama::Values,
) -> askama::Result<String> {
    let locale: Locale = *askama::get_value(values, "locale")?;
    let election: ElectionConfig = *askama::get_value(values, "election")?;

    if !list.electoral_districts.is_empty()
        && list.electoral_districts.len() == election.electoral_districts().len()
    {
        Ok(t!("candidate_list.districts.all", locale).to_string())
    } else {
        Ok(list
            .electoral_districts
            .iter()
            .map(|d| d.title())
            .collect::<Vec<_>>()
            .join(", "))
    }
}

#[askama::filter_fn]
pub fn list_name(list: &CandidateList, values: &dyn askama::Values) -> askama::Result<String> {
    let locale: Locale = *askama::get_value(values, "locale")?;

    if list.electoral_districts.len() < 3 {
        Ok(list
            .electoral_districts
            .iter()
            .map(|d| d.title())
            .collect::<Vec<_>>()
            .join(", "))
    } else {
        Ok(t!("candidate_list.title_single", locale).to_string())
    }
}

/// Returns a cache buster string based on the current git commit hash (set during build on github).
pub fn cache_buster() -> &'static str {
    option_env!("GITHUB_SHA").unwrap_or("development")
}
