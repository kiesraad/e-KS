//! Typed paths for the finalise routes.

use axum_extra::routing::TypedPath;
use serde::Deserialize;

use crate::{AppError, core::ModelLocale};

#[derive(TypedPath, Deserialize)]
#[typed_path("/finalise", rejection(AppError))]
pub struct FinalisePath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/generate/{locale}/documents.zip", rejection(AppError))]
pub struct DownloadDocumentsPath {
    pub locale: ModelLocale,
}
