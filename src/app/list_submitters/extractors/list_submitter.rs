use axum::extract::{FromRequestParts, Path};
use serde::Deserialize;

use crate::{
    AppError, AppStore,
    list_submitters::{ListSubmitter, ListSubmitterId},
};

#[derive(Deserialize)]
struct ListSubmitterPathParams {
    #[serde(alias = "submitter_id")]
    submitter_id: ListSubmitterId,
}

impl<S> FromRequestParts<S> for ListSubmitter
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
        let Path(ListSubmitterPathParams { submitter_id }) =
            Path::<ListSubmitterPathParams>::from_request_parts(parts, state).await?;

        store.get_list_submitter(submitter_id)
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
        AppState, AppStore,
        test_utils::{response_body_string, sample_list_submitter},
    };

    #[tokio::test]
    async fn list_submitter_extractor_loads_submitter() {
        let list_submitter = sample_list_submitter(ListSubmitterId::new());

        let app_state = AppState::new_for_tests().await;
        let store = AppStore::new_for_test().await;
        list_submitter.create(&store).await.unwrap();

        let app = Router::new()
            .route(
                "/political-group/list-submitters/{submitter_id}",
                get(|list_submitter: ListSubmitter| async move {
                    list_submitter.name.last_name.to_string()
                }),
            )
            .with_state(app_state);

        let mut request = Request::builder()
            .uri(format!(
                "/political-group/list-submitters/{}",
                list_submitter.id
            ))
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(store.clone());

        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Bos"));
    }
}
