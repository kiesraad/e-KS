use axum::Router;
use axum_extra::routing::RouterExt;

use crate::AppRequestState;

use super::paths::{CsbAuditLogDetailPath, CsbAuditLogPath};

mod detail;
mod list;

pub fn router<S: AppRequestState>() -> Router<S> {
    Router::new()
        .typed_get(list::csb_audit_log::<S>)
        .typed_get(detail::csb_audit_log_detail::<S>)
}
