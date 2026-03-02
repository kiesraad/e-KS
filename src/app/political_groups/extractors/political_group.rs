use axum::extract::FromRequestParts;

use crate::{AppError, AppStore, political_groups::PoliticalGroup};

impl<S> FromRequestParts<S> for PoliticalGroup
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let store = AppStore::from_request_parts(parts, state).await?;

        store.get_political_group()
    }
}
