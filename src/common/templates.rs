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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Locale;

    use askama::Template;
    use axum::http::StatusCode;

    mod filters {
        #[askama::filter_fn]
        pub fn foo(_val: u32, _values: &dyn askama::Values) -> askama::Result<&'static str> {
            Err(askama::Error::Fmt)
        }
    }

    #[derive(Template)]
    #[template(source = "{{ 123|foo }}", ext = "txt")]
    struct MyTemplate;

    #[test]
    fn html_template_returns_500_when_render_fails() {
        let context = Context::new(Locale::En);
        let response = HtmlTemplate(MyTemplate, context).into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
