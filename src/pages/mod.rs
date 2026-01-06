use askama::Template;
use axum::{extract::OriginalUri, http::StatusCode, response::IntoResponse};

use crate::{AppError, Context, HtmlTemplate, filters, t};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {}

pub async fn index(context: Context) -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {}, context)
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

    use crate::test_utils::response_body_string;

    #[tokio::test]
    async fn index_renders_html() {
        let body = index(Context::default()).await.into_response();
        let body = response_body_string(body).await;
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
