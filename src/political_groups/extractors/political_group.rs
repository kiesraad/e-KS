use axum::extract::{FromRef, FromRequestParts};

use crate::{AppError, AppStore, political_groups::PoliticalGroup};

impl<S> FromRequestParts<S> for PoliticalGroup
where
    S: Clone + Send + Sync + 'static,
    AppStore: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let store = AppStore::from_ref(state);

        store.get_political_group()
    }
}
