//! Askama template filters for, among others, display, translation, and validation errors.
//! Used to keep the formatting logic out of the templates.
use chrono::{DateTime, Utc};

use crate::{
    Locale,
    candidate_lists::CandidateList,
    constants::DEFAULT_DATE_TIME_FORMAT,
    form::{FormData, WithCsrfToken},
    trans,
};

#[askama::filter_fn]
pub fn display<T: std::fmt::Display>(
    value: &Option<T>,
    _: &dyn askama::Values,
) -> askama::Result<String> {
    Ok(value.as_ref().map(|v| v.to_string()).unwrap_or_default())
}

#[askama::filter_fn]
pub fn datetime(value: &DateTime<Utc>, _: &dyn askama::Values) -> askama::Result<String> {
    Ok(value.format(DEFAULT_DATE_TIME_FORMAT).to_string())
}

#[askama::filter_fn]
pub fn flag(country_code: &str, _: &dyn askama::Values) -> askama::Result<String> {
    if !country_code.is_ascii() || country_code.len() != 2 {
        return Ok("🌐".to_string());
    }

    let mut flag = String::new();

    for c in country_code.chars() {
        let code = 0x1f1e6 + c.to_ascii_uppercase() as u32 - 65;

        match char::from_u32(code) {
            Some(flag_char) => flag.push(flag_char),
            None => {
                return Ok("🌐".to_string());
            }
        }
    }

    Ok(flag)
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

    if key.is_empty() {
        return Ok("".to_string());
    }

    let mut result = match locale {
        crate::Locale::En => crate::translate::LOCALE_EN.get(key),
        crate::Locale::Nl => crate::translate::LOCALE_NL.get(key),
    }
    .map(|s| s.to_string())
    .unwrap_or_else(|| {
        tracing::warn!("Undefined translation key: [{key}]");

        format!("[{key}]")
    });

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
pub fn list_name(list: &CandidateList, values: &dyn askama::Values) -> askama::Result<String> {
    let locale: Locale = *askama::get_value(values, "locale")?;

    if !list.electoral_districts.is_empty() && list.electoral_districts.len() < 3 {
        Ok(list.districts_name())
    } else {
        Ok(trans!("candidate_list.title_single", locale).to_string())
    }
}

/// Returns a cache buster string based on the current git commit hash (set during build on github).
pub fn cache_buster() -> &'static str {
    option_env!("GITHUB_SHA").unwrap_or("development")
}
