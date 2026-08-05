use axum::Router;
use axum_extra::routing::RouterExt;

use crate::AppRequestState;

#[allow(unused_imports)]
pub(crate) use super::paths::*;

pub mod documents;
mod index;

pub fn router<S: AppRequestState>() -> Router<S> {
    Router::new()
        .typed_get(index::index)
        .typed_get(documents::gen_documents)
}
