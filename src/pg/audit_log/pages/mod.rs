use crate::AppState;
use axum::Router;
use axum_extra::routing::RouterExt;

mod detail;
mod list;

#[cfg(test)]
mod integration_tests;

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(list::audit_log)
        .typed_get(detail::audit_log_detail)
        .typed_get(detail::audit_log_gen_documents)
}
