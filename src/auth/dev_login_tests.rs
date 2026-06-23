use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    response::Response,
};
use tower::ServiceExt;

use secrecy::SecretString;

use crate::{
    AppEvent, AppState, AppStore, ElectionConfig, Locale, Province, Scope, Session, StreamId,
    router, store::StoreEvent, test_utils::response_body_string,
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

#[tokio::test]
async fn dev_login_scopes_session_to_political_group() {
    let (state, app) = test_app().await;

    let response = app
        .oneshot(dev_login_request("fixtures=false"))
        .await
        .expect("response");

    let session = session_from(&state, &response).await;
    assert_eq!(session.scope, Scope::PoliticalGroup);
}

/// A `GET /dev/login?csb=true` request for the given query.
fn dev_login_csb_request(query: &str) -> Request<Body> {
    Request::builder()
        .uri(format!("/dev/login?csb=true&bsn={TEST_ID_CODE}&{query}"))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn dev_login_csb_scopes_session_to_committee() {
    let (state, app) = test_app().await;

    let response = app
        .oneshot(dev_login_csb_request("fixtures=false"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    // Committee members land on their CSB page, not the app home.
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/csb");

    let session = session_from(&state, &response).await;
    assert_eq!(session.scope, Scope::CentralElectoralCommittee);
    assert_eq!(
        session.stream_id,
        Some(derive_test_id(&state, TEST_ID_CODE))
    );
    assert_eq!(session.current_election, Some(ElectionConfig::EK27));
}

/// A committee session can reach the CSB import page.
#[tokio::test]
async fn csb_import_reachable_for_committee_session() {
    let (_state, app) = test_app().await;

    let login = app
        .clone()
        .oneshot(dev_login_csb_request("fixtures=false"))
        .await
        .expect("response");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/csb/import")
                .header(header::COOKIE, cookie_value(&login))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body_string(response).await;
    assert!(body.contains("Import"));
}

/// A political-group session is rejected from CSB routes.
#[tokio::test]
async fn csb_import_rejected_for_political_group_session() {
    let (_state, app) = test_app().await;

    let login = app
        .clone()
        .oneshot(dev_login_request("fixtures=false"))
        .await
        .expect("response");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/csb/import")
                .header(header::COOKIE, cookie_value(&login))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// A committee session is kept off app routes (which use app stores) and sent
/// to its CSB import page instead.
#[tokio::test]
async fn committee_session_redirected_off_app_routes() {
    let (_state, app) = test_app().await;

    let login = app
        .clone()
        .oneshot(dev_login_csb_request("fixtures=false"))
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

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/csb");
}

/// A political-group session may reach every election under its own stream. A
/// stream is a `(stream_id, election)` pair, so multiple streams sharing one
/// `stream_id` are returned.
#[tokio::test]
async fn accessible_streams_for_political_group_lists_all_its_elections() {
    let state = AppState::new_for_tests().await;
    let stream_id = StreamId::new();

    // Two streams under the same stream_id, each with an event so the registry
    // reports them.
    for election in [ElectionConfig::EK27, ElectionConfig::PS27(Province::GE)] {
        let store = state
            .store_for_stream(stream_id, election, false)
            .await
            .expect("store");
        store
            .update(AppEvent::DeveloperLogin { stream_id })
            .await
            .expect("event");
    }

    let mut session = Session::new_test();
    session.set_stream_id(stream_id);
    session.set_scope(Scope::PoliticalGroup);

    let mut accessible = state
        .accessible_streams(&session)
        .await
        .expect("accessible streams");
    accessible.sort_by_key(|(_, election)| election.stable_id());

    assert_eq!(
        accessible,
        vec![
            (stream_id.uuid(), ElectionConfig::EK27),
            (stream_id.uuid(), ElectionConfig::PS27(Province::GE)),
        ]
    );
}
