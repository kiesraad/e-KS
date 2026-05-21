use super::session_extractor::*;
use axum::{
    Router,
    body::Body,
    http::{Request as HttpRequest, StatusCode, header},
    middleware,
    routing::get,
};
use tower::ServiceExt;

use crate::{AppState, Session, test_utils::response_body_string};

/// Redirects to /login when no session cookie is present.
#[tokio::test]
async fn middleware_redirects_to_login_without_cookie() {
    let state = AppState::new_for_tests().await;
    let app = Router::new()
        .route(
            "/",
            get(|session: Session| async move { session.token().to_exposed_string() }),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session_middleware,
        ))
        .with_state(state);

    let response = app
        .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/login");
}

/// Reuses the existing session when the cookie is provided.
#[tokio::test]
async fn middleware_reuses_session_with_cookie() {
    let state = AppState::new_for_tests().await;
    let app = Router::new()
        .route(
            "/",
            get(|session: Session| async move { session.token().to_exposed_string() }),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session_middleware,
        ))
        .with_state(state.clone());

    let session = Session::new_test();
    let token = session.token().to_exposed_string();
    state.sessions.insert(session).await;
    let cookie_value = format!("{SESSION_COOKIE_NAME}={token}");

    let response = app
        .oneshot(
            HttpRequest::builder()
                .uri("/")
                .header(header::COOKIE, &cookie_value)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    let status = response.status();
    let sets_cookie = response.headers().get(header::SET_COOKIE).is_some();
    let returned_token = response_body_string(response).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!sets_cookie);
    assert_eq!(returned_token, token);
}
