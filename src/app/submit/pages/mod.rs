use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{AppError, AppState, candidate_lists::CandidateListId, core::ModelLocale};

mod documents;
mod index;
#[cfg(all(test, feature = "net-tests", feature = "embed-typst"))]
mod integration_tests;

#[derive(TypedPath, Deserialize)]
#[typed_path("/submit", rejection(AppError))]
pub struct SubmitPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/generate/{list_id}/{locale}/documents.zip", rejection(AppError))]
pub struct DownloadDocumentsPath {
    list_id: CandidateListId,
    locale: ModelLocale,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(index::index)
        .typed_get(documents::gen_documents)
}
