//! Development-only livereload endpoints and script serving.
//! Merged into the main router when the `livereload` feature is enabled.

use axum::{
    Router,
    http::{
        StatusCode,
        header::{CONTENT_LENGTH, CONTENT_TYPE},
    },
    response::IntoResponse,
};

/// Livereload routes and handlers
pub fn livereload_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/livereload/poll.js", axum::routing::get(poll_js_handler))
        .route("/livereload/poll", axum::routing::get(poll_handler))
        .route("/livereload/healthy", axum::routing::get(healthy_handler))
}

/// Serves the poll.js script
async fn poll_js_handler() -> impl IntoResponse {
    let poll_js = include_str!("../../frontend/scripts/poll.js");

    (
        [
            (CONTENT_TYPE, "text/javascript".to_string()),
            (CONTENT_LENGTH, poll_js.len().to_string()),
        ],
        poll_js,
    )
}

/// Health check endpoint for livereload, requested every 500ms by the livereload.js script when the backend is down
async fn healthy_handler() -> StatusCode {
    StatusCode::OK
}

/// Long-polling endpoint for livereload, requested once by the livereload.js script
async fn poll_handler() -> StatusCode {
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{
            Request, StatusCode,
            header::{CONTENT_LENGTH, CONTENT_TYPE},
        },
    };
    use std::time::Duration;
    use tower::ServiceExt;

    use crate::test_utils::response_body_string;

    #[tokio::test]
    async fn poll_js_serves_script_with_headers() {
        let app = livereload_router::<()>();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/livereload/poll.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers().clone();
        let body = response_body_string(response).await;

        let content_type = headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        assert_eq!(content_type, "text/javascript");

        let content_length = headers
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        assert_eq!(content_length, body.len().to_string());
        assert!(!body.is_empty());
    }

    #[tokio::test]
    async fn healthy_returns_ok() {
        let app = livereload_router::<()>();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/livereload/healthy")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test(start_paused = true)]
    async fn poll_returns_ok_after_delay() {
        let app = livereload_router::<()>();

        let response_task = tokio::spawn(async move {
            app.oneshot(
                Request::builder()
                    .uri("/livereload/poll")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response")
        });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(30)).await;

        let response = response_task.await.expect("join");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
