//! Typed paths for the CSB audit-log routes.

use axum_extra::routing::TypedPath;
use serde::Deserialize;

use crate::{AppError, StreamId};

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/audit-log", rejection(AppError))]
pub struct CsbAuditLogPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/csb/audit-log/{stream_id}/{event_id}", rejection(AppError))]
pub struct CsbAuditLogDetailPath {
    pub stream_id: StreamId,
    pub event_id: usize,
}
