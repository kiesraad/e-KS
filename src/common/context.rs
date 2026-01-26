//! Request-scoped template context carrying locale and helpers.
//! Extracted from requests and passed into Askama templates.

use std::convert::Infallible;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::{CsrfTokens, ElectionConfig, Locale};

#[derive(Default, Clone)]
pub struct Context {
    pub locale: Locale,
    pub election: ElectionConfig,
    pub max_candidates: usize,
    pub csrf_tokens: CsrfTokens,
}

impl Context {
    pub fn new(locale: Locale, csrf_tokens: CsrfTokens) -> Self {
        Self {
            locale,
            election: ElectionConfig::EK2027,
            max_candidates: 50,
            csrf_tokens,
        }
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self::new(Locale::En, CsrfTokens::default())
    }

    pub fn livereload_enabled() -> bool {
        cfg!(feature = "livereload")
    }
}

impl askama::Values for Context {
    fn get_value<'a>(&'a self, key: &str) -> Option<&'a dyn std::any::Any> {
        match key {
            "locale" => Some(&self.locale as &dyn std::any::Any),
            "election" => Some(&self.election as &dyn std::any::Any),
            "max_candidates" => Some(&self.max_candidates as &dyn std::any::Any),
            _ => None,
        }
    }
}

impl<S> FromRequestParts<S> for Context
where
    S: Clone + Send + Sync + 'static,
    CsrfTokens: FromRef<S>,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let locale = Locale::from_request_parts(parts, state).await?;
        let csrf_tokens = CsrfTokens::from_ref(state);

        Ok(Context::new(locale, csrf_tokens))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_context_sets_locale_and_index() {
        let context = Context::new(Locale::En, CsrfTokens::default());
        assert_eq!(context.locale, Locale::En);
    }

    #[test]
    fn livereload_flag_matches_feature() {
        assert_eq!(Context::livereload_enabled(), cfg!(feature = "livereload"));
    }
}
