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

    pub fn highlight_last_success(last: usize) -> Self {
        Self {
            success: true,
            highlight_last: Some(last),
            ..Default::default()
        }
    }

    /// Redirect to `redirect_to` query param if present (and a valid relative path),
    /// otherwise redirect to the default path with success query params.
    pub fn redirect_or(&self, default: impl std::fmt::Display) -> Response {
        let mut url = match &self.redirect_to {
            Some(url) if url.starts_with('/') => url.clone(),
            _ => default.to_string(),
        };

        if !url.contains('?') {
            url.push_str("?&success=true");
        }

        Redirect::to(&url).into_response()
    }
}
