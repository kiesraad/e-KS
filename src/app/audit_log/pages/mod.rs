use crate::{AppError, AppState, core::ModelLocale};
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

mod detail;
mod list;

#[cfg(test)]
mod integration_tests;

#[derive(TypedPath, Deserialize)]
#[typed_path("/audit-log", rejection(AppError))]
pub struct AuditLogPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/audit-log/{event_id}", rejection(AppError))]
pub struct AuditLogDetailPath {
    pub event_id: usize,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/audit-log/{event_id}/{locale}/documents.zip", rejection(AppError))]
pub struct AuditLogDownloadDocumentsPath {
    pub event_id: usize,
    pub locale: ModelLocale
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(list::audit_log)
        .typed_get(detail::audit_log_detail)
        .typed_get(detail::audit_log_gen_documents)
}
