use axum::Router;
use axum_extra::routing::RouterExt;

use crate::AppState;

use super::paths::*;

mod detail;
mod list;

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(list::csb_audit_log)
        .typed_get(detail::csb_audit_log_detail)
}
