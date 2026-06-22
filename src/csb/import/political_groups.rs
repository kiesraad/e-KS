use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;

use crate::{
    AppError, CsbStoreData, Scope, political_groups::PoliticalGroup, store::StoreRegistry,
};

/// Extracts all imported political groups visible to the CSB scope.
pub struct CsbPoliticalGroups(pub Vec<PoliticalGroup>);

impl<S> FromRequestParts<S> for CsbPoliticalGroups
where
    S: Send + Sync,
    StoreRegistry<CsbStoreData>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let registry = StoreRegistry::<CsbStoreData>::from_ref(state);

        let mut political_groups = Vec::new();
        for (stream_id, election) in registry
            .streams_by_scope(Scope::CentralElectoralCommittee)
            .await?
        {
            let store = registry.get_or_create(stream_id, election).await?;
            political_groups.push(store.data.read().imported_data.political_group.clone());
        }

        Ok(CsbPoliticalGroups(political_groups))
    }
}
