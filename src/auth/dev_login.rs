use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppError, AppState, Locale, PoliticalGroupId, Session,
    auth::session_extractor::build_session_cookie, form::ValidationError,
    political_groups::PoliticalGroup,
};

pub const DEV_LOGIN_PATH: &str = "/dev/login";

#[derive(Debug, Deserialize)]
pub struct DevLoginQuery {
    name: String,
    fixtures: bool,
}

pub async fn dev_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<DevLoginQuery>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let name = query.name.trim();
    if name.is_empty() {
        return Err(AppError::ValidationError(vec![(
            "name".to_string(),
            ValidationError::InvalidValue,
        )]));
    }

    let political_group_id = dev_political_group_id(name);
    ensure_dev_store(&state, political_group_id, query.fixtures).await?;

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

fn dev_political_group_id(name: &str) -> PoliticalGroupId {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("eks/dev-login/{name}").as_bytes(),
    )
    .into()
}

async fn ensure_dev_store(
    state: &AppState,
    political_group_id: PoliticalGroupId,
    load_fixtures: bool,
) -> Result<(), AppError> {
    let store = state
        .store_registry
        .get_or_create(political_group_id.uuid())
        .await?;
    let store_is_empty = store.data.read().last_event_id == 0;

    if load_fixtures {
        #[cfg(feature = "fixtures")]
        {
            crate::fixtures::load(&store, political_group_id).await?;
            return Ok(());
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    use crate::{AppState, router, test_utils::response_body_string};

    use super::*;

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
                    .uri("/dev/login?name=alice&fixtures=false")
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
            Some(dev_political_group_id("alice"))
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
                    .uri("/dev/login?name=empty-store&fixtures=false")
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

        let store = state
            .store_registry
            .get_or_create(dev_political_group_id("empty-store").uuid())
            .await
            .expect("store");
        assert_eq!(store.get_person_count(), 0);
        assert_eq!(store.get_candidate_list_count(), 0);
        assert_eq!(
            store.get_political_group().id,
            dev_political_group_id("empty-store")
        );
    }

    #[cfg(feature = "fixtures")]
    #[tokio::test]
    async fn dev_login_with_fixtures_loads_fixture_data() {
        let state = AppState::new_for_tests().await;
        let app = router::create(state.clone()).with_state(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/dev/login?name=fixture-store&fixtures=true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let store = state
            .store_registry
            .get_or_create(dev_political_group_id("fixture-store").uuid())
            .await
            .expect("store");
        assert!(store.get_person_count() > 0);
        assert!(store.get_candidate_list_count() > 0);
        assert_eq!(
            store.get_political_group().id,
            dev_political_group_id("fixture-store")
        );
    }
}
