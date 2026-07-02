use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{AppError, AppState, StreamId};

mod detail;
mod list;

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/audit-log", rejection(AppError))]
pub struct CsbAuditLogPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/audit-log/{stream_id}/{event_id}", rejection(AppError))]
pub struct CsbAuditLogDetailPath {
    pub stream_id: StreamId,
    pub event_id: usize,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(list::csb_audit_log)
        .typed_get(detail::csb_audit_log_detail)
}
