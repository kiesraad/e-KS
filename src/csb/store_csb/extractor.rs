use std::{collections::HashMap, str::FromStr};

use axum::{
    extract::{FromRef, FromRequestParts, Path},
    http::request::Parts,
};

use crate::{AppError, CsbStore, CsbStoreData, Session, StreamId, store::StoreRegistry};

impl<S> FromRequestParts<S> for CsbStore
where
    S: Send + Sync,
    StoreRegistry<CsbStoreData>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(params) =
            Path::<HashMap<String, String>>::from_request_parts(parts, state).await?;

        let stream_id = StreamId::from_str(
            params
                .get("stream_id")
                .ok_or(AppError::InternalServerError)?,
        )
        .map_err(|_| AppError::UserError("Invalid stream id".to_string()))?;

        let registry = StoreRegistry::<CsbStoreData>::from_ref(state);

        let election = Session::from_request_parts(parts, state)
            .await?
            .current_election
            .ok_or(AppError::InternalServerError)?;

        registry.get_store(stream_id, election).await
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

    use crate::{AppState, AppStoreData, CsbEvent, CsbStoreData, ElectionConfig, Locale};

    /// Persist a CSB stream carrying a single import event and return its id.
    async fn seed_csb_store(state: &AppState, election: ElectionConfig) -> StreamId {
        let stream_id = StreamId::new();
        let store = state
            .csb_store_for_stream(stream_id, election)
            .await
            .unwrap();
        store
            .update(CsbEvent::Import {
                hash: [0u8; 32],
                source_stream_id: StreamId::new(),
                snapshot: Box::new(AppStoreData::default()),
            })
            .await
            .unwrap();
        stream_id
    }

    /// Build a router whose single handler echoes the extracted store's id, and
    /// drive a request for `uri` through it with `election` as current election.
    async fn request_store(
        state: AppState,
        uri: String,
        election: ElectionConfig,
    ) -> axum::response::Response {
        let app = Router::new()
            .route(
                "/csb/examination/{stream_id}",
                get(|store: CsbStore| async move { store.stream_id.to_string() }),
            )
            .with_state(state);

        let mut request = Request::builder().uri(uri).body(Body::empty()).unwrap();
        let mut session = Session::new_test_with_locale(Locale::En);
        session.set_current_election(election);
        request.extensions_mut().insert(session);

        app.oneshot(request).await.unwrap()
    }

    #[tokio::test]
    async fn loads_the_store_for_a_known_stream() {
        let state = AppState::new_for_tests().await;
        let stream_id = seed_csb_store(&state, ElectionConfig::EK27).await;

        let response = request_store(
            state,
            format!("/csb/examination/{stream_id}"),
            ElectionConfig::EK27,
        )
        .await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = crate::test_utils::response_body_string(response).await;
        assert_eq!(body, stream_id.to_string());
    }

    #[tokio::test]
    async fn rejects_an_unparseable_stream_id() {
        let state = AppState::new_for_tests().await;

        let response = request_store(
            state,
            "/csb/examination/not-a-stream-id".to_string(),
            ElectionConfig::EK27,
        )
        .await;

        // The id fails to parse before any lookup, yielding a user error.
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn returns_not_found_for_an_unknown_stream() {
        let state = AppState::new_for_tests().await;
        let unknown = StreamId::new();

        let response = request_store(
            state,
            format!("/csb/examination/{unknown}"),
            ElectionConfig::EK27,
        )
        .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn store_data_type_reports_csb_scope() {
        // Guards the scope wiring the extractor relies on for registry lookups.
        assert_eq!(
            <CsbStoreData as crate::store::StoreData>::scope(),
            crate::Scope::ImportedByCsb
        );
    }
}
