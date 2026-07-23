use axum::Router;
use axum_extra::routing::RouterExt;

use crate::AppState;

#[allow(unused_imports)]
pub(crate) use super::paths::*;

pub mod documents;
mod index;
#[cfg(all(test, feature = "net-tests"))]
mod integration_tests;

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(index::index)
        .typed_get(documents::gen_documents)
}
