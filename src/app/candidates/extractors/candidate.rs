use axum::extract::{FromRequestParts, Path};

use crate::{AppError, AppStore, Context, candidates::Candidate, trans};

use super::CandidateListAndPersonPathParams;

impl<S> FromRequestParts<S> for Candidate
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
        let Path(CandidateListAndPersonPathParams { list_id, person_id }) =
            Path::<CandidateListAndPersonPathParams>::from_request_parts(parts, state).await?;

        let candidate_list = store.get_candidate_list(list_id).map_err(|err| match err {
            AppError::NotFound(_) => AppError::NotFound(
                trans!("person.not_found_in_candidate_list", context.session.locale).to_string(),
            ),
            _ => err,
        })?;
        let candidate = candidate_list
            .get_candidate(&store, person_id)
            .await
            .map_err(|err| match err {
                AppError::NotFound(_) => AppError::NotFound(
                    trans!("person.not_found_in_candidate_list", context.session.locale)
                        .to_string(),
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
    use tower::ServiceExt;

    use crate::{
        AppState, AppStore, Locale,
        candidate_lists::CandidateListId,
        persons::PersonId,
        render_error_pages,
        test_utils::{response_body_string, sample_candidate_list, sample_person},
    };

    #[tokio::test]
    async fn candidate_extractor_loads_candidate() {
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        let app_state = AppState::new_for_tests().await;
        let store = AppStore::new_for_test().await;
        list.create(&store).await.unwrap();
        person.create(&store).await.unwrap();
        list.clone()
            .update_order(&store, &[person.id])
            .await
            .unwrap();

        let app =
            Router::new()
                .route(
                    "/candidate-lists/{list_id}/persons/{person_id}",
                    get(|candidate: Candidate| async move {
                        candidate.person.name.last_name.to_string()
                    }),
                )
                .with_state(app_state);

        let mut request = Request::builder()
            .uri(format!("/candidate-lists/{list_id}/persons/{}", person.id))
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
    async fn candidate_extractor_returns_not_found() {
        let state = AppState::new_for_tests().await;
        let store = AppStore::new_for_test().await;
        let list_id = CandidateListId::new();
        let list = sample_candidate_list(list_id);
        let person = sample_person(PersonId::new());

        list.create(&store).await.unwrap();
        person.create(&store).await.unwrap();

        let app =
            Router::new()
                .route(
                    "/candidate-lists/{list_id}/persons/{person_id}",
                    get(|candidate: Candidate| async move {
                        candidate.person.name.last_name.to_string()
                    }),
                )
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    render_error_pages,
                ))
                .with_state(state);

        let mut request = Request::builder()
            .uri(format!("/candidate-lists/{list_id}/persons/{}", person.id))
            .header(header::ACCEPT_LANGUAGE, "en")
            .body(Body::empty())
            .unwrap();
        let mut session = crate::Session::new_with_locale(Locale::En);
        session.set_political_group(crate::PoliticalGroupId::new());
        request.extensions_mut().insert(session);
        request.extensions_mut().insert(store.clone());

        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_body_string(response).await;
        let expected = "Not found";
        assert!(body.contains(expected));
    }
}
