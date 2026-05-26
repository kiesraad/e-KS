use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    response::Response,
};
use tower::ServiceExt;

use secrecy::SecretString;

use crate::{
    AppEvent, AppState, AppStore, ElectionConfig, Locale, Session, StreamId, router,
    store::StoreEvent, test_utils::response_body_string,
};

const TEST_ID_CODE: &str = "999999990";

fn derive_test_id(state: &AppState, id_code_str: &str) -> StreamId {
    let id_code: SecretString = id_code_str.into();
    state.id_deriver.derive_stream_id(&id_code)
}

/// Build a fresh test state and a router wired to it.
async fn test_app() -> (AppState, axum::Router) {
    let state = AppState::new_for_tests().await;
    let app = router::create(state.clone()).with_state(state.clone());
    (state, app)
}

/// A `GET /dev/login` request for the test user with the given extra query.
fn dev_login_request(query: &str) -> Request<Body> {
    Request::builder()
        .uri(format!("/dev/login?bsn={TEST_ID_CODE}&{query}"))
        .body(Body::empty())
        .unwrap()
}

fn cookie_value(response: &Response) -> &str {
    response
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .expect("cookie value")
}

/// Resolve the session that a dev-login response established.
async fn session_from(state: &AppState, response: &Response) -> Session {
    let token = cookie_value(response)
        .split_once('=')
        .map(|(_, value)| value)
        .expect("session token");
    state.sessions.get(token).await.expect("session")
}

/// Open the per-stream store for the dev-login test user.
async fn open_store(state: &AppState) -> AppStore {
    let expected_id = derive_test_id(state, TEST_ID_CODE);
    state
        .store_registry
        .get_or_create(expected_id.uuid(), ElectionConfig::EK27)
        .await
        .expect("store")
}

/// Dev-login without fixtures and then load the home page, asserting it
/// rendered successfully.
async fn login_without_fixtures_then_home(app: axum::Router) {
    let login = app
        .clone()
        .oneshot(dev_login_request("fixtures=false"))
        .await
        .expect("response");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header(header::COOKIE, cookie_value(&login))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body_string(response).await;
    assert!(body.contains("Kiesraad - Kandidaatstelling"));
}

#[tokio::test]
async fn dev_login_sets_cookie_and_redirects_home() {
    let (state, app) = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/dev/login?bsn={TEST_ID_CODE}&fixtures=false"))
                .header(header::ACCEPT_LANGUAGE, "en")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/");

    let session = session_from(&state, &response).await;
    assert_eq!(session.locale, Locale::En);
    assert_eq!(
        session.stream_id,
        Some(derive_test_id(&state, TEST_ID_CODE))
    );
}

#[tokio::test]
async fn dev_login_without_fixtures_keeps_store_empty() {
    let (state, app) = test_app().await;
    login_without_fixtures_then_home(app).await;

    let store = open_store(&state).await;
    assert_eq!(store.get_person_count(), 0);
    assert_eq!(store.get_candidate_list_count(), 0);
}

#[tokio::test]
async fn dev_login_without_fixtures_adds_dev_login_event() {
    let (state, app) = test_app().await;
    login_without_fixtures_then_home(app).await;

    let store = open_store(&state).await;
    assert!(matches!(
        store.get_events().as_slice(),
        &[
            StoreEvent {
                payload: AppEvent::UpdatePoliticalGroup(..),
                ..
            },
            StoreEvent {
                payload: AppEvent::DeveloperLogin { .. },
                ..
            }
        ],
    ))
}

#[tokio::test]
async fn dev_login_select_election_skips_election_setup() {
    let (state, app) = test_app().await;

    let response = app
        .oneshot(dev_login_request("select_election=true"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/select-election"
    );

    let session = session_from(&state, &response).await;
    assert_eq!(
        session.stream_id,
        Some(derive_test_id(&state, TEST_ID_CODE))
    );
    assert_eq!(session.current_election, None);
}

#[cfg(feature = "fixtures")]
#[tokio::test]
async fn dev_login_with_fixtures_loads_fixture_data() {
    let (state, app) = test_app().await;

    let response = app
        .oneshot(dev_login_request("fixtures=true"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let store = open_store(&state).await;
    assert!(store.get_person_count() > 0);
    assert!(store.get_candidate_list_count() > 0);
}
