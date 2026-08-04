use axum::Router;
use axum_extra::routing::RouterExt;

use crate::AppRequestState;

mod update;

pub fn router<S: AppRequestState>() -> Router<S> {
    Router::new()
        .typed_get(update::update_list_designation)
        .typed_post(update::update_list_designation_submit)
}
