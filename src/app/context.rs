//! Request-scoped template context carrying locale and helpers.
//! Extracted from requests and passed into Askama templates.

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{AppError, Session, political_groups::PoliticalGroup};

#[cfg(test)]
use crate::Locale;

/// Request-scoped template context used by Askama.
#[derive(Clone)]
pub struct Context {
    /// Political group tied to the session / request.
    pub political_group: PoliticalGroup,
    /// Maximum number of candidates allowed for this political group.
    pub max_candidates: usize,
    /// Whether to show the success alert based on the request query.
    pub show_success_alert: bool,
    /// Whether the request came from an overlay page (via referrer query).
    pub overlay_referrer: bool,
    /// Session data for locale, CSRF, and election configuration.
    pub session: Session,
}

impl Context {
    pub fn new(political_group: PoliticalGroup, session: Session) -> Self {
        let election = session.election;
        let long_list_allowed = political_group.long_list_allowed.unwrap_or(false);

        Self {
            political_group,
            max_candidates: election.get_max_candidates(long_list_allowed),
            show_success_alert: false,
            overlay_referrer: false,
            session,
        }
    }

    #[cfg(test)]
    pub async fn new_test() -> Self {
        let political_group = PoliticalGroup::default();

        Self::new(political_group, Session::new_with_locale(Locale::En))
    }

    #[cfg(test)]
    pub fn new_test_without_db() -> Self {
        Self::new(
            PoliticalGroup::default(),
            Session::new_with_locale(Locale::En),
        )
    }

    pub fn livereload_enabled() -> bool {
        cfg!(feature = "livereload")
    }
}

impl askama::Values for Context {
    fn get_value<'a>(&'a self, key: &str) -> Option<&'a dyn std::any::Any> {
        match key {
            "locale" => Some(&self.session.locale as &dyn std::any::Any),
            "election" => Some(&self.session.election as &dyn std::any::Any),
            "max_candidates" => Some(&self.max_candidates as &dyn std::any::Any),
            "show_success_alert" => Some(&self.show_success_alert as &dyn std::any::Any),
            "overlay_referrer" => Some(&self.overlay_referrer as &dyn std::any::Any),
            _ => None,
        }
    }
}

impl<S> FromRequestParts<S> for Context
where
    S: Clone + Send + Sync + 'static,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;
        let political_group = PoliticalGroup::from_request_parts(parts, state).await?;
        let show_success_alert = parts
            .uri
            .query()
            .is_some_and(|q| q.contains("success=true"));
        let overlay_referrer = parts
            .headers
            .get(axum::http::header::REFERER)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|url| url.contains("overlay=true"));

        let long_list_allowed = political_group.long_list_allowed.unwrap_or(false);
        let max_candidates = session.election.get_max_candidates(long_list_allowed);

        Ok(Context {
            political_group,
            show_success_alert,
            overlay_referrer,
            max_candidates,
            session,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_context_sets_locale() {
        let context = Context::new_test().await;
        assert_eq!(context.session.locale, Locale::En);
    }

    #[test]
    fn livereload_flag_matches_feature() {
        assert_eq!(Context::livereload_enabled(), cfg!(feature = "livereload"));
    }
}
