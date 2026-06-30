use crate::{AppError, AppState};
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};

mod index;

#[derive(TypedPath)]
#[typed_path("/csb", rejection(AppError))]
pub struct CsbIndexPath;

pub fn router() -> Router<AppState> {
    Router::new()
    .typed_get(index::index)
}
