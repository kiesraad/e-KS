use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};

mod index;
mod not_found;
mod switch_locale;

use crate::{AppError, AppState};

pub use not_found::not_found;

#[derive(TypedPath)]
#[typed_path("/", rejection(AppError))]
pub struct IndexPath;

#[derive(TypedPath)]
#[typed_path("/language", rejection(AppError))]
pub struct SwitchLanguagePath;

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(index::index)
        .typed_post(switch_locale::switch_language)
}
