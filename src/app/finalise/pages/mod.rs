use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{AppError, AppState, core::ModelLocale};

pub mod documents;
mod index;
#[cfg(all(test, feature = "net-tests", feature = "embed-typst"))]
mod integration_tests;

#[derive(TypedPath, Deserialize)]
#[typed_path("/finalise", rejection(AppError))]
pub struct FinalisePath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/generate/{locale}/documents.zip", rejection(AppError))]
pub struct DownloadDocumentsPath {
    pub locale: ModelLocale,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(index::index)
        .typed_get(documents::gen_documents)
}
