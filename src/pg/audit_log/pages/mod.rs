use crate::AppRequestState;
use axum::Router;
use axum_extra::routing::RouterExt;

mod detail;
mod list;

pub fn router<S: AppRequestState>() -> Router<S> {
    Router::new()
        .typed_get(list::audit_log)
        .typed_get(detail::audit_log_detail)
        .typed_get(detail::audit_log_gen_documents)
}
