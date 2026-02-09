use axum::extract::{FromRef, FromRequestParts, Path};

use crate::{AppError, AppStore, Context, CsrfTokens, candidate_lists::FullCandidateList, trans};

use super::CandidateListPathParams;

impl<S> FromRequestParts<S> for FullCandidateList
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
        let Path(CandidateListPathParams { list_id }) =
            Path::<CandidateListPathParams>::from_request_parts(parts, state).await?;

        let full_list = FullCandidateList::get(&store, list_id).map_err(|_| {
            AppError::NotFound(trans!("candidate_list.not_found", context.locale, list_id))
        })?;

        Ok(full_list)
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
        AppState, Locale,
        candidate_lists::CandidateListId,
        persons::PersonId,
        render_error_pages,
        test_utils::{response_body_string, sample_candidate_list, sample_person},
    };

    #[tokio::test]
    async fn full_candidate_list_extractor_loads_candidates() {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        let app_state = AppState::new_for_tests().await;
        list.create(&app_state.store)
            .await
            .expect("create candidate list");
        person
            .create(&app_state.store)
            .await
            .expect("create person");
        list.clone()
            .update_order(&app_state.store, &[person.id])
            .await
            .expect("update candidate list order");

        let app = Router::new()
            .route(
                "/candidate-lists/{list_id}/full",
                get(|full_list: FullCandidateList| async move {
                    let candidate = full_list.candidates.first().expect("candidate");

                    candidate.person.name.last_name.to_string()
                }),
            )
            .with_state(app_state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/candidate-lists/{list_id}/full"))
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
    async fn full_candidate_list_extractor_returns_not_found() {
        let state = AppState::new_for_tests().await;
        let list_id = CandidateListId::new();

        let app = Router::new()
            .route(
                "/candidate-lists/{list_id}/full",
                get(|full_list: FullCandidateList| async move { full_list.list.id.to_string() }),
            )
            .layer(middleware::from_fn_with_state(
                state.clone(),
                render_error_pages,
            ))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/candidate-lists/{list_id}/full"))
                    .header(header::ACCEPT_LANGUAGE, "en")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_string(response).await;
        let expected = trans!("candidate_list.not_found", Locale::En, list_id);

        assert!(body.contains(&expected));
    }
}
