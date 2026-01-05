//! Askama template wrapper for Axum responses.
//! Used by handlers to render templates with Context.

use askama::Template;
use axum::response::{Html, IntoResponse, Response};

use crate::{AppError, Context};

pub struct HtmlTemplate<T>(pub T, pub Context);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render_with_values(&self.1) {
            Ok(html) => Html(html).into_response(),
            Err(err) => AppError::TemplateError(err).into_response(),
        }
    }
}
