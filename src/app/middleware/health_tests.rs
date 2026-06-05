use super::health::*;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use crate::{AppState, test_utils::response_body_string};

#[tokio::test]
async fn health_returns_ok_for_reachable_backend() {
    let state = AppState::new_for_tests().await;
    let app: Router = health_router().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body_string(response).await;
    assert_eq!(body, "healthy");
}
