//! Request-scoped template context carrying locale and helpers.
//! Extracted from requests and passed into Askama templates.

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::{
    AppError, AppStore, CsrfTokens, ElectionConfig, Locale, political_groups::PoliticalGroup,
};

#[derive(Clone)]
pub struct Context {
    /// Political group tied to the session / request.
    pub political_group: PoliticalGroup,
    /// Active locale for translations and formatting.
    pub locale: Locale,
    /// Election configuration used to compute names, districts, rules and limits.
    pub election: ElectionConfig,
    /// Maximum number of candidates allowed for this political group.
    pub max_candidates: usize,
    /// Whether to show the success alert based on the request query.
    pub show_success_alert: bool,
    /// Whether the request came from an overlay page (via referrer query).
    pub overlay_refferer: bool,
    /// CSRF tokens used to protect form submissions.
    pub csrf_tokens: CsrfTokens,
}

impl Context {
    pub fn new(political_group: PoliticalGroup, locale: Locale, csrf_tokens: CsrfTokens) -> Self {
        let election = ElectionConfig::EK2027;
        let long_list_allowed = political_group.long_list_allowed.unwrap_or(false);

        Self {
            political_group,
            locale,
            max_candidates: election.get_max_candidates(long_list_allowed),
            election,
            csrf_tokens,
            show_success_alert: false,
            overlay_refferer: false,
        }
    }

    #[cfg(test)]
    pub async fn new_test() -> Self {
        let political_group = PoliticalGroup::default();

        Self::new(political_group, Locale::En, CsrfTokens::default())
    }

    #[cfg(test)]
    pub fn new_test_without_db() -> Self {
        Self::new(PoliticalGroup::default(), Locale::En, CsrfTokens::default())
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
            "show_success_alert" => Some(&self.show_success_alert as &dyn std::any::Any),
            "overlay_refferer" => Some(&self.overlay_refferer as &dyn std::any::Any),
            _ => None,
        }
    }
}

impl<S> FromRequestParts<S> for Context
where
    S: Clone + Send + Sync + 'static,
    CsrfTokens: FromRef<S>,
    AppStore: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let locale = Locale::from_request_parts(parts, state).await?;
        let csrf_tokens = CsrfTokens::from_ref(state);
        let political_group = PoliticalGroup::from_request_parts(parts, state).await?;
        let show_success_alert = parts
            .uri
            .query()
            .is_some_and(|q| q.contains("success=true"));
        let overlay_refferer = parts
            .headers
            .get(axum::http::header::REFERER)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|url| url.contains("overlay=true"));

        let election = ElectionConfig::EK2027;
        let long_list_allowed = political_group.long_list_allowed.unwrap_or(false);
        let max_candidates = election.get_max_candidates(long_list_allowed);

        Ok(Context {
            political_group,
            locale,
            show_success_alert,
            overlay_refferer,
            election,
            max_candidates,
            csrf_tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_context_sets_locale() {
        let context = Context::new_test().await;
        assert_eq!(context.locale, Locale::En);
    }

    #[test]
    fn livereload_flag_matches_feature() {
        assert_eq!(Context::livereload_enabled(), cfg!(feature = "livereload"));
    }
}
