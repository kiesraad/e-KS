use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use tower::ServiceExt;

use crate::{
    AppState, AppStore, ElectionConfig, Locale, Session, StreamId,
    persons::PersonId,
    test_utils::{response_body_string, sample_person, sample_political_group},
};

async fn setup() -> (Router, AppStore, String) {
    let state = AppState::new_for_tests().await;
    let app: Router = crate::router::create(state.clone()).with_state(state.clone());

    let stream_id = StreamId::new();
    // Prime the registry with an empty store so `store_middleware` returns the
    // same instance the test uses to seed events (bypassing fixture loading).
    let store = state
        .store_registry
        .get_or_create(stream_id, ElectionConfig::EK27)
        .await
        .expect("store");

    let mut session = Session::new_test_with_locale(Locale::En);
    session.set_stream_id(stream_id);
    session.set_current_election(ElectionConfig::EK27);
    let token = session.token_string();
    state.sessions.insert(session).await;

    (app, store, token)
}

fn get_request(uri: &str, token: &str, store: AppStore) -> Request<Body> {
    let mut request = Request::builder().uri(uri).body(Body::empty()).unwrap();
    request.headers_mut().insert(
        header::COOKIE,
        format!("{}={}", crate::SESSION_COOKIE_NAME, token)
            .parse()
            .unwrap(),
    );
    request.extensions_mut().insert(store);
    request
}

#[tokio::test]
async fn audit_log_route_returns_ok() {
    let (app, store, token) = setup().await;

    let response = app
        .oneshot(get_request("/audit-log", &token, store))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn audit_log_empty_store_shows_empty_message() {
    let (app, store, token) = setup().await;

    let response = app
        .oneshot(get_request("/audit-log", &token, store))
        .await
        .expect("response");

    let body = response_body_string(response).await;
    assert!(body.contains("No events have been recorded yet."));
    assert!(!body.contains("<table"));
}

#[tokio::test]
async fn audit_log_shows_events_after_mutations() {
    let (app, store, token) = setup().await;

    let pg = sample_political_group();
    pg.update(&store).await.unwrap();
    let person = sample_person(PersonId::new());
    let person_name = person.name.display();
    person.create(&store).await.unwrap();

    let response = app
        .oneshot(get_request("/audit-log", &token, store))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body_string(response).await;
    assert!(body.contains("<table"));
    assert!(body.contains("<td>Created person</td>"));
    assert!(body.contains("<td>Updated political group</td>"));
    assert!(body.contains(&person_name));
}

#[tokio::test]
async fn audit_log_contains_audit_log_nav_link() {
    let (app, store, token) = setup().await;

    let response = app
        .oneshot(get_request("/audit-log", &token, store))
        .await
        .expect("response");

    let body = response_body_string(response).await;
    assert!(body.contains(r#"href="/audit-log""#));
    assert!(body.contains("Audit log"));
}

#[tokio::test]
async fn audit_log_pagination_with_query_params() {
    let (app, store, token) = setup().await;

    for _ in 0..25 {
        let person = sample_person(PersonId::new());
        person.create(&store).await.unwrap();
    }

    // Request page 2
    let response = app
        .oneshot(get_request("/audit-log?page=2", &token, store))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body_string(response).await;
    assert!(body.contains("<table"));
    // Page 2 should have the remaining 5 events
    let row_count = body.matches("<td>Created person</td>").count();
    assert_eq!(row_count, 5);
}
