use askama::Template;
use axum::response::{IntoResponse, Response};

use crate::{AppError, Context, CsbContext, HtmlTemplate, csb::index::CsbIndexPath, filters};

#[derive(Template)]
#[template(path = "csb/index/pages/index.html")]
struct CsbIndexTemplate;

pub async fn index(_: CsbIndexPath, context: CsbContext) -> Result<Response, AppError> {
    Ok(HtmlTemplate(CsbIndexTemplate, context).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    use crate::test_utils::response_body_string;

    #[tokio::test]
    async fn index_renders_all_phase_titles() {
        let response = index(CsbIndexPath {}, CsbContext::new_test())
            .await
            .unwrap()
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Pre-submission"));
        assert!(body.contains("Examination"));
        assert!(body.contains("Rectified lists"));
        assert!(body.contains("List numbering"));
        assert!(body.contains("Finalise candidate lists"));
    }

    #[tokio::test]
    async fn index_links_active_phase_to_examination() {
        let response = index(CsbIndexPath {}, CsbContext::new_test())
            .await
            .unwrap()
            .into_response();

        let body = response_body_string(response).await;
        assert!(body.contains("href=\"/csb/examination\""));
        assert!(body.contains("Go to examination"));
    }
}
