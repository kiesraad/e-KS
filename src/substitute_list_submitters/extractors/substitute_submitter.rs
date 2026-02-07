use axum::extract::{FromRef, FromRequestParts, Path};
use serde::Deserialize;

use crate::{
    AppError, AppStore,
    substitute_list_submitters::{SubstituteSubmitter, SubstituteSubmitterId},
};

#[derive(Deserialize)]
struct SubstituteSubmitterPathParams {
    #[serde(alias = "sub_submitter_id")]
    submitter_id: SubstituteSubmitterId,
}

impl<S> FromRequestParts<S> for SubstituteSubmitter
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
        let Path(SubstituteSubmitterPathParams { submitter_id }) =
            Path::<SubstituteSubmitterPathParams>::from_request_parts(parts, state).await?;

        store.get_substitute_submitter(submitter_id)
    }
}
