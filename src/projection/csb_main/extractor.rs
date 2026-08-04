use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{AppError, AppRequestState, CsbMainStore, Session, projection::CSB_MAIN_STREAM_ID};

impl<S: AppRequestState> FromRequestParts<S> for CsbMainStore {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let election = Session::from_request_parts(parts, state)
            .await?
            .require_current_election()?;

        let registry = state.csb_main_store_registry();
        registry.get_or_create(CSB_MAIN_STREAM_ID, election).await
    }
}
