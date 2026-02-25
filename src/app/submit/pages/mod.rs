use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

use crate::{AppError, AppState, candidate_lists::CandidateListId};

mod h1;
mod index;

#[derive(TypedPath, Deserialize)]
#[typed_path("/submit", rejection(AppError))]
pub struct SubmitPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/generate/{list_id}/h1.pdf", rejection(AppError))]
pub struct DownloadH1Path {
    list_id: CandidateListId,
}

pub fn router() -> Router<AppState> {
    Router::new().typed_get(index::index).typed_get(h1::gen_h1)
}
