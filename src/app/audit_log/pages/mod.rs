use crate::{AppError, AppState};
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

mod list;

#[derive(TypedPath, Deserialize)]
#[typed_path("/audit-log", rejection(AppError))]
pub struct AuditLogPath;

pub fn router() -> Router<AppState> {
    Router::new().typed_get(list::audit_log)
}
