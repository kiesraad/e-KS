use axum::extract::{FromRef, FromRequestParts, Path};
use sqlx::PgPool;

use crate::{
    AppError, Context, CsrfTokens,
    candidate_lists::{self, CandidateList},
    trans,
};

use super::CandidateListPathParams;

impl<S> FromRequestParts<S> for CandidateList
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
    CsrfTokens: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);
        let context = Context::from_request_parts(parts, state).await?;
        let Path(CandidateListPathParams { list_id }) =
            Path::<CandidateListPathParams>::from_request_parts(parts, state).await?;

        let candidate_list = candidate_lists::get_candidate_list(&pool, list_id)
            .await?
            .ok_or(AppError::NotFound(trans!(
                "candidate_list.not_found",
                context.locale,
                list_id
            )))?;

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
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::{
        AppState, Locale,
        candidate_lists::{self, CandidateListId},
        render_error_pages,
        test_utils::{response_body_string, sample_candidate_list},
        trans,
    };

    #[sqlx::test]
    async fn candidate_list_extractor_loads_list(pool: PgPool) {
        let list = sample_candidate_list(CandidateListId::new());
        candidate_lists::create_candidate_list(&pool, &list)
            .await
            .unwrap();

        let app_state = AppState::new_for_tests(&pool).await;

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

    #[sqlx::test]
    async fn candidate_list_extractor_returns_not_found(pool: PgPool) {
        let state = AppState::new_for_tests(&pool).await;
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
