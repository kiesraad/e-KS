use axum::extract::{FromRequestParts, Path};

use crate::{AppError, AppStore, Context, candidate_lists::FullCandidateList, trans};

use super::CandidateListPathParams;

impl<S> FromRequestParts<S> for FullCandidateList
where
    S: Clone + Send + Sync + 'static,
    AppStore: FromRequestParts<S, Rejection = AppError>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let store = AppStore::from_request_parts(parts, state).await?;
        let context = Context::from_request_parts(parts, state).await?;
        let Path(CandidateListPathParams { list_id }) =
            Path::<CandidateListPathParams>::from_request_parts(parts, state).await?;

        let full_list = FullCandidateList::get(&store, list_id).map_err(|_| {
            AppError::NotFound(trans!(
                "candidate_list.not_found",
                context.session.locale,
                list_id
            ))
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
        AppState, AppStore, Locale,
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
        let store = AppStore::new_for_test().await;
        list.create(&store).await.expect("create candidate list");
        person.create(&store).await.expect("create person");
        list.clone()
            .update_order(&store, &[person.id])
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

        let mut request = Request::builder()
            .uri(format!("/candidate-lists/{list_id}/full"))
            .body(Body::empty())
            .unwrap();
        let mut session = crate::Session::new_with_locale(Locale::En);
        session.set_political_group(crate::PoliticalGroupId::new());
        request.extensions_mut().insert(session);
        request.extensions_mut().insert(store.clone());

        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));
    }

    #[tokio::test]
    async fn full_candidate_list_extractor_returns_not_found() {
        let state = AppState::new_for_tests().await;
        let list_id = CandidateListId::new();
        let store = AppStore::new_for_test().await;

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
            .oneshot({
                let mut request = Request::builder()
                    .uri(format!("/candidate-lists/{list_id}/full"))
                    .header(header::ACCEPT_LANGUAGE, "en")
                    .body(Body::empty())
                    .unwrap();
                let mut session = crate::Session::new_with_locale(Locale::En);
                session.set_political_group(crate::PoliticalGroupId::new());
                request.extensions_mut().insert(session);
                request.extensions_mut().insert(store.clone());
                request
            })
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_string(response).await;
        let expected = trans!("candidate_list.not_found", Locale::En, list_id);

        assert!(body.contains(&expected));
    }
}
