use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{AppError, AppState, candidate_lists::CandidateListId, core::ModelLocale};

mod h1;
mod h9;
mod index;

#[derive(TypedPath, Deserialize)]
#[typed_path("/submit", rejection(AppError))]
pub struct SubmitPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/generate/{list_id}/{locale}/h1.pdf", rejection(AppError))]
pub struct DownloadH1Path {
    list_id: CandidateListId,
    locale: ModelLocale,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/generate/{list_id}/{locale}/h9.zip", rejection(AppError))]
pub struct DownloadH9Path {
    list_id: CandidateListId,
    locale: ModelLocale,
}

impl DownloadH9Path {
    pub fn filename(&self) -> String {
        self.to_string().split("/").last().unwrap().to_string()
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(index::index)
        .typed_get(h1::gen_h1)
        .typed_get(h9::gen_h9)
}
