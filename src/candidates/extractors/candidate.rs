use axum::extract::{FromRef, FromRequestParts, Path};

use crate::{
    AppError, AppStore, Context, CsrfTokens, candidate_lists::CandidateList, candidates::Candidate,
    trans,
};

use super::CandidateListAndPersonPathParams;

impl<S> FromRequestParts<S> for Candidate
where
    S: Clone + Send + Sync + 'static,
    AppStore: FromRef<S>,
    CsrfTokens: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let store = AppStore::from_ref(state);
        let context = Context::from_request_parts(parts, state).await?;
        let Path(CandidateListAndPersonPathParams { list_id, person_id }) =
            Path::<CandidateListAndPersonPathParams>::from_request_parts(parts, state).await?;

        let candidate = CandidateList::get_candidate(&store, list_id, person_id)
            .await
            .map_err(|err| match err {
                AppError::NotFound(_) => AppError::NotFound(
                    trans!("person.not_found_in_candidate_list", context.locale).to_string(),
                ),
                _ => err,
            })?;

        Ok(candidate)
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
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::{
        AppEvent, AppState,
        candidate_lists::CandidateListId,
        persons::PersonId,
        render_error_pages,
        test_utils::{response_body_string, sample_candidate_list, sample_person},
    };

    #[sqlx::test]
    async fn candidate_extractor_loads_candidate(pool: PgPool) {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        let app_state = AppState::new_for_tests(&pool).await;
        app_state
            .store
            .update(AppEvent::CreateCandidateList(list.clone()))
            .await
            .unwrap();
        app_state
            .store
            .update(AppEvent::CreatePerson(person.clone()))
            .await
            .unwrap();
        CandidateList::update_order(&app_state.store, list_id, &[person.id])
            .await
            .unwrap();

        let app = Router::new()
            .route(
                "/candidate-lists/{list_id}/persons/{person_id}",
                get(|candidate: Candidate| async move { candidate.person.last_name.clone() }),
            )
            .with_state(app_state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/candidate-lists/{list_id}/persons/{}", person.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));
    }

    #[sqlx::test]
    async fn candidate_extractor_returns_not_found(pool: PgPool) {
        let state = AppState::new_for_tests(&pool).await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        state
            .store
            .update(AppEvent::CreateCandidateList(list.clone()))
            .await
            .unwrap();
        state
            .store
            .update(AppEvent::CreatePerson(person.clone()))
            .await
            .unwrap();

        let app = Router::new()
            .route(
                "/candidate-lists/{list_id}/persons/{person_id}",
                get(|candidate: Candidate| async move { candidate.person.last_name.clone() }),
            )
            .layer(middleware::from_fn_with_state(
                state.clone(),
                render_error_pages,
            ))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/candidate-lists/{list_id}/persons/{}", person.id))
                    .header(header::ACCEPT_LANGUAGE, "en")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_string(response).await;
        let expected = "Not found";
        assert!(body.contains(expected));
    }
}
