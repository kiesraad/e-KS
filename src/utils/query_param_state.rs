//! Query parameter state for UI feedback and highlighting.
use axum::response::{IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, Serialize, Deserialize)]
pub struct QueryParamState {
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    initial: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    highlight: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    highlight_last: Option<usize>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_to: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    max_candidates_reached: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    import_capped: bool,
}

impl QueryParamState {
    pub fn is_initial(&self) -> bool {
        self.initial
    }

    pub fn should_warn(&self) -> bool {
        !self.initial
    }

    pub fn get_highlight(&self) -> Option<Uuid> {
        self.highlight
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn is_max_candidates_reached(&self) -> bool {
        self.max_candidates_reached
    }

    pub fn is_import_capped(&self) -> bool {
        self.import_capped
    }

    pub fn initial() -> Self {
        Self {
            initial: true,
            ..Default::default()
        }
    }

    pub fn created() -> Self {
        Self {
            initial: true,
            success: true,
            ..Default::default()
        }
    }

    pub fn success() -> Self {
        Self {
            success: true,
            ..Default::default()
        }
    }

    pub fn highlight(id: Uuid) -> Self {
        Self {
            highlight: Some(id),
            ..Default::default()
        }
    }

    pub fn highlight_success(id: Uuid) -> Self {
        Self {
            success: true,
            highlight: Some(id),
            ..Default::default()
        }
    }

    pub fn highlight_last(last: usize) -> Self {
        Self {
            highlight_last: Some(last),
            ..Default::default()
        }
    }

    pub fn highlight_last_success(last: usize) -> Self {
        Self {
            success: true,
            highlight_last: Some(last),
            ..Default::default()
        }
    }

    pub fn max_candidates_reached() -> Self {
        Self {
            max_candidates_reached: true,
            ..Default::default()
        }
    }

    pub fn import_capped() -> Self {
        Self {
            import_capped: true,
            ..Default::default()
        }
    }

    pub fn redirect_to(url: String) -> Self {
        Self {
            redirect_to: Some(url),
            ..Default::default()
        }
    }

    pub fn redirect_url(&self) -> Option<&str> {
        self.redirect_to
            .as_deref()
            .filter(|url| url.starts_with('/'))
    }

    /// Builds the redirect URL: the `redirect_to` query param if present (and a
    /// valid relative path), otherwise the default path with success query params.
    fn redirect_url_or(&self, default: impl std::fmt::Display) -> String {
        let mut url = match &self.redirect_to {
            Some(url) if url.starts_with('/') => url.clone(),
            _ => default.to_string(),
        };

        if !url.contains('?') {
            url.push_str("?&success=true");
        }

        url
    }

    /// Redirect to `redirect_to` query param if present (and a valid relative path),
    /// otherwise redirect to the default path with success query params.
    pub fn redirect_or(&self, default: impl std::fmt::Display) -> Response {
        Redirect::to(&self.redirect_url_or(default)).into_response()
    }

    /// Like `redirect_or`, but preserves `initial=true` in the redirect URL when set.
    /// Use this for inter-step saves within the general information section.
    pub fn redirect_or_preserving_initial(&self, default: impl std::fmt::Display) -> Response {
        let mut url = self.redirect_url_or(default);

        if self.initial {
            url.push_str("&initial=true");
        }

        Redirect::to(&url).into_response()
    }
}
