use axum::extract::{FromRef, FromRequestParts, Path};
use serde::Deserialize;

use crate::{
    AppError, Store,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
};

#[derive(Deserialize)]
struct SubstituteSubmitterPathParams {
    #[serde(alias = "sub_submitter_id")]
    submitter_id: SubstituteSubmitterId,
}

impl<S> FromRequestParts<S> for SubstituteSubmitter
where
    S: Clone + Send + Sync + 'static,
    Store: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let store = Store::from_ref(state);
        let Path(SubstituteSubmitterPathParams { submitter_id }) =
            Path::<SubstituteSubmitterPathParams>::from_request_parts(parts, state).await?;

        store.get_substitute_submitter(submitter_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    use crate::{
        AppState,
        test_utils::{response_body_string, sample_substitute_submitter},
    };

    #[tokio::test]
    async fn substitute_submitter_extractor_loads_submitter() {
        let substitute_submitter = sample_substitute_submitter(SubstituteSubmitterId::new());

        let app_state = AppState::new_for_tests().await;
        substitute_submitter.create(&app_state.store).await.unwrap();

        let app = Router::new()
            .route(
                "/political-group/substitute-submitters/{sub_submitter_id}",
                get(|substitute_submitter: SubstituteSubmitter| async move {
                    substitute_submitter.name.last_name.to_string()
                }),
            )
            .with_state(app_state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/political-group/substitute-submitters/{}",
                        substitute_submitter.id
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Bakker"));
    }
}
