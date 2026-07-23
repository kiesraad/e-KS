use crate::AppState;
use axum::Router;
use axum_extra::routing::RouterExt;

mod index;

pub fn router() -> Router<AppState> {
    Router::new().typed_get(index::index)
}
