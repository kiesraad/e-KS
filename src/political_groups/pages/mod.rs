use crate::{AppError, AppState};
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use serde::Deserialize;

mod index;

#[derive(TypedPath, Deserialize)]
#[typed_path("/political-group", rejection(AppError))]
pub struct PoliticalGroupPath;

pub fn router() -> Router<AppState> {
    Router::new().typed_get(index::index)
}
