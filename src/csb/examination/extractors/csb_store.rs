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

        let election = Session::from_request_parts(parts, state).await?.current_election.ok_or(AppError::InternalServerError)?;

        registry.get_store(stream_id, election).await
    }
}
