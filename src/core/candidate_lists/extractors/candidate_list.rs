use axum::extract::{FromRef, FromRequestParts, Path};

use crate::{AppError, AppStore, Context, CsrfTokens, candidate_lists::CandidateList, trans};

use super::CandidateListPathParams;

impl<S> FromRequestParts<S> for CandidateList
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

        let candidate_list = store.get_candidate_list(list_id).map_err(|_| {
            AppError::NotFound(trans!("candidate_list.not_found", context.locale, list_id))
        })?;

        Ok(candidate_list)
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
        AppEvent, AppState, Locale,
        candidate_lists::CandidateListId,
        render_error_pages,
        test_utils::{response_body_string, sample_candidate_list},
        trans,
    };

    #[tokio::test]
    async fn candidate_list_extractor_loads_list() {
        let list = sample_candidate_list(CandidateListId::new());

        let app_state = AppState::new_for_tests().await;
        app_state
            .store
            .update(AppEvent::CreateCandidateList(list.clone()))
            .await
            .unwrap();

        let app = Router::new()
            .route(
                "/candidate-lists/{list_id}",
                get(|candidate_list: CandidateList| async move { candidate_list.id.to_string() }),
            )
            .with_state(app_state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/candidate-lists/{}", list.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains(&list.id.to_string()));
    }

    #[tokio::test]
    async fn candidate_list_extractor_returns_not_found() {
        let state = AppState::new_for_tests().await;
        let list_id = CandidateListId::new();

        let app = Router::new()
            .route(
                "/candidate-lists/{list_id}",
                get(|candidate_list: CandidateList| async move { candidate_list.id.to_string() }),
            )
            .layer(middleware::from_fn_with_state(
                state.clone(),
                render_error_pages,
            ))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/candidate-lists/{}", list_id))
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
