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

/// Builds an app with a POST route behind the session middleware, plus a
/// session; returns the app, its cookie header value, and the raw CSRF token.
async fn post_app_with_session(state: &AppState) -> (Router, String, String) {
    let app = Router::new()
        .route("/", axum::routing::post(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session_middleware,
        ))
        .with_state(state.clone());

    let session = Session::new_test();
    let token = session.token_string();
    let csrf = session.csrf_token().0.clone();
    state.sessions.insert(session).await;

    (app, format!("{SESSION_COOKIE_NAME}={token}"), csrf)
}

fn urlencoded_post(cookie: &str, body: String) -> HttpRequest<Body> {
    HttpRequest::builder()
        .method("POST")
        .uri("/")
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap()
}

/// A POST without a CSRF token gets the styled rejection page.
#[tokio::test]
async fn middleware_rejects_post_without_csrf_token() {
    let state = AppState::new_for_tests().await;
    let (app, cookie, _csrf) = post_app_with_session(&state).await;

    let response = app
        .oneshot(urlencoded_post(&cookie, "a=b".into()))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_body_string(response).await;
    assert!(body.contains("Formulier verlopen"));
}

/// A POST with the session's token reaches the handler.
#[tokio::test]
async fn middleware_accepts_post_with_valid_csrf_token() {
    let state = AppState::new_for_tests().await;
    let (app, cookie, csrf) = post_app_with_session(&state).await;

    let response = app
        .oneshot(urlencoded_post(&cookie, format!("csrf_token={csrf}")))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
}

/// App routes behind session + store middleware whose handler echoes the CSB
/// stream the injected store sends paper corrections to (empty when the store
/// writes to its own stream).
fn store_app(state: AppState) -> Router {
    Router::new()
        .route(
            "/",
            get(|store: crate::PgStore| async move {
                store
                    .paper_corrections_stream_id()
                    .map(|id| id.to_string())
                    .unwrap_or_default()
            }),
        )
        .route("/finalise", get(|| async { "finalise" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            store_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session_middleware,
        ))
        .with_state(state)
}

/// Insert a committee session and return its cookie value. `correcting` sets
/// the paper-corrections stream.
async fn insert_committee_session(state: &AppState, correcting: Option<crate::StreamId>) -> String {
    let mut session = Session::new_test();
    session.set_scope(crate::Scope::CentralElectoralCommittee);
    session.set_current_election(crate::ElectionConfig::EK27);
    session.paper_correction_stream_id = correcting;
    let token = session.token_string();
    state.sessions.insert(session).await;
    format!("{SESSION_COOKIE_NAME}={token}")
}

/// Persist a CSB stream carrying a single import event and return its id.
async fn seed_csb_stream(state: &AppState) -> crate::StreamId {
    let stream_id = crate::StreamId::new();
    let store = state
        .csb_store_for_stream(stream_id, crate::ElectionConfig::EK27)
        .await
        .expect("csb store");
    store
        .update(crate::CsbEvent::Import {
            hash: [0u8; 32],
            source_stream_id: crate::StreamId::new(),
            snapshot: Box::new(crate::PgStoreData::default()),
        })
        .await
        .expect("import");
    stream_id
}

/// A committee session that is not correcting paper documents stays off the
/// app routes.
#[tokio::test]
async fn csb_session_without_correction_stream_is_redirected_to_csb_index() {
    let state = AppState::new_for_tests().await;
    let cookie = insert_committee_session(&state, None).await;

    let response = store_app(state)
        .oneshot(
            HttpRequest::builder()
                .uri("/")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/csb");
}

/// A committee session in paper-corrections mode reaches app routes with a
/// store that writes to the CSB stream.
#[tokio::test]
async fn csb_session_in_corrections_mode_gets_a_paper_corrections_store() {
    let state = AppState::new_for_tests().await;
    let stream_id = seed_csb_stream(&state).await;
    let cookie = insert_committee_session(&state, Some(stream_id)).await;

    let response = store_app(state)
        .oneshot(
            HttpRequest::builder()
                .uri("/")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body_string(response).await;
    assert_eq!(body, stream_id.to_string());
}

/// The finalise flow is not reachable while correcting paper documents.
#[tokio::test]
async fn csb_session_in_corrections_mode_cannot_reach_finalise() {
    let state = AppState::new_for_tests().await;
    let stream_id = seed_csb_stream(&state).await;
    let cookie = insert_committee_session(&state, Some(stream_id)).await;

    let response = store_app(state)
        .oneshot(
            HttpRequest::builder()
                .uri("/finalise")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/");
}
