use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    AppError, AppEvent, AppState, AppStoreData, Locale, PoliticalGroupId, Session,
    auth::session_extractor::build_session_cookie, common::Bsn, political_groups::PoliticalGroup,
    store::Store,
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
    let political_group_id = match query.bsn {
        Some(bsn_str) if !bsn_str.is_empty() => {
            let bsn: Bsn = bsn_str.parse().map_err(|_| AppError::InternalServerError)?;
            state.bsn_id_deriver.derive_political_group_id(&bsn)
        }
        _ => PoliticalGroupId::new(),
    };

    let load_fixtures = query.fixtures.unwrap_or(false);
    let store = ensure_dev_store(&state, political_group_id, load_fixtures).await?;

    store
        .update(AppEvent::DeveloperLogin { political_group_id })
        .await?;

    let locale = request_locale(&headers);
    let mut session = Session::new_with_locale(locale);
    session.set_political_group(political_group_id);

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
    political_group_id: PoliticalGroupId,
    load_fixtures: bool,
) -> Result<Store<AppStoreData>, AppError> {
    let store = state
        .store_registry
        .get_or_create(political_group_id.uuid())
        .await?;
    let store_is_empty = store.data.read().last_event_id == 0;

    if load_fixtures {
        #[cfg(feature = "fixtures")]
        {
            crate::fixtures::load(&store, political_group_id).await?;
            return Ok(store);
        }
    }

    if store_is_empty {
        PoliticalGroup {
            id: political_group_id,
            ..Default::default()
        }
        .create(&store)
        .await?;
    }

    Ok(store)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    use crate::{AppState, common::Bsn, router, test_utils::response_body_string};

    use super::*;

    const TEST_BSN: &str = "999999990";

    fn derive_test_id(state: &AppState, bsn_str: &str) -> PoliticalGroupId {
        let bsn: Bsn = bsn_str.parse().expect("valid test BSN");
        state.bsn_id_deriver.derive_political_group_id(&bsn)
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
                    .uri(format!("/dev/login?bsn={TEST_BSN}&fixtures=false"))
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
            session.political_group_id,
            Some(derive_test_id(&state, TEST_BSN))
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
                    .uri(format!("/dev/login?bsn={TEST_BSN}&fixtures=false"))
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

        let expected_id = derive_test_id(&state, TEST_BSN);
        let store = state
            .store_registry
            .get_or_create(expected_id.uuid())
            .await
            .expect("store");
        assert_eq!(store.get_person_count(), 0);
        assert_eq!(store.get_candidate_list_count(), 0);
        assert_eq!(store.get_political_group().id, expected_id);
    }

    #[tokio::test]
    async fn dev_login_without_fixtures_adds_dev_login_event() {
        let state = AppState::new_for_tests().await;
        let app = router::create(state.clone()).with_state(state.clone());

        let login = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/dev/login?bsn={TEST_BSN}&fixtures=false"))
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

        let expected_id = derive_test_id(&state, TEST_BSN);
        let store = state
            .store_registry
            .get_or_create(expected_id.uuid())
            .await
            .expect("store");

        assert!(matches!(
            store.get_events().as_slice(),
            &[
                AppEvent::UpdatePoliticalGroup(..),
                AppEvent::DeveloperLogin { .. }
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
                    .uri(format!("/dev/login?bsn={TEST_BSN}&fixtures=true"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let expected_id = derive_test_id(&state, TEST_BSN);
        let store = state
            .store_registry
            .get_or_create(expected_id.uuid())
            .await
            .expect("store");
        assert!(store.get_person_count() > 0);
        assert!(store.get_candidate_list_count() > 0);
        assert_eq!(store.get_political_group().id, expected_id);
    }
}
