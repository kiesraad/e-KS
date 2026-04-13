use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};

mod index;
mod not_found;
mod switch_election;
mod switch_locale;

use crate::{AppError, AppState};

pub use not_found::not_found;

#[derive(TypedPath)]
#[typed_path("/", rejection(AppError))]
pub struct IndexPath;

#[derive(TypedPath)]
#[typed_path("/language", rejection(AppError))]
pub struct SwitchLanguagePath;

#[derive(TypedPath)]
#[typed_path("/switch-election", rejection(AppError))]
pub struct SwitchElectionPath;

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(index::index)
        .typed_post(switch_locale::switch_language)
        .typed_get(switch_election::switch_election)
        .typed_post(switch_election::switch_election_submit)
}
