//! Request-scoped template context carrying locale and helpers.
//! Extracted from requests and passed into Askama templates.

use std::convert::Infallible;

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::Locale;

#[derive(Default, Clone, Copy, Debug)]
pub struct Context {
    pub locale: Locale,
    pub l: usize,
}

impl Context {
    pub fn new(locale: Locale) -> Self {
        Self {
            locale,
            l: locale.as_usize(),
        }
    }

    pub fn livereload_enabled() -> bool {
        cfg!(feature = "livereload")
    }
}

impl askama::Values for Context {
    fn get_value<'a>(&'a self, key: &str) -> Option<&'a dyn std::any::Any> {
        match key {
            "locale" => Some(&self.locale as &dyn std::any::Any),
            _ => None,
        }
    }
}

impl<S> FromRequestParts<S> for Context
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let locale = Locale::from_request_parts(parts, state).await?;
        Ok(Context {
            locale,
            l: locale.as_usize(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_context_sets_locale_and_index() {
        let context = Context::new(Locale::En);
        assert_eq!(context.locale, Locale::En);
        assert_eq!(context.l, Locale::En.as_usize());
    }

    #[test]
    fn livereload_flag_matches_feature() {
        assert_eq!(Context::livereload_enabled(), cfg!(feature = "livereload"));
    }
}
