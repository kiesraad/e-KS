use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    response::Response,
};
use tower::ServiceExt;

use secrecy::SecretString;

use crate::{
    AppEvent, AppState, AppStore, CsbEvent, CsbMainEvent, ElectionConfig, Locale, Scope, Session,
    StreamId, router, store::StoreEvent, test_utils::response_body_string,
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

/// The session cookie's `name=value` pair.
fn cookie_value(response: &Response) -> &str {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| value.split(';').next())
        .find(|pair| pair.starts_with(crate::SESSION_COOKIE_NAME))
        .expect("session cookie value")
}

/// Resolve the session that a dev-login response established.
async fn session_from(state: &AppState, response: &Response) -> Session {
    let token = cookie_value(response)
        .split_once('=')
        .map(|(_, value)| value)
        .expect("session token");
    state
        .sessions
        .get(token)
        .await
        .expect("load session")
        .expect("session")
}

/// Open the per-stream store for the dev-login test user.
async fn open_store(state: &AppState) -> AppStore {
    let expected_id = derive_test_id(state, TEST_ID_CODE);
    AppStore::own(
        state
            .store_registry
            .get_or_create(expected_id, ElectionConfig::EK27)
            .await
            .expect("store"),
    )
}

/// Log in with the given dev-login request, then load `path` with the
/// session cookie and return the response.
async fn login_then_get(app: axum::Router, login_request: Request<Body>, path: &str) -> Response {
    let login = app.clone().oneshot(login_request).await.expect("response");

    app.oneshot(
        Request::builder()
            .uri(path)
            .header(header::COOKIE, cookie_value(&login))
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .expect("response")
}

/// Dev-login without fixtures and then load the home page, asserting it
/// rendered successfully.
async fn login_without_fixtures_then_home(app: axum::Router) {
    let response = login_then_get(app, dev_login_request("fixtures=false"), "/").await;

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
async fn dev_login_csb_without_fixtures_adds_dev_login_event() {
    let (state, app) = test_app().await;

    app.oneshot(dev_login_csb_request("fixtures=false"))
        .await
        .expect("response");

    let store = state
        .csb_main_store(ElectionConfig::EK27)
        .await
        .expect("main store");

    assert!(matches!(
        store.data.read().events.as_slice(),
        &[StoreEvent {
            payload: CsbMainEvent::DeveloperLogin { .. },
            ..
        }]
    ));
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

#[cfg(feature = "fixtures")]
#[tokio::test]
async fn dev_login_csb_with_fixtures_creates_imported_stream() {
    let (state, app) = test_app().await;

    let response = app
        .oneshot(dev_login_csb_request("fixtures=true"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let csb_stores = state
        .csb_store_registry
        .stores_by_scope()
        .await
        .expect("csb stores");

    assert_eq!(csb_stores.len(), 1);
    let csb_store = &csb_stores[0];

    let events = csb_store.data.read().events.clone();
    assert_eq!(events.len(), 1);

    let event = events[0].clone();
    let StoreEvent {
        payload: CsbEvent::Import { hash, snapshot, .. },
        ..
    } = event
    else {
        panic!("unexpected event: {event:?}");
    };

    assert_eq!(hash, crate::csb::import::fixture::FIXTURE_IMPORT_HASH);
    assert!(!snapshot.persons.is_empty());
    assert!(!snapshot.candidate_lists.is_empty());
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

    let response =
        login_then_get(app, dev_login_csb_request("fixtures=false"), "/csb/import").await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body_string(response).await;
    assert!(body.contains("Import"));
}

/// A political-group session is rejected from CSB routes.
#[tokio::test]
async fn csb_import_rejected_for_political_group_session() {
    let (_state, app) = test_app().await;

    let response = login_then_get(app, dev_login_request("fixtures=false"), "/csb/import").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// A committee session is kept off app routes (which use app stores) and sent
/// to its CSB import page instead.
#[tokio::test]
async fn committee_session_redirected_off_app_routes() {
    let (_state, app) = test_app().await;

    let response = login_then_get(app, dev_login_csb_request("fixtures=false"), "/").await;

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/csb");
}
