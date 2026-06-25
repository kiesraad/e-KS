use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::{
    AppError, CsbStoreData, StreamId, political_groups::PoliticalGroup, store::StoreRegistry,
};

pub struct CsbPoliticalGroup {
    pub political_group: PoliticalGroup,
    pub stream_id: StreamId,
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
            political_groups.push(CsbPoliticalGroup {
                stream_id: store.stream_id,
                political_group: store.data.read().imported_data.political_group.clone(),
            });
        }

        Ok(CsbPoliticalGroups(political_groups))
    }
}
