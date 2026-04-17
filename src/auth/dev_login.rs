use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use secrecy::SecretString;
use serde::Deserialize;

use crate::{
    AppError, AppEvent, AppState, AppStoreData, ElectionConfig, Locale, Session, StreamId,
    auth::session_extractor::build_session_cookie, political_groups::PoliticalGroup, store::Store,
};

pub const DEV_LOGIN_PATH: &str = "/dev/login";

#[derive(Debug, Deserialize)]
pub struct DevLoginQuery {
    bsn: Option<String>,
    fixtures: Option<bool>,
}

pub async fn dev_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<DevLoginQuery>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let election = ElectionConfig::EK27;
    let id_code: Option<SecretString> = query
        .bsn
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(SecretString::from);

    let stream_id = match &id_code {
        Some(id_code) => state.id_deriver.derive_stream_id(id_code, election),
        None => StreamId::new(),
    };

    let load_fixtures = query.fixtures.unwrap_or(false);
    let (store, was_new) = ensure_dev_store(&state, stream_id, load_fixtures, election).await?;

    if was_new {
        store.update(AppEvent::DeveloperLogin { stream_id }).await?;
    }

    let locale = request_locale(&headers);
    let mut session = Session::new_with_locale(locale);
    session.set_stream_id(stream_id);
    session.id_code = id_code;

    state.sessions.cleanup_expired();
    state.sessions.insert(session.clone());

    Ok((jar.add(build_session_cookie(&session)), Redirect::to("/")))
}

pub(crate) fn request_locale(headers: &axum::http::HeaderMap) -> Locale {
    headers
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
        .and_then(Locale::from_accept_language)
        .unwrap_or_default()
}

async fn ensure_dev_store(
    state: &AppState,
    stream_id: StreamId,
    load_fixtures: bool,
    election: ElectionConfig,
) -> Result<(Store<AppStoreData>, bool), AppError> {
    let store = state
        .store_registry
        .get_or_create(stream_id.uuid(), election)
        .await?;
    let store_is_empty = store.data.read().events.is_empty();

    if store_is_empty {
        PoliticalGroup::default().create(&store).await?;
    }

    if load_fixtures {
        #[cfg(feature = "fixtures")]
        {
            crate::fixtures::load(&store, election).await?;
            return Ok((store, store_is_empty));
        }
    }

    Ok((store, store_is_empty))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    use crate::{AppState, router, store::StoreEvent, test_utils::response_body_string};

    use super::*;

    const TEST_ID_CODE: &str = "999999990";

    fn derive_test_id(state: &AppState, id_code_str: &str) -> StreamId {
        let id_code: SecretString = id_code_str.into();
        state
            .id_deriver
            .derive_stream_id(&id_code, ElectionConfig::EK27)
    }

    fn cookie_value(response: &axum::response::Response) -> &str {
        response
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .expect("cookie value")
    }

    #[tokio::test]
    async fn dev_login_sets_cookie_and_redirects_home() {
        let state = AppState::new_for_tests().await;
        let app = router::create(state.clone()).with_state(state.clone());

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

        let token = cookie_value(&response)
            .split_once('=')
            .map(|(_, value)| value)
            .expect("session token");
        let session = state.sessions.get(token).expect("session");
        assert_eq!(session.locale, Locale::En);
        assert_eq!(
            session.stream_id,
            Some(derive_test_id(&state, TEST_ID_CODE))
        );
    }

    #[tokio::test]
    async fn dev_login_without_fixtures_keeps_store_empty() {
        let state = AppState::new_for_tests().await;
        let app = router::create(state.clone()).with_state(state.clone());

        let login = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/dev/login?bsn={TEST_ID_CODE}&fixtures=false"))
                    .body(Body::empty())
                    .unwrap(),
            )
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

        let expected_id = derive_test_id(&state, TEST_ID_CODE);
        let store = state
            .store_registry
            .get_or_create(expected_id.uuid(), ElectionConfig::EK27)
            .await
            .expect("store");
        assert_eq!(store.get_person_count(), 0);
        assert_eq!(store.get_candidate_list_count(), 0);
    }

    #[tokio::test]
    async fn dev_login_without_fixtures_adds_dev_login_event() {
        let state = AppState::new_for_tests().await;
        let app = router::create(state.clone()).with_state(state.clone());

        let login = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/dev/login?bsn={TEST_ID_CODE}&fixtures=false"))
                    .body(Body::empty())
                    .unwrap(),
            )
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

        let expected_id = derive_test_id(&state, TEST_ID_CODE);
        let store = state
            .store_registry
            .get_or_create(expected_id.uuid(), ElectionConfig::EK27)
            .await
            .expect("store");

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

    #[cfg(feature = "fixtures")]
    #[tokio::test]
    async fn dev_login_with_fixtures_loads_fixture_data() {
        let state = AppState::new_for_tests().await;
        let app = router::create(state.clone()).with_state(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/dev/login?bsn={TEST_ID_CODE}&fixtures=true"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let expected_id = derive_test_id(&state, TEST_ID_CODE);
        let store = state
            .store_registry
            .get_or_create(expected_id.uuid(), ElectionConfig::EK27)
            .await
            .expect("store");
        assert!(store.get_person_count() > 0);
        assert!(store.get_candidate_list_count() > 0);
    }
}
