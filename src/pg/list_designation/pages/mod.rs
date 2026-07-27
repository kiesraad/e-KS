use axum::Router;
use axum_extra::routing::RouterExt;

use crate::AppState;

mod update;

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(update::update_list_designation)
        .typed_post(update::update_list_designation_submit)
}
