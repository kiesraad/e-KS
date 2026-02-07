use axum::extract::{FromRef, FromRequestParts, Path};
use serde::Deserialize;

use crate::{
    AppError, AppStore,
    list_submitters::{ListSubmitter, ListSubmitterId},
};

#[derive(Deserialize)]
struct ListSubmitterPathParams {
    #[serde(alias = "submitter_id")]
    submitter_id: ListSubmitterId,
}

impl<S> FromRequestParts<S> for ListSubmitter
where
    S: Clone + Send + Sync + 'static,
    AppStore: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let store = AppStore::from_ref(state);
        let Path(ListSubmitterPathParams { submitter_id }) =
            Path::<ListSubmitterPathParams>::from_request_parts(parts, state).await?;

        store.get_list_submitter(submitter_id)
    }
}
