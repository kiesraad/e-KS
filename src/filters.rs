//! Askama template filters for, among others, display, translation, and validation errors.
//! Used to keep the formatting logic out of the templates.
use crate::{
    ElectionConfig, Locale,
    candidate_lists::CandidateList,
    form::{FormData, WithCsrfToken},
    trans,
};

#[askama::filter_fn]
pub fn display<'a>(value: &'a Option<String>, _: &dyn askama::Values) -> askama::Result<&'a str> {
    Ok(value.as_deref().unwrap_or_default())
}

#[askama::filter_fn]
pub fn trans(
    key: &str,
    values: &dyn askama::Values,
    #[optional("")] param0: &str,
    #[optional("")] param1: &str,
    #[optional("")] param2: &str,
) -> askama::Result<String> {
    let locale: Locale = *askama::get_value(values, "locale")?;

    let mut result = match locale {
        crate::locale::Locale::En => crate::translate::LOCALE_EN.get(key),
        crate::locale::Locale::Nl => crate::translate::LOCALE_NL.get(key),
    }
    .map(|s| s.to_string())
    .ok_or_else(|| askama::Error::Custom(format!("translation key not found: {key}").into()))?;

    if !param0.is_empty() {
        result = result.replacen("{}", param0, 1);

        if !param1.is_empty() {
            result = result.replacen("{}", param1, 1);

            if !param2.is_empty() {
                result = result.replacen("{}", param2, 1);
            }
        }
    }

    Ok(result)
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
        Ok(trans!("candidate_list.districts.all", locale).to_string())
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
        Ok(trans!("candidate_list.title_single", locale).to_string())
    }
}

/// Returns a cache buster string based on the current git commit hash (set during build on github).
pub fn cache_buster() -> &'static str {
    option_env!("GITHUB_SHA").unwrap_or("development")
}
