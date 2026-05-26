use axum::extract::{FromRequestParts, Path};
use serde::Deserialize;

use crate::{
    AppError, AppStore,
    name_authorisations::{NameAuthorisation, NameAuthorisationId},
};

#[derive(Deserialize)]
struct NameAuthorisationPathParams {
    authorisation_id: NameAuthorisationId,
}

impl<S> FromRequestParts<S> for NameAuthorisation
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
        let Path(NameAuthorisationPathParams { authorisation_id }) =
            Path::<NameAuthorisationPathParams>::from_request_parts(parts, state).await?;

        store.get_name_authorisation(authorisation_id)
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
        test_utils::{response_body_string, sample_name_authorisation},
    };

    #[tokio::test]
    async fn name_authorisation_extractor_loads_name() {
        let name_auth = sample_name_authorisation(NameAuthorisationId::new());

        let app_state = AppState::new_for_tests().await;
        let store = AppStore::new_for_test();
        name_auth
            .create(&store)
            .await
            .expect("create name authorisation");

        let app =
            Router::new()
                .route(
                    "/political-group/name-authorisation/{authorisation_id}",
                    get(|name_auth: NameAuthorisation| async move {
                        name_auth.name.last_name.to_string()
                    }),
                )
                .with_state(app_state);

        let mut request = Request::builder()
            .uri(format!(
                "/political-group/name-authorisation/{}",
                name_auth.id
            ))
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(store.clone());

        let response = app.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response_body_string(response).await;
        assert!(body.contains("Jansen"));
    }
}
