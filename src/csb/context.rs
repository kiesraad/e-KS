//! Request-scoped template context for the CSB domain, carrying locale and
//! helpers. Extracted from requests and passed into Askama templates.

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{AppError, AppRequestState, Session};

#[cfg(test)]
use crate::Locale;

/// Request-scoped template context used by CSB Askama templates.
#[derive(Clone)]
pub struct CsbContext {
    /// Session data for locale and CSRF.
    pub session: Session,
    /// Short identifier of the server this instance runs on (e.g. "S1"),
    /// rendered next to the version in the layout footer when set.
    pub server_name: Option<&'static str>,
    /// Whether to show the success alert based on the request query.
    pub show_success_alert: bool,
}

impl CsbContext {
    pub fn new(session: Session) -> Self {
        Self {
            session,
            server_name: None,
            show_success_alert: false,
        }
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        Self::new(Session::new_test_with_locale(Locale::En))
    }

    pub fn livereload_enabled() -> bool {
        cfg!(feature = "livereload")
    }
}

impl askama::Values for CsbContext {
    fn get_value<'a>(&'a self, key: &str) -> Option<&'a dyn std::any::Any> {
        match key {
            "locale" => Some(&self.session.locale as &dyn std::any::Any),
            "server_name" => Some(&self.server_name as &dyn std::any::Any),
            "show_success_alert" => Some(&self.show_success_alert as &dyn std::any::Any),
            _ => None,
        }
    }
}

impl<S: AppRequestState> FromRequestParts<S> for CsbContext {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;
        let mut context = CsbContext::new(session);

        context.server_name = state.config().server_name.as_deref();

        context.show_success_alert = parts
            .uri
            .query()
            .is_some_and(|q| q.contains("success=true"));

        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_context_carries_session_locale() {
        let context = CsbContext::new_test();
        assert_eq!(context.session.locale, Locale::En);
    }

    #[test]
    fn livereload_flag_matches_feature() {
        assert_eq!(
            CsbContext::livereload_enabled(),
            cfg!(feature = "livereload")
        );
    }
}
