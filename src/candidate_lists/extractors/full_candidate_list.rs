use axum::extract::{FromRef, FromRequestParts, Path};
use sqlx::PgPool;

use crate::{
    AppError, Context, CsrfTokens,
    candidate_lists::{self, FullCandidateList},
    trans,
};

use super::CandidateListPathParams;

impl<S> FromRequestParts<S> for FullCandidateList
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
        let mut conn = PgPool::from_ref(state).acquire().await?;
        let context = Context::from_request_parts(parts, state)
            .await
            .unwrap_or_default();
        let Path(CandidateListPathParams { list_id }) =
            Path::<CandidateListPathParams>::from_request_parts(parts, state).await?;

        let full_list = candidate_lists::get_full_candidate_list(&mut conn, list_id)
            .await?
            .ok_or(AppError::NotFound(trans!(
                "candidate_list.not_found",
                context.locale,
                list_id
            )))?;

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
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::{
        AppState, Locale,
        candidate_lists::{self, CandidateListId},
        persons::{self, PersonId},
        render_error_pages,
        test_utils::{response_body_string, sample_candidate_list, sample_person},
    };

    #[sqlx::test]
    async fn full_candidate_list_extractor_loads_candidates(pool: PgPool) {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        let mut conn = pool.acquire().await.unwrap();
        candidate_lists::create_candidate_list(&mut conn, &list)
            .await
            .unwrap();
        persons::create_person(&mut conn, &person).await.unwrap();
        candidate_lists::update_candidate_list_order(&mut conn, list_id, &[person.id])
            .await
            .unwrap();

        let app = Router::new()
            .route(
                "/candidate-lists/{list_id}/full",
                get(|full_list: FullCandidateList| async move {
                    full_list
                        .candidates
                        .first()
                        .expect("candidate")
                        .person
                        .last_name
                        .clone()
                }),
            )
            .with_state(AppState::new_for_tests(pool));

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

    #[sqlx::test]
    async fn full_candidate_list_extractor_returns_not_found(pool: PgPool) {
        let state = AppState::new_for_tests(pool);
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
