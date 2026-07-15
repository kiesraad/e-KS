use std::fmt;

use axum_extra::routing::TypedPath;

use super::QueryParamState;

/// Navigation context for an overlay, carrying an optional `redirect_to` URL
#[derive(Default)]
pub struct Overlay {
    redirect_to: Option<String>,
}

impl Overlay {
    pub fn new(query: &QueryParamState) -> Self {
        Self {
            redirect_to: query.redirect_url().map(str::to_string),
        }
    }

    /// Returns `redirect_to` if set, otherwise the given default path
    pub fn close_url(&self, default: impl fmt::Display) -> String {
        self.redirect_to
            .as_deref()
            .map(str::to_string)
            .unwrap_or_else(|| default.to_string())
    }

    /// Returns `path` with `?redirect_to=<value>` appended when a redirect is
    /// set, so the target step can return to the right place after saving
    pub fn forward(&self, path: impl TypedPath) -> String {
        match &self.redirect_to {
            Some(r) => path
                .with_query_params(QueryParamState::redirect_to(r.clone()))
                .to_string(),
            None => path.to_string(),
        }
    }
}
