use askama::Template;
use axum::{
    extract::OriginalUri,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use tracing::error;

use crate::{AppError, Context, HtmlTemplate, filters, t};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {}

pub async fn index() -> Result<Html<String>, AppError> {
    IndexTemplate {}.render().map(Html).map_err(|err| {
        error!(?err, "failed to render index template");
        AppError::InternalServerError
    })
}

#[derive(Template)]
#[template(path = "not_found.html")]
pub struct NotFoundTemplate {
    path: String,
}

pub async fn not_found(
    OriginalUri(uri): OriginalUri,
    context: Context,
) -> Result<impl IntoResponse, AppError> {
    let html = HtmlTemplate(
        NotFoundTemplate {
            path: uri.to_string(),
        },
        context,
    );

    Ok((StatusCode::NOT_FOUND, html))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::Html;

    use crate::test_utils::response_body_string;

    #[tokio::test]
    async fn index_renders_html() {
        let Html(body) = index().await.unwrap();
        assert!(body.contains("Hello World!"));
    }

    #[tokio::test]
    async fn not_found_renders_html() {
        let into_response = not_found(
            OriginalUri("/not_found".parse().unwrap()),
            Context::default(),
        )
        .await
        .unwrap();
        let body = response_body_string(into_response.into_response()).await;
        assert!(body.contains("Pagina niet gevonden"));
    }
}
