use axum::extract::{FromRef, FromRequestParts, Path};
use serde::Deserialize;

use crate::{
    AppError, AppStore, Locale,
    persons::{Person, PersonId},
    trans,
};

#[derive(Deserialize)]
struct PersonPathParams {
    #[serde(alias = "person_id")]
    person_id: PersonId,
}

impl<S> FromRequestParts<S> for Person
where
    S: Clone + Send + Sync + 'static,
    AppStore: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let store = AppStore::from_ref(state);
        let locale = Locale::from_request_parts(parts, state).await?;
        let Path(PersonPathParams { person_id }) =
            Path::<PersonPathParams>::from_request_parts(parts, state).await?;

        let person = store
            .get_person(person_id)
            .map_err(|_| AppError::NotFound(trans!("person.not_found", locale, person_id)))?;

        Ok(person)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
        middleware,
        routing::get,
    };
    use tower::ServiceExt;

    use crate::{
        AppState, Locale, render_error_pages,
        test_utils::{response_body_string, sample_person},
    };

    #[tokio::test]
    async fn person_extractor_loads_person() {
        let person = sample_person(PersonId::new());

        let app_state = AppState::new_for_tests().await;
        person.create(&app_state.store).await.unwrap();

        let app = Router::new()
            .route(
                "/persons/{person_id}",
                get(|person: Person| async move { person.name.last_name.to_string() }),
            )
            .with_state(app_state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/persons/{}", person.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));
    }

    #[tokio::test]
    async fn person_extractor_returns_not_found() {
        let person_id = PersonId::new();

        let app_state = AppState::new_for_tests().await;

        let app = Router::new()
            .route(
                "/persons/{person_id}",
                get(|person: Person| async move { person.name.last_name.to_string() }),
            )
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                render_error_pages,
            ))
            .with_state(app_state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/persons/{}", person_id))
                    .header(header::ACCEPT_LANGUAGE, "en")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_string(response).await;

        let expected = trans!("person.not_found", Locale::En, person_id);
        assert!(body.contains(&expected));
    }
}
