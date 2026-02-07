use askama::Template;
use axum::response::IntoResponse;

use crate::{Context, HtmlTemplate, filters, submit::pages::SubmitPath};

#[derive(Template)]
#[template(path = "submit/index.html")]
pub struct IndexTemplate {}

pub async fn index(_: SubmitPath, context: Context) -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {}, context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::response_body_string;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn index_renders_under_construction() {
        let response = index(SubmitPath, Context::new_test().await)
            .await
            .into_response();
        let body = response_body_string(response).await;
        assert!(body.contains("Under construction"));
    }
}
