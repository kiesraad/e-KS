use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use crate::{
    AppError, CsbMainStore, CsbMainStoreData, Session, csb::CSB_MAIN_STREAM_ID,
    store::StoreRegistry,
};

impl<S> FromRequestParts<S> for CsbMainStore
where
    S: Send + Sync,
    StoreRegistry<CsbMainStoreData>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let election = Session::from_request_parts(parts, state)
            .await?
            .require_current_election()?;

        let registry = StoreRegistry::<CsbMainStoreData>::from_ref(state);
        registry.get_or_create(CSB_MAIN_STREAM_ID, election).await
    }
}
