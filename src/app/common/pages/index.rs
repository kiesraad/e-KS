use askama::Template;
use axum::response::IntoResponse;

use super::IndexPath;
use crate::{Context, HtmlTemplate, filters};

#[derive(Template)]
#[template(path = "common/pages/index.html")]
pub struct IndexTemplate {}

pub async fn index(_: IndexPath, context: Context) -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {}, context)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{ElectionConfig, core::AnyLocale, test_utils::response_body_string};

    #[tokio::test]
    async fn index_renders_html() {
        let body = index(IndexPath, Context::new_test_without_db())
            .await
            .into_response();
        let body = response_body_string(body).await;
        assert!(body.contains(ElectionConfig::default().title(AnyLocale::En)));
    }
}
