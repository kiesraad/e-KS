use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::{
    AppError, CsbStore, CsbStoreData, StreamId, political_groups::PoliticalGroup,
    store::StoreRegistry,
};

pub struct CsbPoliticalGroup {
    pub political_group: PoliticalGroup,
    pub stream_id: StreamId,
    pub is_examination_finished: bool,
}

impl CsbPoliticalGroup {
    pub fn new_from_csb_store(store: &CsbStore) -> Self {
        let store_data = store.data.read();
        CsbPoliticalGroup {
            political_group: store_data.imported_data.political_group.clone(),
            stream_id: store.stream_id,
            is_examination_finished: store_data.is_examination_finished,
        }
    }
}

/// Extracts all imported political groups visible to the CSB scope.
pub struct CsbPoliticalGroups(pub Vec<CsbPoliticalGroup>);

impl<S> FromRequestParts<S> for CsbPoliticalGroups
where
    S: Send + Sync,
    StoreRegistry<CsbStoreData>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let registry = StoreRegistry::<CsbStoreData>::from_ref(state);

        let mut political_groups = Vec::new();
        for store in registry.stores_by_scope().await? {
            let store_data = store.data.read();
            political_groups.push(CsbPoliticalGroup {
                stream_id: store.stream_id,
                political_group: store_data.imported_data.political_group.clone(),
                is_examination_finished: store_data.is_examination_finished,
            });
        }

        Ok(CsbPoliticalGroups(political_groups))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};

    use crate::{AppState, AppStoreData, CsbEvent, ElectionConfig};

    /// Persist a CSB stream carrying a single import event in the (in-memory)
    /// test registry, returning its `stream_id`.
    async fn seed_csb_store(state: &AppState, election: ElectionConfig) -> StreamId {
        let stream_id = StreamId::new();
        let store = state
            .csb_store_for_stream(stream_id, election)
            .await
            .unwrap();
        store
            .update(CsbEvent::Import {
                hash: "AAAA BBBB".to_string(),
                source_stream_id: StreamId::new(),
                snapshot: Box::new(AppStoreData::default()),
            })
            .await
            .unwrap();
        stream_id
    }

    fn empty_parts() -> axum::http::request::Parts {
        Request::builder()
            .uri("/csb/examination")
            .body(Body::empty())
            .unwrap()
            .into_parts()
            .0
    }

    #[tokio::test]
    async fn returns_every_csb_scoped_political_group() {
        let state = AppState::new_for_tests().await;
        let first = seed_csb_store(&state, ElectionConfig::EK27).await;
        let second = seed_csb_store(&state, ElectionConfig::EK27).await;

        let mut parts = empty_parts();
        let CsbPoliticalGroups(groups) = CsbPoliticalGroups::from_request_parts(&mut parts, &state)
            .await
            .unwrap();

        assert_eq!(groups.len(), 2);
        let stream_ids: Vec<_> = groups.iter().map(|g| g.stream_id).collect();
        assert!(stream_ids.contains(&first));
        assert!(stream_ids.contains(&second));
    }

    #[tokio::test]
    async fn returns_empty_when_nothing_imported() {
        let state = AppState::new_for_tests().await;

        let mut parts = empty_parts();
        let CsbPoliticalGroups(groups) = CsbPoliticalGroups::from_request_parts(&mut parts, &state)
            .await
            .unwrap();

        assert!(groups.is_empty());
    }
}
