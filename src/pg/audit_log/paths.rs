//! Typed paths for the audit-log routes.

use axum_extra::routing::TypedPath;
use serde::Deserialize;

use crate::{AppError, core::ModelLocale};

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
    pub locale: ModelLocale,
}
