use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{AppError, AppState, core::ModelLocale};

mod documents;
mod index;
#[cfg(all(test, feature = "net-tests", feature = "embed-typst"))]
mod integration_tests;

#[derive(TypedPath, Deserialize)]
#[typed_path("/submit", rejection(AppError))]
pub struct SubmitPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/generate/{locale}/documents.zip", rejection(AppError))]
pub struct DownloadDocumentsPath {
    locale: ModelLocale,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(index::index)
        .typed_get(documents::gen_documents)
}
