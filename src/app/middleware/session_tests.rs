use super::session::*;
use crate::auth::{session::hash_token, session_extractor::user_agent_hash};
use axum::{
    Router,
    body::Body,
    http::{Request as HttpRequest, StatusCode, header},
    middleware,
    routing::get,
};
use tower::ServiceExt;

use crate::{AppState, SESSION_COOKIE_NAME, Session, test_utils::response_body_string};

/// Redirects to the login start page when no session cookie is present.
#[tokio::test]
async fn middleware_redirects_to_login_without_cookie() {
    let state = AppState::new_for_tests().await;
    let app = Router::new()
        .route(
            "/",
            get(|session: Session| async move { session.token_hash().to_string() }),
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
            get(|session: Session| async move { session.token_hash().to_string() }),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session_middleware,
        ))
        .with_state(state.clone());

    let session = Session::new_test();
    let token = session.token_string();
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
    let returned_hash = response_body_string(response).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!sets_cookie);
    // Session reused: the handler saw it, keyed by the cookie token's hash.
    assert_eq!(returned_hash, hash_token(&token));
}

/// Builds an app whose only route echoes the session token hash behind the
/// session middleware.
fn session_app(state: AppState) -> Router {
    Router::new()
        .route(
            "/",
            get(|session: Session| async move { session.token_hash().to_string() }),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session_middleware,
        ))
        .with_state(state)
}

/// Inserts a session pinned to `user_agent` and returns its raw cookie token.
async fn insert_pinned_session(state: &AppState, user_agent: &str) -> String {
    let mut ua_headers = axum::http::HeaderMap::new();
    ua_headers.insert(header::USER_AGENT, user_agent.parse().unwrap());

    let mut session = Session::new_test();
    session.set_user_agent_hash(user_agent_hash(&ua_headers));
    let token = session.token_string();
    state.sessions.insert(session).await;
    token
}

/// Sets up an app with a session pinned to `browser-1`, then replays the cookie
/// with `request_user_agent`. Returns the state, raw token, and the response.
async fn replay_pinned_session_with_user_agent(
    request_user_agent: &str,
) -> (AppState, String, axum::response::Response) {
    let state = AppState::new_for_tests().await;
    let app = session_app(state.clone());
    let token = insert_pinned_session(&state, "browser-1").await;

    let response = app
        .oneshot(
            HttpRequest::builder()
                .uri("/")
                .header(header::COOKIE, format!("{SESSION_COOKIE_NAME}={token}"))
                .header(header::USER_AGENT, request_user_agent)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    (state, token, response)
}

/// A cookie replayed with a different User-Agent is rejected and dropped.
#[tokio::test]
async fn middleware_rejects_mismatched_user_agent() {
    let (state, token, response) = replay_pinned_session_with_user_agent("browser-2").await;

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/login");
    assert!(
        state
            .sessions
            .get_existing(Some(&token))
            .await
            .expect("load session")
            .is_none(),
        "a session rejected on UA mismatch must be dropped"
    );
}

/// The same client (matching User-Agent) is accepted and the session reused.
#[tokio::test]
async fn middleware_accepts_matching_user_agent() {
    let (_state, token, response) = replay_pinned_session_with_user_agent("browser-1").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_body_string(response).await, hash_token(&token));
}
